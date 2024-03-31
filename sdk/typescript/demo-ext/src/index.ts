import { dag } from "./plugin";

declare const Host: {
  inputString: () => string;
  outputString: (str: string) => void;
};

export function exec() {
  const command = Host.inputString();

  const cacheId = dag.cache("flox").id()!;

  const stdout = dag
    .flox()
    .withWorkdir("./flox-demo")
    .withCache("./.flox", cacheId)
    .withExec(command.split(" "))
    .stdout();

  Host.outputString(stdout);
}

export function git_tree() {
  const url = Host.inputString();
  const entries = dag.git(url).branch("main").tree().entries();
  Host.outputString(JSON.stringify(entries, null, 2));
}
