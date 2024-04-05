import { dag } from "@fluentci/sdk/plugin";

declare const Host: {
  inputString: () => string;
  outputString: (str: string) => void;
};

export function call_fn() {
  const opts = Host.inputString();
  const [url, fn, ...args] = opts.split(" ");
  const response = dag.call(url, fn, args);
  Host.outputString(response);
}
