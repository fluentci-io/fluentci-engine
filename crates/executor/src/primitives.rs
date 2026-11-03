//! Small value-producing functions for functional pipeline composition.
//! Construction performs no filesystem, environment, provisioning or process IO.
//! `run` is the effect boundary; it uses the same kernel as CLI and legacy adapters.
pub use crate::matrix::{after, axis, exclude, include, matrix, named, parallel, product, Case};

use crate::{cache::CachePolicy, service::Service, Cancellation, Plan, RunResult, Task};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub fn exec(
    id: impl Into<String>,
    program: impl Into<String>,
    args: Vec<String>,
    cwd: impl Into<PathBuf>,
) -> Task {
    Task {
        id: id.into(),
        program: program.into(),
        args,
        cwd: cwd.into(),
        env: BTreeMap::new(),
        needs: vec![],
        resources: vec![],
        shared_resources: vec![],
        package_install: None,
        package_environment: vec![],
        resource_wait_timeout_ms: None,
        timeout_ms: None,
        cache: None,
        service: None,
    }
}
pub fn env(mut task: Task, values: BTreeMap<String, String>) -> Task {
    task.env.extend(values);
    task
}
pub fn needs(mut task: Task, dependencies: Vec<String>) -> Task {
    task.needs = dependencies;
    task
}
pub fn cache(mut task: Task, policy: CachePolicy) -> Task {
    task.cache = Some(policy);
    task
}
pub fn service(mut task: Task, lifecycle: Service) -> Task {
    task.service = Some(lifecycle);
    task
}
pub fn timeout(mut task: Task, milliseconds: u64) -> Task {
    task.timeout_ms = Some(milliseconds);
    task
}
pub fn resources(mut task: Task, names: Vec<String>) -> Task {
    task.resources = names;
    task
}
pub fn plan(tasks: Vec<Task>) -> Plan {
    Plan {
        version: 1,
        tasks,
        jobs: 1,
        max_services: 8,
        fail_fast: true,
        inherit_env: false,
        workspace: None,
        cache_dir: None,
        resource_dir: None,
    }
}
pub fn jobs(mut plan: Plan, concurrency: usize) -> Plan {
    plan.jobs = concurrency;
    plan
}
pub fn fail_fast(mut plan: Plan, enabled: bool) -> Plan {
    plan.fail_fast = enabled;
    plan
}
pub fn run(
    plan: &Plan,
    directory: &Path,
    cancellation: &Cancellation,
) -> Result<RunResult, String> {
    plan.execute(directory, cancellation)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn composition_preserves_argv_and_has_no_ambient_environment_or_effects() {
        let original = exec(
            "compile",
            "/tool/not-installed",
            vec![
                "one argument with spaces".into(),
                "$(do-not-execute)".into(),
            ],
            "/workspace/not-created",
        );
        let configured = env(
            timeout(needs(original.clone(), vec!["fetch".into()]), 2_000),
            BTreeMap::from([("MODE".into(), "release".into())]),
        );
        let pipeline = jobs(plan(vec![configured]), 4);
        assert_eq!(pipeline.tasks[0].args, original.args);
        assert!(original.needs.is_empty());
        assert!(original.env.is_empty());
        assert_eq!(pipeline.tasks[0].needs, vec!["fetch"]);
        assert!(!pipeline.inherit_env);
        assert_eq!(pipeline.jobs, 4);
    }
}
