defmodule FluentCI.HostTest do
  use ExUnit.Case, async: true
  alias FluentCI.Host, as: H

  test "pipe builders preserve originals and compile explicit argv and environment" do
    original = H.task("echo", "/bin/echo", "/not-created-by-compilation")
    value = "quotes \" unicode 雪\n$(touch unexpected)"
    derived = original |> H.args([value]) |> H.env(%{"TEXT" => value}) |> H.timeout_ms(50)
    {:ok, json} = H.plan([derived]) |> H.jobs(2) |> H.compile()
    wire = JSON.decode!(json)
    assert wire["version"] == 1
    assert wire["inherit_env"] == false
    assert hd(wire["tasks"])["args"] == [value]
    assert hd(wire["tasks"])["env"] == %{"TEXT" => value}
    refute Map.has_key?(original.data, :args)
    assert {:ok, _} = H.plan() |> H.compile()
  end

  test "dependency diamonds compile while duplicates, missing edges and cycles fail" do
    a = H.task("a", "/bin/true", ".")
    b = H.task("b", "/bin/true", ".") |> H.needs(["a"])
    c = H.task("c", "/bin/true", ".") |> H.needs(["a"])
    d = H.task("d", "/bin/true", ".") |> H.needs(["b", "c"])
    assert {:ok, _} = H.plan([d, c, b, a]) |> H.compile()
    assert {:error, {:duplicate_id, "a"}} = H.plan([a, a]) |> H.compile()
    assert {:error, {:missing_dependency, "b", "a"}} = H.plan([b]) |> H.compile()
    assert {:error, {:cycle, _}} = H.plan([H.needs(a, ["b"]), b]) |> H.compile()
  end

  test "large reverse-ordered plans share dependency traversal" do
    tasks =
      for i <- 1..1000 do
        H.task("task-#{i}", "/bin/true", ".")
        |> H.needs(if i == 1, do: [], else: ["task-#{i - 1}"])
      end

    assert {:ok, _} = H.plan(Enum.reverse(tasks)) |> H.compile()
  end

  test "all package providers encode and installs reject incompatible execution options" do
    for {kind, manager, backend} <- [
          {:homebrew, "homebrew", "native"},
          {:macports, "macports", "native"},
          {:macports_rust, "macports", "rust"},
          {:freebsd, "freebsd", "native"},
          {:netbsd, "netbsd", "native"},
          {:openbsd, "openbsd", "native"}
        ] do
      provider = H.provider(kind, executable: "/explicit/manager")
      install = H.install("packages", provider, ["jq"], ".")

      consumer =
        H.task("use", "jq", ".") |> H.needs(["packages"]) |> H.package_environment([provider])

      {:ok, json} = H.plan([install, consumer]) |> H.compile()
      [first, second] = JSON.decode!(json)["tasks"]

      assert first["package_install"]["provider"] == %{
               "manager" => manager,
               "backend" => backend,
               "executable" => "/explicit/manager"
             }

      assert second["package_environment"] == [first["package_install"]["provider"]]

      assert {:error, {:invalid_task, "packages"}} =
               H.plan([H.cache(install, [], ["out"], "toolchain")]) |> H.compile()
    end

    invalid = H.install("bad", H.provider(:homebrew, backend: "rust"), ["jq"], ".")
    assert {:error, {:invalid_task, "bad"}} = H.plan([invalid]) |> H.compile()
  end

  test "invalid schemas return field paths without exposing environment values" do
    task = H.task("work", "/bin/true", ".")

    for invalid <- [
          H.jobs(H.plan(), 0),
          H.max_services(H.plan(), -1),
          H.plan([H.args(task, [nil])]),
          H.plan([H.timeout_ms(task, 1.5)]),
          H.plan([H.env(task, %{"BAD=KEY" => "private-secret"})]),
          H.plan([H.env(task, %{"KEY" => <<255>>})]),
          H.plan([H.args(task, [<<0>>])]),
          H.plan([H.cache(task, [], [], "toolchain")]),
          H.plan([H.cache(task, [], ["out"], "t", typo: true)]),
          H.plan([H.service(task, H.probe("/bin/true"), failure_threshold: 0)]),
          H.plan([H.install("bad", H.provider(:unknown), ["jq"], ".")]),
          H.plan([H.install("bad", H.provider(:homebrew), ["--unsafe"], ".")]),
          H.plan([H.install("bad", H.provider(:homebrew), ["jq; echo nope"], ".")]),
          H.plan([%{}])
        ] do
      assert {:error, {:invalid_field, path}} = H.compile(invalid)
      assert is_list(path)
      refute inspect(path) =~ "private-secret"
    end
  end

  test "service and workspace configuration roundtrip" do
    readiness = H.probe("/bin/true", [], interval_ms: 10)
    health = H.probe("/bin/false", [], timeout_ms: 20)
    task = H.task("db", "/bin/sh", ".") |> H.service(readiness, health: health)
    {:ok, json} = H.plan() |> H.add_task(task) |> H.workspace("/source", ["src"]) |> H.compile()
    wire = JSON.decode!(json)
    assert wire["workspace"] == %{"source" => "/source", "inputs" => ["src"]}
    assert hd(wire["tasks"])["service"]["health"]["timeout_ms"] == 20

    assert {:error, {:invalid_task, "db"}} =
             H.plan([H.cache(task, [], ["out"], "t")]) |> H.compile()
  end
end
