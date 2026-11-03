import { greet as invoke } from "./build/dev/javascript/fluentci_plugin_demo/plugin.mjs";
export function greet() {
  const result = invoke(Host.inputString());
  if (!result.isOk()) throw Error(result[0]);
  Host.outputString(result[0]);
}
