import fluentci/host
import gleeunit/should

@external(javascript, "./host_test_ffi.mjs", "inspect")
fn inspect(encoded: String) -> Bool

@external(javascript, "./host_test_ffi.mjs", "execute_fixture")
fn execute_fixture(build: fn(String, String) -> String) -> Bool

pub fn immutable_branches_and_validation_test() {
  let root = host.task("root", "/bin/sh", ".")
  let original = host.compile(host.plan([root]))
  let left = root |> host.with_args(["-c", "true"])
  host.compile(host.plan([root])) |> should.equal(original)
  host.compile(host.plan([root, left]))
  |> should.equal(Error(host.DuplicateIds))
  let consumer = host.task("consumer", "/bin/true", ".") |> host.needs([root])
  host.compile(host.plan([consumer]))
  |> should.equal(Error(host.MissingDependency("root")))
  host.plan([root])
  |> host.with_jobs(0)
  |> host.compile
  |> should.equal(Error(host.InvalidLimits))
}

pub fn wire_contract_test() {
  let ports =
    host.provider(host.MacPortsRust)
    |> host.with_provider_executable("/tools/port")
  let install = host.install("packages", ".", ports, ["jq"])
  let probe = host.Probe("/bin/true", [], 10, 100)
  let service = host.service(probe) |> host.with_health(probe)
  let task =
    host.task("db", "/bin/sh", ".")
    |> host.with_args(["literal; $VALUE", "two words", "quote\"\n"])
    |> host.with_env("__proto__", "preserved")
    |> host.with_env("VALUE", "old")
    |> host.with_env("VALUE", "new")
    |> host.needs([install])
    |> host.with_package_environment([ports])
    |> host.as_service(service)
  let assert Ok(encoded) =
    host.plan([install, task])
    |> host.with_workspace("/source", ["src"])
    |> host.compile
  inspect(encoded) |> should.be_true
}

pub fn release_engine_cache_and_workspace_contract_test() {
  execute_fixture(fn(source, cache) {
    let build =
      host.task("build", "/bin/sh", ".")
      |> host.with_args(["-c", "printf '%s' \"$VALUE\" > artifact"])
      |> host.with_env("VALUE", "literal; $VALUE")
      |> host.with_relocatable_cache(host.cache(
        [],
        ["artifact"],
        "gleam-host-v1",
      ))
    let read =
      host.task("read", "/bin/sh", ".")
      |> host.with_args(["-c", "/bin/cat artifact > consumed"])
      |> host.needs([build])
      |> host.with_relocatable_cache(host.cache(
        ["artifact"],
        ["consumed"],
        "gleam-host-v1",
      ))
    let assert Ok(encoded) =
      host.plan([build, read])
      |> host.with_jobs(2)
      |> host.with_workspace(source, [])
      |> host.with_cache_directory(cache)
      |> host.compile
    encoded
  })
  |> should.be_true
}
