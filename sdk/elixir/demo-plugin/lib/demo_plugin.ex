defmodule DemoPlugin do
  use FluentCI.Plugin

  defplugin greet do
    pipeline("elixir-greeting")
    |> exec(["/usr/bin/printf", "%s", input()])
    |> stdout()
  end
end
