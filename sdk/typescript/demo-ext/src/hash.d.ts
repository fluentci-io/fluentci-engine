// @ts-nocheck

declare module "main" {
  export function compute_md5(): I64;
  export function compute_sha256(): I64;
}

declare module "extism:host" {
  interface user {
    cache(ptr: I64): I64;
    directory(ptr: I64): I64;
    entries(ptr: I64): I64;
    devbox(): I64;
    devenv(): I64;
    flox(): I64;
    nix(): I64;
    pkgx(): I64;
    pixi(): I64;
    mise(): I64;
    envhub(): I64;
    tar_czvf(ptr: I64): I64;
    zip(ptr: I64): I64;
    with_exec(ptr: I64): void;
    with_workdir(ptr: I64): void;
    with_cache(ptr: I64): void;
    stdout(): I64;
    stderr(): I64;
    set_runner(ptr: I64): void;
    file(ptr: I64): I64;
    unzip(ptr: I64): I64;
    tar_xzvf(ptr: I64): I64;
    git(ptr: I64): I64;
    branch(ptr: I64): void;
    commit(): I64;
    tree(): I64;
    http(ptr: I64): I64;
    md5(ptr: I64): I64;
    sha256(ptr: I64): I64;
  }
}
