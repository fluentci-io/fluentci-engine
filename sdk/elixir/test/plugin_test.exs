defmodule FluentCI.PluginTest do
  use ExUnit.Case, async: true
  alias FluentCI.Plugin, as: P

  defmodule Example do
    use FluentCI.Plugin

    defplugin greet do
      pipeline("example") |> exec(["/usr/bin/printf", "%s", input()]) |> stdout()
    end
  end

  test "plugin builders are immutable and only construct data" do
    original = P.pipeline("build")

    derived =
      original
      |> P.exec(["missing-program", "space ' $(touch unexpected)"])
      |> P.env("MODE", "release")

    assert original.steps == []
    assert length(derived.steps) == 2
  end

  test "DSL emits a guest using FluentCI exposed functions, without a host SDK" do
    source = Example.__fluentci_exports__() |> FluentCI.Plugin.Codegen.source()
    assert source =~ "#[plugin_fn]"
    assert source =~ "pub fn greet(input: String)"
    assert source =~ "dag().pipeline"
    assert source =~ "pipeline.with_exec"
    assert source =~ "input.clone()"
    refute source =~ "Extism.Plugin"
    refute source =~ "PluginBuilder"
  end

  test "Rust literals cannot escape the generated string" do
    value = "quotes \"#\\ unicode 世界"

    source =
      FluentCI.Plugin.Codegen.source([{:greet, P.pipeline("test") |> P.exec(["echo", value])}])

    assert source =~ value
    assert source =~ "r##\""
  end

  test "invalid exports and non-DSL return values fail clearly" do
    assert_raise ArgumentError, fn -> FluentCI.Plugin.Codegen.source([{:type, "x"}]) end

    assert_raise ArgumentError, fn ->
      FluentCI.Plugin.Codegen.source([{:greet, fn -> :anything end}])
    end

    assert_raise ArgumentError, fn -> P.exec(P.pipeline("x"), []) end
  end
end
