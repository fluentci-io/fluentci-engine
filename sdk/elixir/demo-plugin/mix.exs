defmodule FluentCI.PluginDemo.MixProject do
  use Mix.Project

  def project do
    [
      app: :fluentci_plugin_demo,
      version: "0.1.0",
      elixir: ">= 1.18.0",
      deps: [{:fluentci, path: ".."}]
    ]
  end
end
