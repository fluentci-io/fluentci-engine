import { dag } from "./plugin";

declare const Host: {
  inputString: () => string;
  outputString: (str: string) => void;
};

export function exec() {
  const command = Host.inputString();

  const stdout = dag
    .flox()
    .withWorkdir("./flox-demo")
    .withExec(command.split(" "))
    .stdout();

  Host.outputString(stdout);
}

export function git_tree() {
  // todo
}
