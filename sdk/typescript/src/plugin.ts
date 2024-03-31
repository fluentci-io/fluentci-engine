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

/**
 * Root of the DAG client.
 */
export class Client extends BaseClient {
  /**
   * Creates a named sub-pipeline.
   *
   * ```ts
   *  dag.pipeline("my-pipeline").withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string} name Name of the sub-pipeline
   * @returns {Pipeline}
   */
  pipeline = (name: string): Pipeline => {
    let mem = Memory.fromString(name);
    let offset = pipeline(mem.offset);
    let response = Memory.find(offset).readJsonObject();

    return new Pipeline({
      id: response.id,
    });
  };

  /**
   * Adds a cache
   *
   * ```ts
   * dag.cache("my-cache").key();
   * ```
   *
   * @param {string} key Unique key for the cache
   * @returns {Cache}
   */
  cache = (key: string): Cache => {
    let mem = Memory.fromString(key);
    let offset = cache(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new Cache({
      id: response.id,
      key: response.key,
    });
  };

  /**
   * Setup a new devbox environment
   *
   * ```ts
   * dag.devbox().withExec(["echo", "hello world"]).stdout();
   * ```
   *
   * @returns {Devbox}
   */
  devbox = (): Devbox => {
    let offset = devbox();
    let response = Memory.find(offset).readJsonObject();
    return new Devbox({
      id: response.id,
    });
  };

  /**
   * Returns a Directory object at the given path
   *
   * ```ts
   * dag.directory("/path/to/dir").entries();
   * ```
   *
   * @param path
   * @returns
   */
  directory = (path: string): Directory => {
    let mem = Memory.fromString(path);
    let offset = directory(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };

  /**
   * Setup a new envhub environment
   *
   * ```ts
   * dag.envhub().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns
   */
  envhub = (): Envhub => {
    let offset = envhub();
    let response = Memory.find(offset).readJsonObject();
    return new Envhub({
      id: response.id,
    });
  };

  /**
   * Setup a new flox environment
   *
   * ```ts
   * dag.flox().withExec(["echo", "hello world"]).stdout();
   * ```
   *
   * @returns
   */
  flox = (): Flox => {
    let offset = flox();
    let response = Memory.find(offset).readJsonObject();
    return new Flox({
      id: response.id,
    });
  };

  /**
   * Return a Git Repository from the given URL
   * ```ts
   * dag.git("https://github.com/tsirysndr/me").branch("main").commit();
   * ```
   * @param url
   * @returns
   */
  git = (url: string): Git => {
    let mem = Memory.fromString(url);
    let offset = git(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new Git({
      id: response.id,
    });
  };

  /**
   * Return a File object from the given path
   * ts```
   * dag.file("/path/to/file");
   * ```
   * @param path
   * @returns
   */
  file = (path: string): File => {
    let mem = Memory.fromString(path);
    let offset = file(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  /**
   * Setup a new nix environment
   * ```ts
   * dag.nix().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns
   */
  nix = (): Nix => {
    let offset = nix();
    let response = Memory.find(offset).readJsonObject();
    return new Nix({
      id: response.id,
    });
  };

  /**
   * Get a File object from the given URL
   * ```ts
   * dag.http("https://example.com/file.txt");
   * ```
   * @param url
   * @returns
   */
  http = (url: string): File => {
    let mem = Memory.fromString(url);
    let offset = http(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  /**
   * Setup a new mise environment
   * ```ts
   * dag.mise().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns
   */
  mise = (): Mise => {
    let offset = mise();
    let response = Memory.find(offset).readJsonObject();
    return new Mise({
      id: response.id,
    });
  };

  /**
   * Setup a new pixi environment
   * ```ts
   * dag.pixi().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns
   */
  pixi = (): Pixi => {
    let offset = pixi();
    let response = Memory.find(offset).readJsonObject();
    return new Pixi({
      id: response.id,
    });
  };

  /**
   * Setup a new pkgx environment
   * ```ts
   * dag.pkgx().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns
   */
  pkgx = (): Pkgx => {
    let offset = pkgx();
    let response = Memory.find(offset).readJsonObject();
    return new Pkgx({
      id: response.id,
    });
  };
}

/**
 * Represents a Cache
 */
export class Cache extends BaseClient {
  private _id?: string;
  private _key?: string;

  constructor({ id, key }: { id: string; key: string }) {
    super();
    this._id = id;
    this._key = key;
  }

  /**
   * Returns the cache id
   * ```ts
   * dag.cache("my-cache").id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Returns the cache key
   * ```ts
   * dag.cache("my-cache").key();
   * ```
   * @returns {string}
   */
  key = (): string => {
    return this._key;
  };
}

/**
 * Represents an Devbox environment
 */
export class Devbox extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the devbox id
   * ```ts
   * dag.devbox().id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Setup a new exec command
   * ```ts
   * dag.devbox().withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Devbox}
   */
  withExec = (args: string[]): Devbox => {
    let mem = Memory.fromString("devbox");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.devbox().withWorkdir("/path/to/dir");
   * ```
   * @param {string} path Path to the new working directory
   * @returns {Devbox}
   */
  withWorkdir = (path: string): Devbox => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.devbox().withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Devbox} Devbox
   */
  withCache = (path: string, cacheId: String): Devbox => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   * ```ts
   * dag.devbox().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {string}
   */
  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   * dag.devbox().withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a Devenv environment
 */
export class Devenv extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the devenv id
   * ```ts
   * dag.devenv().id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Setup a new exec command
   * ```ts
   * dag.devenv().withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Devenv}
   */
  withExec = (args: string[]): Devenv => {
    let mem = Memory.fromString("devenv");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.devenv().withWorkdir("/path/to/dir");
   * ```
   * @param {string} path Path to the new working directory
   * @returns {Devenv}
   */
  withWorkdir = (path: string): Devenv => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.devenv().withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Devenv} Devenv
   */
  withCache = (path: string, cacheId: String): Devenv => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   *
   * ```ts
   *  dag.devenv().withExec(["echo", "hello world"]).stdout();
   * ```
   *
   * @returns {string}
   */
  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   *  dag.devenv().withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a Directory
 */
export class Directory extends BaseClient {
  private _id?: string;
  private _path?: string;

  constructor({ id, path }: { id: string; path: string }) {
    super();
    this._id = id;
    this._path = path;
  }

  /**
   * Returns the directory id
   * ```ts
   * dag.directory("/path/to/dir").id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Returns the directory path
   * ```ts
   * dag.directory("/path/to/dir").path();
   * ```
   * @returns {string}
   */
  path = (): string => {
    return this._path;
  };

  /**
   * Returns a Directory object with the given path
   * ```ts
   * dag.directory("/path/to/dir").directory("sub-dir").entries();
   * ```
   * @param {string} path Path to the directory
   * @returns {Directory}
   */
  directory = (path: string): Directory => {
    let mem = Memory.fromString(path);
    let offset = directory(mem.offset);
    let response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };

  /**
   * Returns the list of entries in the directory
   * ```ts
   *  dag.directory("/path/to/dir").entries();
   * ```
   * @returns {string[]}
   */
  entries = (): string[] => {
    const mem = Memory.fromString(this._path);
    const offset = entries(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return response;
  };

  /**
   * Setup a devbox environment
   * ```ts
   * dag.directory("/path/to/dir").devbox().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Devbox}
   */
  devbox = (): Devbox => {
    const offset = devbox();
    const response = Memory.find(offset).readJsonObject();
    return new Devbox({
      id: response.id,
    });
  };

  /**
   * Setup a devenv environment
   * ```ts
   * dag.directory("/path/to/dir").devenv().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Devenv}
   */
  devenv = (): Devenv => {
    const offset = devenv();
    const response = Memory.find(offset).readJsonObject();
    return new Devenv({
      id: response.id,
    });
  };

  /**
   * Setup a flox environment
   * ```ts
   * dag.directory("/path/to/dir").flox().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Flox}
   */
  flox = (): Flox => {
    const offset = flox();
    const response = Memory.find(offset).readJsonObject();
    return new Flox({
      id: response.id,
    });
  };

  /**
   * Setup a nix environment
   * ```ts
   * dag.directory("/path/to/dir").nix().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Nix}
   */
  nix = (): Nix => {
    const offset = nix();
    const response = Memory.find(offset).readJsonObject();
    return new Nix({
      id: response.id,
    });
  };

  /**
   * Setup a pkgx environment
   * ```ts
   * dag.directory("/path/to/dir").pkgx().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Pkgx}
   */
  pkgx = (): Pkgx => {
    const offset = pkgx();
    const response = Memory.find(offset).readJsonObject();
    return new Pkgx({
      id: response.id,
    });
  };

  /**
   * Setup a pixi environment
   * ```ts
   * dag.directory("/path/to/dir").pixi().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Pixi}
   */
  pixi = (): Pixi => {
    const offset = pixi();
    const response = Memory.find(offset).readJsonObject();
    return new Pixi({
      id: response.id,
    });
  };

  /**
   * Setup a mise environment
   *
   * ```ts
   * dag.directory("/path/to/dir").mise().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Mise}
   */
  mise = (): Mise => {
    const offset = mise();
    const response = Memory.find(offset).readJsonObject();
    return new Mise({
      id: response.id,
    });
  };

  /**
   * Setup a envhub environment
   * ```ts
   * dag.directory("/path/to/dir").envhub().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Envhub}
   */
  envhub = (): Envhub => {
    const offset = envhub();
    const response = Memory.find(offset).readJsonObject();
    return new Envhub({
      id: response.id,
    });
  };

  /**
   * Creates a new tar archive from the directory
   * ```ts
   * dag.directory("/path/to/dir").tarCzvf();
   * ```
   * @returns {File}
   */
  tarCzvf = (): File => {
    const mem = Memory.fromString(this._path);
    const offset = tar_czvf(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  /**
   * Creates a new zip archive from the directory
   * ```ts
   * dag.directory("/path/to/dir").zip();
   * ```
   * @returns {File}
   */
  zip = (): File => {
    const mem = Memory.fromString(this._path);
    const offset = zip(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  /**
   * Execute a command in the directory
   * ```ts
   * dag.directory("/path/to/dir").withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Directory}
   */
  withExec = (args: string[]): Directory => {
    const mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.directory("/path/to/dir").withWorkdir("/path/to/new/dir").stdout();
   * ```
   * @param {string} path Path to the new working directory
   */
  withWorkdir = (path: string): Directory => {
    const mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.directory("/path/to/dir").withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Directory}
   */
  withCache = (path: string, cacheId: String): Directory => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   * ```ts
   * dag.directory("/path/to/dir").withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {string}
   */
  stdout = (): string => {
    const offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   * dag.directory("/path/to/dir").withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    const offset = stderr();
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a Service
 */
export class Service extends BaseClient {
  private _id?: string;

  id = () => {
    return this._id;
  };
}

/**
 * Represents a File
 */
export class File extends BaseClient {
  private _id?: string;
  private _path?: string;

  constructor({ id, path }: { id: string; path: string }) {
    super();
    this._id = id;
    this._path = path;
  }

  /**
   * Returns the file id
   * ```ts
   * dag.file("/path/to/file").id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Returns the file path
   * ```ts
   * dag.file("/path/to/file").path();
   * ```
   * @returns {string}
   */
  path = (): string => {
    return this._path;
  };

  /**
   * Creates a new zip archive from the file
   * ```ts
   * dag.file("/path/to/file").zip();
   * ```
   * @returns {File}
   */
  zip = (): File => {
    const mem = Memory.fromString(this._path);
    const offset = zip(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new File({
      id: response.id,
      path: response.path,
    });
  };

  /**
   * Decompress the zip file into a directory
   * ```ts
   * dag.file("/path/to/file.zip").unzip();
   * ```
   * @returns {Directory}
   */
  unzip = (): Directory => {
    const mem = Memory.fromString(this._path);
    const offset = unzip(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };

  /**
   * Decompress the tar file into a directory
   * ```ts
   * dag.file("/path/to/file.tar.gz").tarXzvf();
   * ```
   * @returns {Directory}
   */
  tarXzvf = (): Directory => {
    const mem = Memory.fromString(this._path);
    const offset = tar_xzvf(mem.offset);
    const response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };

  /**
   * Returns the md5 hash of the file
   * ```ts
   * dag.file("/path/to/file").md5();
   * ```
   * @returns {string}
   */
  md5 = (): string => {
    const mem = Memory.fromString(this._path);
    const offset = md5(mem.offset);
    return Memory.find(offset).readString();
  };

  /**
   * Returns the sha256 hash of the file
   * ```ts
   * dag.file("/path/to/file").sha256();
   * ```
   * @returns {string}
   */
  sha256 = (): string => {
    const mem = Memory.fromString(this._path);
    const offset = sha256(mem.offset);
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a Flox environment
 */
export class Flox extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the flox id
   * ```ts
   * dag.flox().id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Execute a command in the flox environment
   * ```ts
   * dag.flox().withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Flox}
   */
  withExec = (args: string[]): Flox => {
    let mem = Memory.fromString("flox");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.flox().withWorkdir("/path/to/dir");
   * ```
   * @param {string} path Path to the new working directory
   * @returns {Flox}
   */
  withWorkdir = (path: string): Flox => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.flox().withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Flox}
   */
  withCache = (path: string, cacheId: String): Flox => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
      key: "",
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   * ```ts
   * dag.flox().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {string}
   */
  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   * dag.flox().withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a Git Repository
 */
export class Git extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the git id
   * ```ts
   * dag.git("https://github.com/tsirysndr/me").id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Switch to a new branch
   * ```ts
   * dag.git("https://github.com/tsirysndr/me").branch("main");
   * ```
   * @param {string} name Name of the branch
   * @returns {Git}
   */
  branch = (name: string): Git => {
    let mem = Memory.fromString(name);
    branch(mem.offset);
    return this;
  };

  /**
   * Returns the last commit hash
   * ```ts
   * dag.git("https://github.com/tsirysndr/me").commit();
   * ```
   * @returns {string}
   */
  commit = (): string => {
    let offset = commit();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the tree of the repository
   * ```ts
   * dag.git("https://github.com/tsirysndr/me").tree();
   * ```
   * @returns {Directory}
   */
  tree = (): Directory => {
    let offset = tree();
    let response = Memory.find(offset).readJsonObject();
    return new Directory({
      id: response.id,
      path: response.path,
    });
  };
}

/**
 * Represents a Nix environment
 */
export class Nix extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the Nix id
   * ```ts
   * dag.nix().id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Execute a command in the Nix environment
   * ```ts
   * dag.nix().withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Nix}
   */
  withExec = (args: string[]): Nix => {
    let mem = Memory.fromString("nix");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.nix().withWorkdir("/path/to/dir");
   * ```
   * @param {string} path Path to the new working directory
   * @returns {Nix}
   */
  withWorkdir = (path: string): Nix => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.nix().withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Nix}
   */
  withCache = (path: string, cacheId: String): Nix => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   * ```ts
   * dag.nix().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {string}
   */
  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   * dag.nix().withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a named sub-pipeline
 */
export class Pipeline extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the pipeline id
   * ```ts
   * dag.pipeline("my-pipeline").id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Setup a new exec command
   * ```ts
   * dag.pipeline("my-pipeline").withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Devbox}
   */
  devbox = (): Devbox => {
    let offset = devbox();
    let response = Memory.find(offset).readJsonObject();
    return new Devbox({
      id: response.id,
    });
  };

  /**
   * Setup a new devenv environment
   * ```ts
   * dag.pipeline("my-pipeline").devenv().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Devenv}
   */
  devenv = (): Devenv => {
    let offset = devenv();
    let response = Memory.find(offset).readJsonObject();
    return new Devenv({
      id: response.id,
    });
  };

  /**
   * Setup a new flox environment
   * ```ts
   * dag.pipeline("my-pipeline").flox().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Flox}
   */
  flox = (): Flox => {
    let offset = flox();
    let response = Memory.find(offset).readJsonObject();
    return new Flox({
      id: response.id,
    });
  };

  /**
   * Setup a new nix environment
   * ```ts
   * dag.pipeline("my-pipeline").nix().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Nix}
   */
  nix = (): Nix => {
    let offset = nix();
    let response = Memory.find(offset).readJsonObject();
    return new Nix({
      id: response.id,
    });
  };

  /**
   * Setup a new pkgx environment
   * ```ts
   * dag.pipeline("my-pipeline").pkgx().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Pkgx}
   */
  pkgx = (): Pkgx => {
    let offset = pkgx();
    let response = Memory.find(offset).readJsonObject();
    return new Pkgx({
      id: response.id,
    });
  };

  /**
   * Setup a new pixi environment
   * ```ts
   * dag.pipeline("my-pipeline").pixi().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Pixi}
   */
  pixi = (): Pixi => {
    let offset = pixi();
    let response = Memory.find(offset).readJsonObject();
    return new Pixi({
      id: response.id,
    });
  };

  /**
   * Setup a new mise environment
   * ```ts
   * dag.pipeline("my-pipeline").mise().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Mise}
   */
  mise = (): Mise => {
    let offset = mise();
    let response = Memory.find(offset).readJsonObject();
    return new Mise({
      id: response.id,
    });
  };

  /**
   * Setup a new envhub environment
   * ```ts
   * dag.pipeline("my-pipeline").envhub().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {Envhub}
   */
  envhub = (): Envhub => {
    let offset = envhub();
    let response = Memory.find(offset).readJsonObject();
    return new Envhub({
      id: response.id,
    });
  };

  /**
   * Execute a command in the pipeline
   * ```ts
   * dag.pipeline("my-pipeline").withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Pipeline}
   */
  withExec = (args: string[]): Pipeline => {
    let mem = Memory.fromString("default");
    set_runner(mem.offset);
    mem = Memory.fromString(args.join(" "));
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.pipeline("my-pipeline").withWorkdir("/path/to/dir");
   * ```
   * @param {string} path Path to the new working directory
   * @returns {Pipeline}
   */
  withWorkdir = (path: string): Pipeline => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.pipeline("my-pipeline").withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Pipeline}
   */
  withCache = (path: string, cacheId: String): Pipeline => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   * ```ts
   * dag.pipeline("my-pipeline").withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {string}
   */
  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   * dag.pipeline("my-pipeline").withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a Pkgx environment
 */
export class Pkgx extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the Pkgx id
   * ```ts
   * dag.pkgx().id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Execute a command in the Pkgx environment
   * ```ts
   * dag.pkgx().withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Pkgx}
   */
  withExec = (args: string[]): Pkgx => {
    let mem = Memory.fromString("pkgx");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.pkgx().withWorkdir("/path/to/dir");
   * ```
   * @param {string} path Path to the new working directory
   * @returns {Pkgx}
   */
  withWorkdir = (path: string): Pkgx => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.pkgx().withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Pkgx}
   */
  withCache = (path: string, cacheId: String): Pkgx => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   * ```ts
   * dag.pkgx().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {string}
   */
  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   * dag.pkgx().withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a Pixi environment
 */
export class Pixi extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the Pixi id
   * ```ts
   * dag.pixi().id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Execute a command in the Pixi environment
   * ```ts
   * dag.pixi().withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Pixi}
   */
  withExec = (args: string[]): Pixi => {
    let mem = Memory.fromString("pixi");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.pixi().withWorkdir("/path/to/dir");
   * ```
   * @param {string} path Path to the new working directory
   * @returns {Pixi}
   */
  withWorkdir = (path: string): Pixi => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.pixi().withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Pixi}
   */
  withCache = (path: string, cacheId: String): Pixi => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   * ```ts
   * dag.pixi().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {string}
   */
  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   * dag.pixi().withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a Mise environment
 */
export class Mise extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the Mise id
   * ```ts
   * dag.mise().id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Execute a command in the Mise environment
   * ```ts
   * dag.mise().withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Mise}
   */
  withExec = (args: string[]): Mise => {
    let mem = Memory.fromString("mise");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.mise().withWorkdir("/path/to/dir");
   * ```
   * @param {string} path Path to the new working directory
   * @returns {Mise}
   */
  withWorkdir = (path: string): Mise => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.mise().withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Mise}
   */
  withCache = (path: string, cacheId: String): Mise => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   * ```ts
   * dag.mise().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {string}
   */
  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   * dag.mise().withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

/**
 * Represents a Envhub environment
 */
export class Envhub extends BaseClient {
  private _id?: string;

  constructor({ id }: { id: string }) {
    super();
    this._id = id;
  }

  /**
   * Returns the Envhub id
   * ```ts
   * dag.envhub().id();
   * ```
   * @returns {string}
   */
  id = (): string => {
    return this._id;
  };

  /**
   * Execute a command in the Envhub environment
   * ```ts
   * dag.envhub().withExec(["echo", "hello world"]).stdout();
   * ```
   * @param {string[]} args
   * @returns {Envhub}
   */
  withExec = (args: string[]): Envhub => {
    let mem = Memory.fromString("envhub");
    set_runner(mem.offset);
    mem = Memory.fromJsonObject(args);
    with_exec(mem.offset);
    return this;
  };

  /**
   * Change the working directory
   * ```ts
   * dag.envhub().withWorkdir("/path/to/dir");
   * ```
   * @param {string} path Path to the new working directory
   * @returns {Envhub}
   */
  withWorkdir = (path: string): Envhub => {
    let mem = Memory.fromString(path);
    with_workdir(mem.offset);
    return this;
  };

  /**
   * Setup a new cache
   * ```ts
   * dag.envhub().withCache("/path/to/dir", "cache-id");
   * ```
   * @param {string} path Path to the cache
   * @param {String} cacheId Unique cache identifier
   * @returns {Envhub}
   */
  withCache = (path: string, cacheId: String): Envhub => {
    let mem = Memory.fromJsonObject({
      path,
      id: cacheId,
    });
    with_cache(mem.offset);
    return this;
  };

  /**
   * Returns the stdout of the last executed command
   * ```ts
   * dag.envhub().withExec(["echo", "hello world"]).stdout();
   * ```
   * @returns {string}
   */
  stdout = (): string => {
    let offset = stdout();
    return Memory.find(offset).readString();
  };

  /**
   * Returns the stderr of the last executed command
   * ```ts
   * dag.envhub().withExec(["echo", "hello world"]).stderr();
   * ```
   * @returns {string}
   */
  stderr = (): string => {
    let offset = stderr();
    return Memory.find(offset).readString();
  };
}

export const dag = new Client();