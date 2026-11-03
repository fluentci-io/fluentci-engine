/// Immutable host plans. Construction and compilation perform no I/O.
import gleam/dict
import gleam/int
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/string

pub type Cache {
  Cache(
    inputs: List(String),
    outputs: List(String),
    toolchain: String,
    relocatable: Bool,
    toolchain_files: List(String),
  )
}

pub type Probe {
  Probe(program: String, args: List(String), interval_ms: Int, timeout_ms: Int)
}

pub type Service {
  Service(
    readiness: Probe,
    health: Option(Probe),
    startup_timeout_ms: Int,
    shutdown_timeout_ms: Int,
    failure_threshold: Int,
  )
}

pub type ProviderKind {
  Homebrew
  MacPortsNative
  MacPortsRust
  FreeBsd
  NetBsd
  OpenBsd
}

pub type Provider {
  Provider(kind: ProviderKind, executable: Option(String))
}

pub opaque type Task {
  Task(
    id: String,
    program: String,
    cwd: String,
    args: List(String),
    needs: List(String),
    env: List(#(String, String)),
    resources: List(String),
    shared_resources: List(String),
    timeout_ms: Option(Int),
    resource_wait_timeout_ms: Option(Int),
    cache: Option(Cache),
    service: Option(Service),
    installation: Option(#(Provider, List(String))),
    packages: List(Provider),
  )
}

pub opaque type Plan {
  Plan(
    tasks: List(Task),
    jobs: Int,
    max_services: Int,
    fail_fast: Bool,
    cache_dir: Option(String),
    resource_dir: Option(String),
    workspace: Option(#(String, List(String))),
  )
}

pub type PlanError {
  InvalidLimits
  InvalidTask
  DuplicateIds
  MissingDependency(String)
}

pub fn task(id: String, program: String, cwd: String) -> Task {
  Task(
    id:,
    program:,
    cwd:,
    args: [],
    needs: [],
    env: [],
    resources: [],
    shared_resources: [],
    timeout_ms: None,
    resource_wait_timeout_ms: None,
    cache: None,
    service: None,
    installation: None,
    packages: [],
  )
}

/// Rename a value without changing its command/dependencies.
pub fn with_id(task: Task, id: String) -> Task {
  Task(..task, id:)
}

/// Join existing dependencies with the complete parent group.
pub fn after_jobs(task: Task, parents: List(Task)) -> Task {
  Task(
    ..task,
    needs: list.unique(list.append(task.needs, list.map(parents, task_id))),
  )
}

pub fn task_id(task: Task) -> String {
  task.id
}

pub fn with_args(task: Task, args: List(String)) -> Task {
  Task(..task, args:)
}

pub fn needs(task: Task, parents: List(Task)) -> Task {
  Task(..task, needs: list.map(parents, task_id))
}

pub fn with_env(task: Task, key: String, value: String) -> Task {
  Task(..task, env: [
    #(key, value),
    ..list.filter(task.env, fn(entry) { entry.0 != key })
  ])
}

pub fn with_resources(task: Task, resources: List(String)) -> Task {
  Task(..task, resources:)
}

pub fn with_shared_resources(task: Task, resources: List(String)) -> Task {
  Task(..task, shared_resources: resources)
}

pub fn with_timeout(task: Task, milliseconds: Int) -> Task {
  Task(..task, timeout_ms: Some(milliseconds))
}

pub fn with_resource_wait_timeout(task: Task, milliseconds: Int) -> Task {
  Task(..task, resource_wait_timeout_ms: Some(milliseconds))
}

pub fn cache(
  inputs: List(String),
  outputs: List(String),
  toolchain: String,
) -> Cache {
  Cache(inputs:, outputs:, toolchain:, relocatable: False, toolchain_files: [])
}

pub fn with_cache(task: Task, cache: Cache) -> Task {
  Task(..task, cache: Some(cache))
}

/// Verify the contents and permissions of absolute toolchain file paths.
pub fn with_toolchain_files(cache: Cache, paths: List(String)) -> Cache {
  Cache(..cache, toolchain_files: paths)
}

pub fn with_relocatable_cache(task: Task, cache: Cache) -> Task {
  Task(..task, cache: Some(Cache(..cache, relocatable: True)))
}

pub fn service(readiness: Probe) -> Service {
  Service(
    readiness:,
    health: None,
    startup_timeout_ms: 30_000,
    shutdown_timeout_ms: 5000,
    failure_threshold: 3,
  )
}

pub fn with_health(service: Service, health: Probe) -> Service {
  Service(..service, health: Some(health))
}

pub fn as_service(task: Task, service: Service) -> Task {
  Task(..task, service: Some(service))
}

pub fn provider(kind: ProviderKind) -> Provider {
  Provider(kind:, executable: None)
}

pub fn with_provider_executable(
  provider: Provider,
  executable: String,
) -> Provider {
  Provider(..provider, executable: Some(executable))
}

pub fn install(
  id: String,
  cwd: String,
  provider: Provider,
  packages: List(String),
) -> Task {
  Task(..task(id, "", cwd), installation: Some(#(provider, packages)))
}

pub fn with_package_environment(task: Task, providers: List(Provider)) -> Task {
  Task(..task, packages: providers)
}

pub fn plan(tasks: List(Task)) -> Plan {
  Plan(
    tasks:,
    jobs: 1,
    max_services: 8,
    fail_fast: True,
    cache_dir: None,
    resource_dir: None,
    workspace: None,
  )
}

pub fn with_jobs(plan: Plan, jobs: Int) -> Plan {
  Plan(..plan, jobs:)
}

pub fn with_max_services(plan: Plan, count: Int) -> Plan {
  Plan(..plan, max_services: count)
}

pub fn with_fail_fast(plan: Plan, enabled: Bool) -> Plan {
  Plan(..plan, fail_fast: enabled)
}

pub fn with_cache_directory(plan: Plan, path: String) -> Plan {
  Plan(..plan, cache_dir: Some(path))
}

pub fn with_resource_directory(plan: Plan, path: String) -> Plan {
  Plan(..plan, resource_dir: Some(path))
}

pub fn with_workspace(
  plan: Plan,
  source: String,
  inputs: List(String),
) -> Plan {
  Plan(..plan, workspace: Some(#(source, inputs)))
}

@external(javascript, "../host_ffi.mjs", "quote")
fn quote(value: String) -> String

fn object(fields: List(#(String, String))) -> String {
  "{"
  <> string.join(
    list.map(fields, fn(field) { quote(field.0) <> ":" <> field.1 }),
    ",",
  )
  <> "}"
}

fn array(values: List(String)) -> String {
  "[" <> string.join(values, ",") <> "]"
}

fn strings(values: List(String)) -> String {
  array(list.map(values, quote))
}

fn boolean(value: Bool) -> String {
  case value {
    True -> "true"
    False -> "false"
  }
}

fn optional(
  value: Option(a),
  key: String,
  encode: fn(a) -> String,
) -> List(#(String, String)) {
  case value {
    None -> []
    Some(value) -> [#(key, encode(value))]
  }
}

fn encode_probe(probe: Probe) -> String {
  object([
    #("program", quote(probe.program)),
    #("args", strings(probe.args)),
    #("interval_ms", int.to_string(probe.interval_ms)),
    #("timeout_ms", int.to_string(probe.timeout_ms)),
  ])
}

fn encode_service(service: Service) -> String {
  object(list.append(
    [
      #("readiness", encode_probe(service.readiness)),
      #("startup_timeout_ms", int.to_string(service.startup_timeout_ms)),
      #("shutdown_timeout_ms", int.to_string(service.shutdown_timeout_ms)),
      #("failure_threshold", int.to_string(service.failure_threshold)),
    ],
    optional(service.health, "health", encode_probe),
  ))
}

fn encode_provider(provider: Provider) -> String {
  let #(manager, backend) = case provider.kind {
    Homebrew -> #("homebrew", "native")
    MacPortsNative -> #("macports", "native")
    MacPortsRust -> #("macports", "rust")
    FreeBsd -> #("freebsd", "native")
    NetBsd -> #("netbsd", "native")
    OpenBsd -> #("openbsd", "native")
  }
  object(list.append(
    [#("manager", quote(manager)), #("backend", quote(backend))],
    optional(provider.executable, "executable", quote),
  ))
}

fn encode_cache(cache: Cache) -> String {
  let base = [
    #("inputs", strings(cache.inputs)),
    #("outputs", strings(cache.outputs)),
    #("toolchain", quote(cache.toolchain)),
  ]
  let base = case cache.toolchain_files {
    [] -> base
    paths -> list.append(base, [#("toolchain_files", strings(paths))])
  }
  object(case cache.relocatable {
    True -> list.append(base, [#("relocatable", "true")])
    False -> base
  })
}

fn encode_task(task: Task) -> String {
  let fields = [
    #("id", quote(task.id)),
    #("program", quote(task.program)),
    #("cwd", quote(task.cwd)),
    #("args", strings(task.args)),
    #("needs", strings(task.needs)),
    #(
      "env",
      object(list.map(task.env, fn(entry) { #(entry.0, quote(entry.1)) })),
    ),
    #("resources", strings(task.resources)),
  ]
  let additions =
    list.flatten([
      optional(task.timeout_ms, "timeout_ms", int.to_string),
      optional(
        task.resource_wait_timeout_ms,
        "resource_wait_timeout_ms",
        int.to_string,
      ),
      optional(task.cache, "cache", encode_cache),
      optional(task.service, "service", encode_service),
      optional(task.installation, "package_install", fn(install) {
        object([
          #("provider", encode_provider(install.0)),
          #("packages", strings(install.1)),
        ])
      }),
      case task.packages {
        [] -> []
        values -> [
          #("package_environment", array(list.map(values, encode_provider))),
        ]
      },
      case task.shared_resources {
        [] -> []
        values -> [#("shared_resources", strings(values))]
      },
    ])
  object(list.append(fields, additions))
}

pub fn compile(plan: Plan) -> Result(String, PlanError) {
  let ids = list.map(plan.tasks, task_id)
  let index = dict.from_list(list.map(ids, fn(id) { #(id, Nil) }))
  let missing =
    list.find(list.flat_map(plan.tasks, fn(task) { task.needs }), fn(id) {
      !dict.has_key(index, id)
    })
  case
    plan.jobs > 0 && plan.max_services > 0,
    dict.size(index) == list.length(ids),
    list.all(plan.tasks, fn(task) {
      task.id != ""
      && task.cwd != ""
      && { task.program != "" || task.installation != None }
      && option.unwrap(task.timeout_ms, 1) > 0
      && option.unwrap(task.resource_wait_timeout_ms, 1) > 0
    }),
    missing
  {
    False, _, _, _ -> Error(InvalidLimits)
    _, False, _, _ -> Error(DuplicateIds)
    _, _, False, _ -> Error(InvalidTask)
    _, _, _, Ok(id) -> Error(MissingDependency(id))
    True, True, True, Error(_) -> {
      let fields = [
        #("version", "1"),
        #("inherit_env", "false"),
        #("jobs", int.to_string(plan.jobs)),
        #("max_services", int.to_string(plan.max_services)),
        #("fail_fast", boolean(plan.fail_fast)),
        #("tasks", array(list.map(plan.tasks, encode_task))),
      ]
      Ok(
        object(list.append(
          fields,
          list.flatten([
            optional(plan.cache_dir, "cache_dir", quote),
            optional(plan.resource_dir, "resource_dir", quote),
            optional(plan.workspace, "workspace", fn(workspace) {
              object([
                #("source", quote(workspace.0)),
                #("inputs", strings(workspace.1)),
              ])
            }),
          ]),
        )),
      )
    }
  }
}
