import { Context, defaultContext } from "./context.ts";
import { computeQuery } from "./utils.ts";

/**
 * @hidden
 */
export type QueryTree = {
  operation: string;
  args?: Record<string, unknown>;
};

/**
 * @hidden
 */
export type Metadata = {
  [key: string]: {
    is_enum?: boolean;
  };
};

interface ClientConfig {
  queryTree?: QueryTree[];
  ctx?: Context;
}

export interface NixArgs {
  impure?: boolean;
}

class BaseClient {
  protected _queryTree: QueryTree[];
  protected _ctx: Context;

  /**
   * @hidden
   */
  constructor({ queryTree, ctx }: ClientConfig = {}) {
    this._queryTree = queryTree || [];
    this._ctx = ctx || new Context();
  }

  /**
   * @hidden
   */
  get queryTree(): QueryTree[] {
    return this._queryTree;
  }
}

export type BuildArg = {
  /**
   * The build argument name.
   */
  name: string;

  /**
   * The build argument value.
   */
  value: string;
};

/**
 * The root of the DAG.
 */
export class Client extends BaseClient {
  private readonly _checkVersionCompatibility?: boolean = undefined;

  constructor(
    parent?: { queryTree?: QueryTree[]; ctx: Context },
    _checkVersionCompatibility?: boolean
  ) {
    super(parent);

    this._checkVersionCompatibility = _checkVersionCompatibility;
  }

  pipeline = (name: string): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "pipeline",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };

  cache = (key: string): Cache => {
    return new Cache({
      queryTree: [
        ...this.queryTree,
        {
          operation: "cache",
          args: { key },
        },
      ],
      ctx: this._ctx,
    });
  };

  http = (url: string): File => {
    return new File({
      queryTree: [
        ...this.queryTree,
        {
          operation: "http",
          args: { url },
        },
      ],
      ctx: this._ctx,
    });
  };

  directory = (path: string): Directory => {
    return new Directory({
      queryTree: [
        ...this.queryTree,
        {
          operation: "directory",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  file = (path: string): File => {
    return new File({
      queryTree: [
        ...this.queryTree,
        {
          operation: "file",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  git = (url: string): Git => {
    return new Git({
      queryTree: [
        ...this.queryTree,
        {
          operation: "git",
          args: { url },
        },
      ],
      ctx: this._ctx,
    });
  };

  devbox = (): Devbox => {
    return new Devbox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "devbox",
        },
      ],
      ctx: this._ctx,
    });
  };

  devenv = (): Devenv => {
    return new Devenv({
      queryTree: [
        ...this.queryTree,
        {
          operation: "devenv",
        },
      ],
      ctx: this._ctx,
    });
  };

  flox = (): Flox => {
    return new Flox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "flox",
        },
      ],
      ctx: this._ctx,
    });
  };

  nix = (args?: NixArgs): Nix => {
    return new Nix({
      queryTree: [
        ...this.queryTree,
        {
          operation: "nix",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  pkgx = (): Pkgx => {
    return new Pkgx({
      queryTree: [
        ...this.queryTree,
        {
          operation: "pkgx",
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A directory whose contents persist across runs.
 */
export class Cache extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  key = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "key",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };
}

/**
 * A devbox environment.
 */
export class Devbox extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  withExec = (args: string[]): Devbox => {
    return new Devbox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Devbox => {
    return new Devbox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Devbox => {
    return new Devbox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Devbox => {
    return new Devbox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Devbox => {
    return new Devbox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  asService = (name: string): Service => {
    return new Service({
      queryTree: [
        ...this.queryTree,
        {
          operation: "asService",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A devenv environment.
 */
export class Devenv extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  withExec = (args: string[]): Devenv => {
    return new Devenv({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Devenv => {
    return new Devenv({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Devenv => {
    return new Devenv({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Devenv => {
    return new Devenv({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Devenv => {
    return new Devenv({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  asService = (name: string): Service => {
    return new Service({
      queryTree: [
        ...this.queryTree,
        {
          operation: "asService",
          args: {
            name,
          },
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A directory.
 */
export class Directory extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  directory = (path: string): Directory => {
    return new Directory({
      queryTree: [
        ...this.queryTree,
        {
          operation: "directory",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  entries = async (): Promise<string[]> => {
    const response: Awaited<string[]> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "entries",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  devbox = (): Devbox => {
    return new Devbox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "devbox",
        },
      ],
      ctx: this._ctx,
    });
  };

  devenv = (): Devenv => {
    return new Devenv({
      queryTree: [
        ...this.queryTree,
        {
          operation: "devenv",
        },
      ],
      ctx: this._ctx,
    });
  };

  flox = (): Flox => {
    return new Flox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "flox",
        },
      ],
      ctx: this._ctx,
    });
  };

  nix = (args?: NixArgs): Nix => {
    return new Nix({
      queryTree: [
        ...this.queryTree,
        {
          operation: "nix",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  pkgx = (): Pkgx => {
    return new Pkgx({
      queryTree: [
        ...this.queryTree,
        {
          operation: "pkgx",
        },
      ],
      ctx: this._ctx,
    });
  };

  pixi = (): Pixi => {
    return new Pixi({
      queryTree: [
        ...this.queryTree,
        {
          operation: "pixi",
        },
      ],
      ctx: this._ctx,
    });
  };

  mise = (): Mise => {
    return new Mise({
      queryTree: [
        ...this.queryTree,
        {
          operation: "mise",
        },
      ],
      ctx: this._ctx,
    });
  };

  envhub = (): Envhub => {
    return new Envhub({
      queryTree: [
        ...this.queryTree,
        {
          operation: "envhub",
        },
      ],
      ctx: this._ctx,
    });
  };

  withExec = (args: string[]): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  zip = (): File => {
    return new File({
      queryTree: [
        ...this.queryTree,
        {
          operation: "zip",
        },
      ],
      ctx: this._ctx,
    });
  };

  tarCzvf = (): File => {
    return new File({
      queryTree: [
        ...this.queryTree,
        {
          operation: "tarCzvf",
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A service.
 */
export class Service extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };
}

/**
 * A file.
 */
export class File extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  path = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "path",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  zip = (): File => {
    return new File({
      queryTree: [
        ...this.queryTree,
        {
          operation: "zip",
        },
      ],
      ctx: this._ctx,
    });
  };

  tarCzvf = (): File => {
    return new File({
      queryTree: [
        ...this.queryTree,
        {
          operation: "tarCzvf",
        },
      ],
      ctx: this._ctx,
    });
  };

  unzip = (outputDir?: String): Directory => {
    return new Directory({
      queryTree: [
        ...this.queryTree,
        {
          operation: "unzip",
          args: {
            outputDir,
          },
        },
      ],
      ctx: this._ctx,
    });
  };

  tarXzvf = (outputDir?: String): Directory => {
    return new Directory({
      queryTree: [
        ...this.queryTree,
        {
          operation: "tarXzvf",
          args: {
            outputDir,
          },
        },
      ],
      ctx: this._ctx,
    });
  };

  md5 = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "md5",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  sha256 = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "sha256",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  chmod = (mode: string): File => {
    return new File({
      queryTree: [
        ...this.queryTree,
        {
          operation: "chmod",
          args: { mode },
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A Flox environment.
 */
export class Flox extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  withExec = (args: string[]): Flox => {
    return new Flox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Flox => {
    return new Flox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Flox => {
    return new Flox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Flox => {
    return new Flox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Flox => {
    return new Flox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  asService = (name: string): Service => {
    return new Service({
      queryTree: [
        ...this.queryTree,
        {
          operation: "asService",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A Git repository.
 */
export class Git extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  branch = (name: string): Git => {
    return new Git({
      queryTree: [
        ...this.queryTree,
        {
          operation: "branch",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };

  commit = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "commit",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  tree = (): Directory => {
    return new Directory({
      queryTree: [
        ...this.queryTree,
        {
          operation: "tree",
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A Nix environment.
 */
export class Nix extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  withExec = (args: string[]): Nix => {
    return new Nix({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Nix => {
    return new Nix({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Nix => {
    return new Nix({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Nix => {
    return new Nix({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Nix => {
    return new Nix({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  asService = (name: string): Service => {
    return new Service({
      queryTree: [
        ...this.queryTree,
        {
          operation: "asService",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * Creates a named sub-pipeline
 */
export class Pipeline extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  http = (url: string): File => {
    return new File({
      queryTree: [
        ...this.queryTree,
        {
          operation: "http",
          args: { url },
        },
      ],
      ctx: this._ctx,
    });
  };

  git = (url: string): Git => {
    return new Git({
      queryTree: [
        ...this.queryTree,
        {
          operation: "git",
          args: { url },
        },
      ],
      ctx: this._ctx,
    });
  };

  devbox = (): Devbox => {
    return new Devbox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "devbox",
        },
      ],
      ctx: this._ctx,
    });
  };

  devenv = (): Devenv => {
    return new Devenv({
      queryTree: [
        ...this.queryTree,
        {
          operation: "devenv",
        },
      ],
      ctx: this._ctx,
    });
  };

  flox = (): Flox => {
    return new Flox({
      queryTree: [
        ...this.queryTree,
        {
          operation: "flox",
        },
      ],
      ctx: this._ctx,
    });
  };

  nix = (args?: NixArgs): Nix => {
    return new Nix({
      queryTree: [
        ...this.queryTree,
        {
          operation: "nix",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  pkgx = (): Pkgx => {
    return new Pkgx({
      queryTree: [
        ...this.queryTree,
        {
          operation: "pkgx",
        },
      ],
      ctx: this._ctx,
    });
  };

  pixi = (): Pixi => {
    return new Pixi({
      queryTree: [
        ...this.queryTree,
        {
          operation: "pixi",
        },
      ],
      ctx: this._ctx,
    });
  };

  mise = (): Mise => {
    return new Mise({
      queryTree: [
        ...this.queryTree,
        {
          operation: "mise",
        },
      ],
      ctx: this._ctx,
    });
  };

  envhub = (): Envhub => {
    return new Envhub({
      queryTree: [
        ...this.queryTree,
        {
          operation: "envhub",
        },
      ],
      ctx: this._ctx,
    });
  };

  withExec = (args: string[]): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Pipeline => {
    return new Pipeline({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  asService = (name: string): Service => {
    return new Service({
      queryTree: [
        ...this.queryTree,
        {
          operation: "asService",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A Pkgx environment.
 */
export class Pkgx extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  withExec = (args: string[]): Pkgx => {
    return new Pkgx({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Pkgx => {
    return new Pkgx({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Pkgx => {
    return new Pkgx({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Pkgx => {
    return new Pkgx({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Pkgx => {
    return new Pkgx({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withPackages = (packages: string[]): Pkgx => {
    return new Pkgx({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withPackages",
          args: { packages },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  asService = (name: string): Service => {
    return new Service({
      queryTree: [
        ...this.queryTree,
        {
          operation: "asService",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A Pixi environment.
 */
export class Pixi extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  withExec = (args: string[]): Pixi => {
    return new Pixi({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Pixi => {
    return new Pixi({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Pixi => {
    return new Pixi({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Pixi => {
    return new Pixi({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Pixi => {
    return new Pixi({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  asService = (name: string): Service => {
    return new Service({
      queryTree: [
        ...this.queryTree,
        {
          operation: "asService",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A Mise environment.
 */
export class Mise extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  withExec = (args: string[]): Mise => {
    return new Mise({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Mise => {
    return new Mise({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Mise => {
    return new Mise({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Mise => {
    return new Mise({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Mise => {
    return new Mise({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  asService = (name: string): Service => {
    return new Service({
      queryTree: [
        ...this.queryTree,
        {
          operation: "asService",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };
}

/**
 * A Envhub environment.
 */
export class Envhub extends BaseClient {
  private readonly _id?: string = undefined;

  constructor(parent?: { queryTree?: QueryTree[]; ctx: Context }) {
    super(parent);
  }

  id = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "id",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  use = (environment: string): Envhub => {
    return new Envhub({
      queryTree: [
        ...this.queryTree,
        {
          operation: "use",
          args: { environment },
        },
      ],
      ctx: this._ctx,
    });
  };

  withExec = (args: string[]): Envhub => {
    return new Envhub({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withExec",
          args: { args },
        },
      ],
      ctx: this._ctx,
    });
  };

  withWorkdir = (path: string): Envhub => {
    return new Envhub({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withWorkdir",
          args: { path },
        },
      ],
      ctx: this._ctx,
    });
  };

  withService = (serviceId: string): Envhub => {
    return new Envhub({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withService",
          args: { serviceId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withCache = (path: string, cacheId: string): Envhub => {
    return new Envhub({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withCache",
          args: { path, cacheId },
        },
      ],
      ctx: this._ctx,
    });
  };

  withFile = (path: string, fileId: string): Envhub => {
    return new Envhub({
      queryTree: [
        ...this.queryTree,
        {
          operation: "withFile",
          args: { path, fileId },
        },
      ],
      ctx: this._ctx,
    });
  };

  stdout = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stdout",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  stderr = async (): Promise<string> => {
    const response: Awaited<string> = await computeQuery(
      [
        ...this.queryTree,
        {
          operation: "stderr",
        },
      ],
      await this._ctx.connection()
    );
    return response;
  };

  asService = (name: string): Service => {
    return new Service({
      queryTree: [
        ...this.queryTree,
        {
          operation: "asService",
          args: { name },
        },
      ],
      ctx: this._ctx,
    });
  };
}

export const dag: Client = new Client({ ctx: defaultContext });
