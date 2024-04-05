import { dag } from "@fluentci/sdk/plugin";

declare const Host: {
  inputString: () => string;
  outputString: (str: string) => void;
};

export function sys() {
  const home = dag.getEnv("HOME");
  const os = dag.getOS();
  const arch = dag.getArch();
  Host.outputString(JSON.stringify({ home, os, arch }));
}
