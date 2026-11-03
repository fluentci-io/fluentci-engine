import { dag } from "../src/plugin.ts";
export function greet() {
  const words = ["/usr/bin/printf", "%s", Host.inputString()].map(word => "'" + word.replace(/'/g, "'\\''") + "'");
  Host.outputString(dag.pipeline("typescript-greeting").withExec(words).stdout());
}
