import { greet as invoke, matrix as invokeMatrix, workflow as invokeWorkflow, files as invokeFiles } from "./build/dev/javascript/fluentci_plugin_demo/plugin.mjs";
export function greet() {
  const result = invoke(Host.inputString());
  if (!result.isOk()) throw Error(result[0]);
  Host.outputString(result[0]);
}

export function matrix() {
  const result = invokeMatrix(Host.inputString());
  if (!result.isOk()) throw Error(result[0]);
  Host.outputString("matrix complete");
}

export function workflow() {
  const result = invokeWorkflow(Host.inputString());
  if (!result.isOk()) throw Error(result[0]);
  Host.outputString("workflow complete");
}

export function files() {
  const result = invokeFiles(Host.inputString());
  if (!result.isOk()) throw Error(result[0]);
  Host.outputString("files complete");
}
