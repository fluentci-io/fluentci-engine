#![cfg(unix)]
use fluentci_executor::{cache::CachePolicy, Cancellation, Plan, Status, Task};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{symlink, PermissionsExt},
    path::Path,
    thread,
};
use tempfile::TempDir;

fn plan(root: &Path) -> Plan {
    fs::write(root.join("input"), "hello").unwrap();
    Plan { workspace: None, version: 1, resource_dir: None, max_services: 8, jobs: 2, fail_fast: true, inherit_env: false,
        cache_dir: Some(root.join("cache")), tasks: vec![Task {
            id: "build".into(), needs: vec![], program: "/bin/sh".into(),
            args: vec!["-c".into(), "/bin/cat input > output; printf executed >> executions; printf log; printf error >&2".into()],
            cwd: root.into(), env: BTreeMap::new(), resources: vec![], shared_resources: vec![], package_install: None, package_environment: vec![], resource_wait_timeout_ms: None, timeout_ms: Some(3000),
            service: None, cache: Some(CachePolicy { relocatable: false, inputs: vec!["input".into()], outputs: vec!["output".into()], toolchain: "fixture-v1".into() })
        }] }
}
fn run(p: &Plan, root: &Path, name: &str) -> fluentci_executor::RunResult {
    let result = p
        .execute(&root.join(name), &Cancellation::default())
        .unwrap();
    assert!(result.success(), "{result:?}");
    result
}

#[test]
fn warm_hit_restores_outputs_and_both_logs_without_reexecution() {
    let dir = TempDir::new().unwrap();
    let p = plan(dir.path());
    let cold = run(&p, dir.path(), "cold");
    assert_eq!(cold.tasks[0].cache.outcome, "stored");
    fs::remove_file(dir.path().join("output")).unwrap();
    let warm = run(&p, dir.path(), "warm");
    assert_eq!(warm.tasks[0].cache.outcome, "hit");
    assert_eq!(
        fs::read_to_string(dir.path().join("output")).unwrap(),
        "hello"
    );
    assert_eq!(fs::read_to_string(&warm.tasks[0].stdout).unwrap(), "log");
    assert_eq!(fs::read_to_string(&warm.tasks[0].stderr).unwrap(), "error");
    assert_eq!(
        fs::read_to_string(dir.path().join("executions")).unwrap(),
        "executed"
    );
}

#[test]
fn input_environment_arguments_toolchain_and_modes_invalidate_keys() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    let mut previous = run(&p, dir.path(), "initial").tasks.remove(0).cache;
    for (index, component) in ["inputs", "environment", "command", "toolchain", "inputs"]
        .iter()
        .enumerate()
    {
        match index {
            0 => fs::write(dir.path().join("input"), "changed").unwrap(),
            1 => {
                p.tasks[0]
                    .env
                    .insert("FLUENTCI_TEST".into(), "changed".into());
            }
            2 => p.tasks[0].args[1].push_str("; : changed"),
            3 => p.tasks[0].cache.as_mut().unwrap().toolchain = "fixture-v2".into(),
            4 => fs::set_permissions(dir.path().join("input"), fs::Permissions::from_mode(0o700))
                .unwrap(),
            _ => unreachable!(),
        }
        let result = run(&p, dir.path(), &format!("change-{index}"));
        let current = &result.tasks[0].cache;
        assert_ne!(previous.key, current.key);
        assert_ne!(
            previous.components[*component],
            current.components[*component]
        );
        assert_eq!(current.outcome, "stored");
        previous = current.clone();
    }
}

#[test]
fn corrupted_blob_and_unsafe_manifest_are_misses_not_trusted_hits() {
    let dir = TempDir::new().unwrap();
    let p = plan(dir.path());
    let cold = run(&p, dir.path(), "cold");
    let key = cold.tasks[0].cache.key.as_ref().unwrap();
    let manifest_path = dir.path().join("cache/actions").join(format!("{key}.json"));
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    let hash = manifest["manifest"]["outputs"][0]["digest"]
        .as_str()
        .unwrap();
    fs::write(dir.path().join("cache/blobs").join(hash), "corrupt").unwrap();
    let repaired = run(&p, dir.path(), "repaired");
    assert_eq!(repaired.tasks[0].cache.outcome, "stored");
    assert!(repaired.tasks[0].cache.reason.contains("integrity"));
    manifest["manifest"]["outputs"][0]["path"] = "../escape".into();
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let result = run(&p, dir.path(), "unsafe");
    assert!(result.tasks[0].cache.reason.contains("checksum"));
    assert_eq!(
        fs::read_to_string(dir.path().join("output")).unwrap(),
        "hello"
    );
}

#[test]
fn concurrent_identical_actions_execute_once() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    p.tasks[0].args[1].insert_str(0, "/bin/sleep 0.1; ");
    thread::scope(|scope| {
        let a = scope.spawn(|| run(&p, dir.path(), "a"));
        let b = scope.spawn(|| run(&p, dir.path(), "b"));
        let a = a.join().unwrap();
        let b = b.join().unwrap();
        let outcomes = [&a.tasks[0].cache.outcome, &b.tasks[0].cache.outcome];
        assert!(outcomes.iter().any(|s| s.as_str() == "hit"));
        assert!(outcomes.iter().any(|s| s.as_str() == "stored"));
    });
    assert_eq!(
        fs::read_to_string(dir.path().join("executions")).unwrap(),
        "executed"
    );
}

#[test]
fn directories_empty_directories_and_executable_permissions_restore() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    p.tasks[0].cache.as_mut().unwrap().outputs = vec!["dist".into()];
    p.tasks[0].args[1] =
        "/bin/mkdir -p dist/empty; printf binary > dist/tool; /bin/chmod 755 dist/tool".into();
    run(&p, dir.path(), "cold");
    fs::remove_dir_all(dir.path().join("dist")).unwrap();
    let result = run(&p, dir.path(), "warm");
    assert_eq!(result.tasks[0].cache.outcome, "hit");
    assert!(dir.path().join("dist/empty").is_dir());
    assert_eq!(
        fs::metadata(dir.path().join("dist/tool"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
}

#[test]
fn missing_outputs_mutated_inputs_and_failed_commands_are_not_published() {
    for (index, script) in [
        "true",
        "printf changed > input; printf out > output",
        "printf out > output; exit 1",
    ]
    .iter()
    .enumerate()
    {
        let dir = TempDir::new().unwrap();
        let mut p = plan(dir.path());
        p.tasks[0].args[1] = script.to_string();
        let result = p
            .execute(
                &dir.path().join(format!("run-{index}")),
                &Cancellation::default(),
            )
            .unwrap();
        assert_eq!(result.tasks[0].status, Status::Failed);
        assert_eq!(
            fs::read_dir(dir.path().join("cache/actions"))
                .unwrap()
                .count(),
            0
        );
    }
}

#[test]
fn inherited_environment_overlap_and_symlinks_are_rejected() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    p.inherit_env = true;
    assert!(p.validate().is_err());
    p.inherit_env = false;
    p.tasks[0].cache.as_mut().unwrap().outputs = vec!["input/nested".into()];
    assert!(p.validate().is_err());
    p.tasks[0].cache.as_mut().unwrap().outputs = vec!["output".into()];
    fs::remove_file(dir.path().join("input")).unwrap();
    symlink("/etc/hosts", dir.path().join("input")).unwrap();
    let error = p
        .execute(&dir.path().join("run"), &Cancellation::default())
        .unwrap_err();
    assert!(error.contains("symlink"));
    assert!(!dir.path().join("run").exists());
}

#[test]
fn cache_and_log_storage_cannot_overlap_declared_inputs_or_outputs() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    p.cache_dir = Some(dir.path().join("output/cache"));
    assert!(p.validate().unwrap_err().contains("overlaps"));
    p.cache_dir = Some(dir.path().join("cache"));
    assert!(p
        .execute(&dir.path().join("output/run"), &Cancellation::default())
        .unwrap_err()
        .contains("overlaps"));
    assert!(!dir.path().join("output").exists());
}

#[test]
fn relative_path_lookup_is_pinned_to_the_hashed_executable_in_task_cwd() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    fs::create_dir(dir.path().join("tools")).unwrap();
    let tool = dir.path().join("tools/build-fixture");
    fs::write(&tool, "#!/bin/sh\nprintf original > output\n").unwrap();
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).unwrap();
    p.tasks[0].program = "build-fixture".into();
    p.tasks[0].args.clear();
    p.tasks[0].env.insert("PATH".into(), "tools".into());
    let first = run(&p, dir.path(), "first");
    assert_eq!(
        fs::read_to_string(dir.path().join("output")).unwrap(),
        "original"
    );
    fs::write(&tool, "#!/bin/sh\nprintf changed > output\n").unwrap();
    let changed = run(&p, dir.path(), "changed");
    assert_ne!(
        first.tasks[0].cache.components["executable"],
        changed.tasks[0].cache.components["executable"]
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("output")).unwrap(),
        "changed"
    );
}

#[test]
fn unordered_declared_writers_are_rejected_before_run_creation() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    let mut reader = p.tasks[0].clone();
    reader.id = "reader".into();
    reader.cache.as_mut().unwrap().inputs = vec!["output/nested".into()];
    reader.cache.as_mut().unwrap().outputs = vec!["result".into()];
    p.tasks.push(reader);
    // Even jobs=1 or a common lock must not silently choose the dataflow order.
    p.jobs = 1;
    for task in &mut p.tasks {
        task.resources.push("workspace".into());
    }
    let run_dir = dir.path().join("rejected");
    let error = p.execute(&run_dir, &Cancellation::default()).unwrap_err();
    assert!(error.contains("unordered cache paths overlap"), "{error}");
    assert!(!run_dir.exists());
    assert!(!dir.path().join("executions").exists());
    p.tasks[1].needs.push("build".into());
    p.validate().unwrap();
}

#[test]
fn shared_inputs_and_transitively_ordered_writers_are_valid() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    let mut other = p.tasks[0].clone();
    other.id = "other".into();
    other.cache.as_mut().unwrap().outputs = vec!["other-output".into()];
    p.tasks.push(other);
    p.validate().unwrap();
    p.tasks[1].cache.as_mut().unwrap().outputs = vec!["output".into()];
    assert!(p
        .validate()
        .unwrap_err()
        .contains("unordered cache paths overlap"));
    let mut middle = p.tasks[0].clone();
    middle.id = "middle".into();
    middle.cache = None;
    middle.needs = vec!["build".into()];
    p.tasks.push(middle);
    p.tasks[1].needs = vec!["middle".into()];
    p.validate().unwrap();
}

#[test]
fn workspace_aliases_and_nested_working_directories_cannot_hide_conflicts() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    fs::create_dir(dir.path().join("output")).unwrap();
    let alias = dir.path().join("alias");
    symlink(dir.path().join("output"), &alias).unwrap();
    let mut nested = p.tasks[0].clone();
    nested.id = "nested".into();
    nested.cwd = alias;
    nested.cache.as_mut().unwrap().inputs = vec![];
    nested.cache.as_mut().unwrap().outputs = vec!["nested".into()];
    p.tasks.push(nested);
    assert!(p
        .validate()
        .unwrap_err()
        .contains("unordered cache paths overlap"));
}

#[test]
fn ordered_output_overwrites_restore_each_action_before_consumers() {
    let dir = TempDir::new().unwrap();
    let mut p = plan(dir.path());
    p.tasks[0].args[1] = "printf first > output; printf first >> executions".into();
    let mut second = p.tasks[0].clone();
    second.id = "second".into();
    second.needs = vec!["build".into()];
    second.args[1] = "printf second > output; printf second >> executions".into();
    let mut consumer = second.clone();
    consumer.id = "consumer".into();
    consumer.needs = vec!["second".into()];
    consumer.cache.as_mut().unwrap().inputs = vec!["output".into()];
    consumer.cache.as_mut().unwrap().outputs = vec!["final".into()];
    consumer.args[1] = "/bin/cat output > final; printf consumer >> executions".into();
    p.tasks.extend([second, consumer]);
    run(&p, dir.path(), "cold");
    fs::write(dir.path().join("output"), "unrelated").unwrap();
    fs::remove_file(dir.path().join("final")).unwrap();
    let warm = run(&p, dir.path(), "warm");
    assert!(warm.tasks.iter().all(|task| task.cache.outcome == "hit"));
    assert_eq!(
        fs::read_to_string(dir.path().join("final")).unwrap(),
        "second"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("executions")).unwrap(),
        "firstsecondconsumer"
    );
}
