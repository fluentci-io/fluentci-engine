// @ts-nocheck

declare const Host: {
  getFunctions: () => {
    cache: (ptr: I64) => I64;
    directory: (ptr: I64) => I64;
    entries: (ptr: I64) => I64;
    devbox: () => I64;
    devenv: () => I64;
    flox: () => I64;
    nix: () => I64;
    pkgx: () => I64;
    pixi: () => I64;
    pipeline: (ptr: I64) => I64;
    mise: () => I64;
    envhub: () => I64;
    tar_czvf: (ptr: I64) => I64;
    zip: (ptr: I64) => I64;
    with_exec: (ptr: I64) => void;
    with_workdir: (ptr: I64) => void;
    withCache: (ptr: I64) => I64;
    stdout: () => I64;
    stderr: () => I64;
    set_runner: (ptr: I64) => void;
    file: (ptr: I64) => I64;
    unzip: (ptr: I64) => I64;
    tar_xzvf: (ptr: I64) => I64;
    git: (ptr: I64) => I64;
    branch: (ptr: I64) => void;
    commit: () => I64;
    tree: () => I64;
    http: (ptr: I64) => I64;
    md5: (ptr: I64) => I64;
    sha256: (ptr: I64) => I64;
  };
};

declare const Memory: {
  fromString: (str: string) => I64;
  fromJsonObject: (obj: any) => I64;
  find: (offset: I64) => {
    readString: () => string;
    readJsonObject: () => Record<string, any>;
  };
};

export const {
  cache,
  http,
  file,
  directory,
  entries,
  devbox,
  devenv,
  flox,
  nix,
  pkgx,
  pipeline,
  pixi,
  mise,
  envhub,
  tar_czvf,
  zip,
  with_exec,
  with_workdir,
  with_cache,
  stdout,
  stderr,
  set_runner,
  git,
  branch,
  commit,
  tree,
  md5,
  sha256,
  unzip,
  tar_xzvf,
} = Host.getFunctions();

class BaseClient {
  constructor() {}
}

export class Client extends BaseClient {
  pipeline = (name: string): Pipeline => {
    let mem = Memory.fromString(name);
    let offset = pipeline(mem.offset);
    let response = Memory.find(offset).readJsonObject();

    return new Pipeline({
      id: response.id,
    });
  };

  cache = (key: string): Cache => {
    let mem = Memory.fromString(key);
    let offset = cache(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new Cache({
      id: response.id,
      key: response.key,
    });
  };

  devbox = (): Devbox => {
    let offset = devbox();
    let response = Memory.find(offset).readJsonObject();
    return new Devbox({
      id: response.id,
    });
  };

  directory = (path: string): Directory => {
    let mem = Memory.fromString(path);
    let offset = directory(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };

  envhub = (): Envhub => {
    let offset = envhub();
    let response = Memory.find(offset).readJsonObject();
    return new Envhub({
      id: response.id,
    });
  };

  flox = (): Flox => {
    let offset = flox();
    let response = Memory.find(offset).readJsonObject();
    return new Flox({
      id: response.id,
    });
  };

  git = (url: string): Git => {
    let mem = Memory.fromString(url);
    let offset = git(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new Git({
      id: response.id,
    });
  };

  file = (path: string): File => {
    let mem = Memory.fromString(path);
    let offset = file(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  nix = (): Nix => {
    let offset = nix();
    let response = Memory.find(offset).readJsonObject();
    return new Nix({
      id: response.id,
    });
  };

  http = (url: string): File => {
    let mem = Memory.fromString(url);
    let offset = http(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  mise = (): Mise => {
    let offset = mise();
    let response = Memory.find(offset).readJsonObject();
    return new Mise({
      id: response.id,
    });
  };

  pixi = (): Pixi => {
    let offset = pixi();
    let response = Memory.find(offset).readJsonObject();
    return new Pixi({
      id: response.id,
    });
  };

  pkgx = (): Pkgx => {
    let offset = pkgx();
    let response = Memory.find(offset).readJsonObject();
    return new Pkgx({
      id: response.id,
    });
  };
}

export class Cache extends BaseClient {
  private _id?: string;
  private _key?: string;

  constructor({ id, key }: { id: string; key: string }) {
    super();
    this._id = id;
    this._key = key;
  }

  id = () => {
    return this._id;
  };

  key = () => {
    return this._key;
  };
}

export class Devbox extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  withExec = (args: string[]): Devbox => {
    let mem = Memory.fromString("devbox");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Devbox => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Devbox => {
    // todo
    return this;
  };

  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export class Devenv extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  withExec = (args: string[]): Devenv => {
    let mem = Memory.fromString("devenv");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Devenv => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Devenv => {
    // todo
    return this;
  };

  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export class Directory extends BaseClient {
  private _id?: string;
  private _path?: string;

  constructor({ id, path }: { id: string; path: string }) {
    super();
    this._id = id;
    this._path = path;
  }

  id = () => {
    return this._id;
  };

  path = () => {
    return this._path;
  };

  directory = (path: string): Directory => {
    let mem = Memory.fromString(path);
    let offset = directory(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };

  entries = (): string[] => {
    const mem = Memory.fromString(this._path);
    const offset = entries(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return response;
  };

  devbox = (): Devbox => {
    const offset = devbox();
    const response = Memory.find(offset).readJsonObject();
    return new Devbox({
      id: response.id,
    });
  };

  devenv = (): Devenv => {
    const offset = devenv();
    const response = Memory.find(offset).readJsonObject();
    return new Devenv({
      id: response.id,
    });
  };

  flox = (): Flox => {
    const offset = flox();
    const response = Memory.find(offset).readJsonObject();
    return new Flox({
      id: response.id,
    });
  };

  nix = (): Nix => {
    const offset = nix();
    const response = Memory.find(offset).readJsonObject();
    return new Nix({
      id: response.id,
    });
  };

  pkgx = (): Pkgx => {
    const offset = pkgx();
    const response = Memory.find(offset).readJsonObject();
    return new Pkgx({
      id: response.id,
    });
  };

  pixi = (): Pixi => {
    const offset = pixi();
    const response = Memory.find(offset).readJsonObject();
    return new Pixi({
      id: response.id,
    });
  };

  mise = (): Mise => {
    const offset = mise();
    const response = Memory.find(offset).readJsonObject();
    return new Mise({
      id: response.id,
    });
  };

  envhub = (): Envhub => {
    const offset = envhub();
    const response = Memory.find(offset).readJsonObject();
    return new Envhub({
      id: response.id,
    });
  };

  tarCzvf = (path: string): File => {
    const mem = Memory.fromString(path);
    const offset = tar_czvf(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  zip = (path: string): File => {
    const mem = Memory.fromString(path);
    const offset = zip(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  withExec = (args: string[]): Directory => {
    const mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Directory => {
    const mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Directory => {
    // todo
    return this;
  };

  stdout = (): string => {
    const offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    const offset = stderr();
    return Memory.find(offset).readString();
  };
}

export class Service extends BaseClient {
  private _id?: string;

  id = () => {
    return this._id;
  };
}

export class File extends BaseClient {
  private _id?: string;
  private _path?: string;

  constructor({ id, path }: { id: string; path: string }) {
    super();
    this._id = id;
    this._path = path;
  }

  id = () => {
    return this._id;
  };

  path = () => {
    return this._path;
  };

  zip = (): File => {
    const mem = Memory.fromString(this._path);
    const offset = zip(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  unzip = (): Directory => {
    const mem = Memory.fromString(this._path);
    const offset = unzip(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };

  tarXzvf = (): Directory => {
    const mem = Memory.fromString(this._path);
    const offset = tar_xzvf(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };

  md5 = (): string => {
    const mem = Memory.fromString(this._path);
    const offset = md5(mem.offset);
    return Memory.find(offset).readString();
  };

  sha256 = (): string => {
    const mem = Memory.fromString(this._path);
    const offset = sha256(mem.offset);
    return Memory.find(offset).readString();
  };
}

export class Flox extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  withExec = (args: string[]): Flox => {
    let mem = Memory.fromString("flox");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Flox => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Flox => {
    // todo
    return this;
  };

  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export class Git extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  branch = (name: string): Git => {
    let mem = Memory.fromString(name);
    branch(mem.offset);
    return this;
  };

  commit = (): string => {
    let offset = commit();
    return Memory.find(offset).readString();
  };

  tree = (): Directory => {
    let offset = tree();
    let response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };
}

export class Nix extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  withExec = (args: string[]): Nix => {
    let mem = Memory.fromString("nix");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Nix => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Nix => {
    // todo
    return this;
  };

  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export class Pipeline extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  devbox = (): Devbox => {
    let offset = devbox();
    let response = Memory.find(offset).readJsonObject();
    return new Devbox({
      id: response.id,
    });
  };

  devenv = (): Devenv => {
    let offset = devenv();
    let response = Memory.find(offset).readJsonObject();
    return new Devenv({
      id: response.id,
    });
  };

  flox = (): Flox => {
    let offset = flox();
    let response = Memory.find(offset).readJsonObject();
    return new Flox({
      id: response.id,
    });
  };

  nix = (): Nix => {
    let offset = nix();
    let response = Memory.find(offset).readJsonObject();
    return new Nix({
      id: response.id,
    });
  };

  pkgx = (): Pkgx => {
    let offset = pkgx();
    let response = Memory.find(offset).readJsonObject();
    return new Pkgx({
      id: response.id,
    });
  };

  pixi = (): Pixi => {
    let offset = pixi();
    let response = Memory.find(offset).readJsonObject();
    return new Pixi({
      id: response.id,
    });
  };

  mise = (): Mise => {
    let offset = mise();
    let response = Memory.find(offset).readJsonObject();
    return new Mise({
      id: response.id,
    });
  };

  envhub = (): Envhub => {
    let offset = envhub();
    let response = Memory.find(offset).readJsonObject();
    return new Envhub({
      id: response.id,
    });
  };

  withExec = (args: string[]): Pipeline => {
    let mem = Memory.fromString("default");
    set_runner(mem.offset);
    mem = Memory.fromString(args.join(" "));
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Pipeline => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Pipeline => {
    // todo
    return this;
  };

  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export class Pkgx extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  withExec = (args: string[]): Pkgx => {
    let mem = Memory.fromString("pkgx");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Pkgx => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Pkgx => {
    // todo
    return this;
  };

  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export class Pixi extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  withExec = (args: string[]): Pixi => {
    let mem = Memory.fromString("pixi");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Pixi => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Pixi => {
    // todo
    return this;
  };

  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export class Mise extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  withExec = (args: string[]): Mise => {
    let mem = Memory.fromString("mise");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Mise => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Mise => {
    // todo
    return this;
  };

  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export class Envhub extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  id = () => {
    return this._id;
  };

  withExec = (args: string[]): Envhub => {
    let mem = Memory.fromString("envhub");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  withWorkdir = (path: string): Envhub => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  withCache = (path: string, cacheId: String): Envhub => {
    // todo
    return this;
  };

  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export const dag = new Client();
