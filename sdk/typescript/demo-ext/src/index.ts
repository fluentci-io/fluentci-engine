import { dag } from "./plugin";

declare const Host: {
  inputString: () => string;
  outputString: (str: string) => void;
};

export function exec() {
  let command = Host.inputString();

  let stdout = dag
    .flox()
    .withWorkdir("./flox-demo")
    .withExec(command.split(" "))
    .stdout();

  Host.outputString(stdout);
}

export function git_tree() {
  // todo
}
