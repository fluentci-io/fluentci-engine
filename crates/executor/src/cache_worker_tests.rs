//! Deterministic scheduler tests: gate cache publication without slow disk fixtures.
use super::*;
use std::sync::{mpsc, Mutex, OnceLock};

type Gate = (mpsc::Sender<()>, mpsc::Receiver<()>);
static GATES: OnceLock<Mutex<HashMap<PathBuf, Gate>>> = OnceLock::new();
fn gate(root: &Path) -> (mpsc::Receiver<()>, mpsc::Sender<()>) {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    GATES
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .insert(root.into(), (started_tx, release_rx));
    (started_rx, release_tx)
}
pub(super) fn pause_save(root: &Path, cancel: &Cancellation) -> Result<(), String> {
    let gate = GATES
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .remove(root);
    if let Some((started, release)) = gate {
        started.send(()).map_err(|e| e.to_string())?;
        loop {
            if cancel.is_cancelled() {
                return Err("cache operation cancelled".into());
            }
            match release.recv_timeout(Duration::from_millis(5)) {
                Ok(()) => return Ok(()),
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(e) => return Err(e.to_string()),
            }
        }
    }
    Ok(())
}
fn plan(root: &Path, jobs: usize) -> Plan {
    serde_json::from_value(serde_json::json!({
        "version": 1, "jobs": jobs, "inherit_env": false, "fail_fast": false,
        "cache_dir": root.join("cache"), "resource_dir": root.join("locks"),
        "tasks": [
            {"id":"cached", "program":"/bin/sh", "args":["-c","printf artifact > output"], "cwd":root,
             "resources":["workspace"], "cache":{"inputs":[],"outputs":["output"],"toolchain":"fixture"}},
            {"id":"dependent", "program":"/bin/sh", "args":["-c","echo ran > dependent"], "cwd":root, "needs":["cached"]}
        ]
    })).unwrap()
}
fn wait_state(path: &Path, id: &str, status: &str) {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let events = fs::read_to_string(path).unwrap_or_default();
        if events
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .any(|event| event["id"] == id && event["status"] == status)
        {
            return;
        }
        assert!(Instant::now() < deadline, "missing state {id}: {status}");
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn publication_keeps_dependencies_and_leases_but_allows_unrelated_work() {
    let root = tempfile::tempdir().unwrap();
    let (started, release) = gate(root.path());
    let mut p = plan(root.path(), 2);
    let mut independent = p.tasks[1].clone();
    independent.id = "independent".into();
    independent.needs.clear();
    independent.args[1] = "/bin/sleep 5".into();
    independent.timeout_ms = Some(100);
    p.tasks.push(independent);
    let run = root.path().join("run");
    thread::scope(|scope| {
        // Drop the sender before scope joins if an assertion fails.
        let release = release;
        let handle = scope.spawn(|| p.execute(&run, &Cancellation::default()).unwrap());
        started.recv_timeout(Duration::from_secs(3)).unwrap();
        wait_state(&run.join("events.jsonl"), "independent", "timed_out");
        assert!(!root.path().join("dependent").exists());
        assert!(matches!(
            resources::acquire(&root.path().join("locks"), &["workspace".into()], &[]).unwrap(),
            resources::Attempt::Busy(_)
        ));
        release.send(()).unwrap();
        let result = handle.join().unwrap();
        assert_eq!(result.tasks[0].status, Status::Succeeded);
        assert_eq!(result.tasks[1].status, Status::Succeeded);
        assert_eq!(result.tasks[2].status, Status::TimedOut);
        assert_eq!(result.tasks[0].cache.outcome, "stored");
        assert!(root.path().join("dependent").exists());
    });
}

#[test]
fn cancellation_joins_publication_before_releasing_leases() {
    let root = tempfile::tempdir().unwrap();
    let (started, _release) = gate(root.path());
    let p = plan(root.path(), 1);
    let cancel = Cancellation::default();
    thread::scope(|scope| {
        let handle = scope.spawn(|| p.execute(&root.path().join("run"), &cancel).unwrap());
        started.recv_timeout(Duration::from_secs(3)).unwrap();
        cancel.cancel();
        let result = handle.join().unwrap();
        assert_eq!(result.tasks[0].status, Status::Cancelled);
        assert!(!root.path().join("dependent").exists());
        assert_eq!(
            fs::read_dir(root.path().join("cache/actions"))
                .unwrap()
                .count(),
            0
        );
        assert!(matches!(
            resources::acquire(&root.path().join("locks"), &["workspace".into()], &[]).unwrap(),
            resources::Attempt::Acquired(_)
        ));
    });
}

#[test]
fn publication_counts_against_jobs_and_obeys_timeout() {
    let root = tempfile::tempdir().unwrap();
    let (started, _release) = gate(root.path());
    let mut p = plan(root.path(), 1);
    p.tasks[0].timeout_ms = Some(300);
    let mut independent = p.tasks[1].clone();
    independent.id = "independent".into();
    independent.needs.clear();
    independent.args[1] = "echo ran > independent".into();
    p.tasks.push(independent);
    thread::scope(|scope| {
        let handle = scope.spawn(|| {
            p.execute(&root.path().join("run"), &Cancellation::default())
                .unwrap()
        });
        started.recv_timeout(Duration::from_secs(3)).unwrap();
        assert!(!root.path().join("independent").exists());
        let result = handle.join().unwrap();
        assert_eq!(result.tasks[0].status, Status::TimedOut);
        assert_eq!(result.tasks[2].status, Status::Succeeded);
        assert!(!root.path().join("dependent").exists());
    });
}

#[test]
fn journal_failure_joins_publication_and_releases_its_leases() {
    let root = tempfile::tempdir().unwrap();
    let (started, _release) = gate(root.path());
    let p = plan(root.path(), 1);
    let run = root.path().join("run");
    thread::scope(|scope| {
        let handle = scope.spawn(|| p.execute(&run, &Cancellation::default()));
        started.recv_timeout(Duration::from_secs(3)).unwrap();
        let stdout = run.join("0.stdout");
        fs::remove_file(&stdout).unwrap();
        fs::create_dir(&stdout).unwrap();
        assert!(handle.join().unwrap().is_err());
    });
    assert_eq!(
        fs::read_dir(root.path().join("cache/actions"))
            .unwrap()
            .count(),
        0
    );
    assert!(matches!(
        resources::acquire(&root.path().join("locks"), &["workspace".into()], &[]).unwrap(),
        resources::Attempt::Acquired(_)
    ));
}
