import { dag } from "./plugin";

declare const Host: {
  inputString: () => string;
  outputString: (str: string) => void;
};

export function compute_md5() {
  const path = Host.inputString();
  const hash = dag.file(path).md5();
  Host.outputString(hash);
}

export function compute_sha256() {
  const path = Host.inputString();
  const hash = dag.file(path).sha256();
  Host.outputString(hash);
}
