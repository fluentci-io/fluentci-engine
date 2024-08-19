module Cache = {
  type t

  @send
  external id: t => string = "id"

  @send
  external key: t => string = "key"
}

module NixArgs = {
  type t = {impure?: bool}
}

module Devbox = {
  type t

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Devenv = {
  type t

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Envhub = {
  type t

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Flox = {
  type t

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Mise = {
  type t

  @send
  external id: t => string = "id"

  @send
  external trust: t => t = "trust"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Nix = {
  type t

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Pixi = {
  type t

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Pkgx = {
  type t

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Proto = {
  type t

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Hermite = {
  type t

  @send
  external id: t => string = "id"

  @send
  external install: t => t = "install"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Secret = {
  type t

  @send
  external id: t => string = "id"

  @send
  external mount: t => string = "mount"

  @send
  external name: t => string = "name"

  @send
  external plaintext: t => string = "plaintext"
}

module SecretManager = {
  type t

  @send
  external getSecret: (t, ~name: string) => Belt.Array.t<Secret.t> = "getSecret"

  @send
  external id: t => string = "id"
}

module Service = {
  type t

  @send
  external id: t => string = "id"
}

module Pipeline = {
  type t

  @send
  external devbox: t => Devbox.t = "devbox"

  @send
  external devenv: t => Devenv.t = "devenv"

  @send
  external envhub: t => Envhub.t = "envhub"

  @send
  external flox: t => Flox.t = "flox"

  @send
  external mise: t => Mise.t = "mise"

  @send
  external nix: (t, ~args: NixArgs.t=?) => Nix.t = "nix"

  @send
  external pixi: t => Pixi.t = "pixi"

  @send
  external pkgx: t => Pkgx.t = "pkgx"

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Directory = {
  type t

  module File = {
    type f

    @send
    external id: f => string = "id"

    @send
    external chmod: (f, ~mode: string) => f = "chmod"

    @send
    external md5: f => string = "md5"

    @send
    external path: f => string = "path"

    @send
    external sha256: f => string = "sha256"

    @send
    external tarXzvf: f => t = "tarXzvf"

    @send
    external unzip: f => t = "unzip"

    @send
    external zip: f => f = "zip"
  }

  @send
  external id: t => string = "id"

  @send
  external asService: (t, ~name: string) => string = "asService"

  @send
  external devbox: t => Devbox.t = "devbox"

  @send
  external devenv: t => Devenv.t = "devenv"

  @send
  external directory: (t, ~path: string) => t = "directory"

  @send
  external entries: t => Belt.Array.t<string> = "entries"

  @send
  external envhub: t => Envhub.t = "envhub"

  @send
  external flox: t => Flox.t = "flox"

  @send
  external mise: t => Mise.t = "mise"

  @send
  external nix: (t, ~args: NixArgs.t=?) => Nix.t = "nix"

  @send
  external path: t => string = "path"

  @send
  external pixi: t => Pixi.t = "pixi"

  @send
  external pkgx: t => Pkgx.t = "pkgx"

  @send
  external stderr: t => string = "stderr"

  @send
  external stdout: t => string = "stdout"

  @send
  external tarCzvf: t => File.f = "tarCzvf"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cacheID: string) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, ~args: Belt.Array.t<string>) => t = "withExec"

  @send
  external withSecretVariable: (t, ~name: string, ~secretID: string, ~secretName: string) => t =
    "withSecretVariable"

  @send
  external withService: (t, ~serviceId: string) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"

  @send
  external zip: t => File.f = "zip"
}

module File = {
  type t

  @send
  external id: t => string = "id"

  @send
  external chmod: (t, ~mode: string) => t = "chmod"

  @send
  external md5: t => string = "md5"

  @send
  external path: t => string = "path"

  @send
  external sha256: t => string = "sha256"

  @send
  external tarXzvf: t => Directory.t = "tarXzvf"

  @send
  external unzip: t => Directory.t = "unzip"

  @send
  external zip: t => t = "zip"
}

module Git = {
  type t

  @send
  external id: t => string = "id"

  @send
  external branch: (t, ~name: string) => t = "branch"

  @send
  external commit: t => string = "commit"

  @send
  external tree: t => Directory.t = "tree"
}

module Client = {
  type t

  @send
  external awsSecretManager: (t, ~region: string) => SecretManager.t = "awsSecretManager"

  @send
  external azureKeyvault: (
    t,
    ~vault: string,
    ~clientId: string,
    ~clientSecret: string,
    ~tenantId: string,
    ~keyvaultName: string,
    ~keyvaultUrl: string,
  ) => SecretManager.t = "azureKeyvault"

  @send
  external cache: (t, ~key: string) => Cache.t = "cache"

  @send
  external call: (t, ~func: string, ~args: Belt.Array.t<string>) => string = "call"

  @send
  external devbox: t => Devbox.t = "devbox"

  @send
  external directory: (t, ~path: string) => Directory.t = "directory"

  @send
  external envhub: t => Envhub.t = "envhub"

  @send
  external file: (t, ~path: string) => File.t = "file"

  @send
  external flox: t => Flox.t = "flox"

  @send
  external getArch: t => string = "getArch"

  @send
  external getEnv: (t, ~name: string) => string = "getEnv"

  @send
  external getOS: t => string = "getOS"

  @send
  external git: (t, ~url: string) => Git.t = "git"

  @send
  external googleCloudSecretManager: (
    t,
    ~project: string,
    ~googleCredentialsFile: string,
  ) => SecretManager.t = "googleCloudSecretManager"

  @send
  external hasEnv: (t, ~name: string) => bool = "hasEnv"

  @send
  external hashicorpVault: (
    t,
    ~address: string,
    ~token: string,
    ~cacerts: string,
  ) => SecretManager.t = "hashicorpVault"

  @send
  external http: (t, ~url: string) => File.t = "http"

  @send
  external mise: t => Mise.t = "mise"

  @send
  external nix: (t, ~args: NixArgs.t=?) => Nix.t = "nix"

  @send
  external pipeline: (t, ~name: string) => Pipeline.t = "pipeline"

  @send
  external pixi: t => Pixi.t = "pixi"

  @send
  external pkgx: t => Pkgx.t = "pkgx"

  @send
  external removeEnv: (t, ~name: string) => unit = "removeEnv"

  @send
  external setEnvs: (t, ~envs: Belt.Map.String.t<string>) => unit = "setEnvs"

  @send
  external setSecret: (t, ~name: string, ~value: string) => Secret.t = "setSecret"
}

@module("jsr:@fluentci/sdk/plugin") @val external dag: Client.t = "dag"

module Node = {
  @module("@fluentci/sdk/plugin") @val external dag: Client.t = "dag"
}

module Bun = {
  @module("@fluentci/sdk/plugin") @val external dag: Client.t = "dag"
}

module Deno = {
  @module("jsr:@fluentci/sdk/plugin") @val external dag: Client.t = "dag"
}
