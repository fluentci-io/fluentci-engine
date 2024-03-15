import { JobSpec, Workflow } from "fluent_github_actions";

export function generateYaml(): Workflow {
  const workflow = new Workflow("Test");

  const push = {
    branches: ["main"],
  };

  const test: JobSpec = {
    "runs-on": "ubuntu-latest",
    steps: [
      {
        uses: "actions/checkout@v2",
      },
      {
        name: "Setup Fluent CI",
        uses: "fluentci-io/setup-fluentci@v2",
      },
      {
        name: "Run Tests and Build",
        run: "fluentci run rust_pipeline test build",
      },
    ],
  };

  workflow.on({ push }).jobs({ test });

  return workflow;
}
