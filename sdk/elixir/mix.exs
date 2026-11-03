defmodule FluentCI.MixProject do
  use Mix.Project

  def project do
    [
      app: :fluentci,
      version: "0.1.0",
      elixir: ">= 1.18.0",
      deps: [],
      description: "Immutable host execution plans for FluentCI"
    ]
  end

  def application, do: []
end
