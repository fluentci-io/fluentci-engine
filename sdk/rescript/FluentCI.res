module NixArgs = {
  type t = {impure?: bool}
}

module Cache = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external key: t => Promise.t<string> = "key"
}

module Service = {
  type t

  @send
  external id: t => Promise.t<string> = "id"
}

module Secret = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external mount: t => Promise.t<string> = "mount"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external plaintext: t => Promise.t<string> = "plaintext"
}

module Devbox = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asService: (t, ~name: string) => Service.t = "asService"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Devenv = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asService: (t, ~name: string) => Service.t = "asService"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Envhub = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asService: (t, ~name: string) => Service.t = "asService"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external use: (t, ~environment: string) => t = "use"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Flox = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asService: (t, ~name: string) => Service.t = "asService"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Mise = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asService: (t, ~name: string) => Service.t = "asService"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Nix = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asService: (t, ~name: string) => Service.t = "asService"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Pixi = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asService: (t, ~name: string) => Service.t = "asService"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module Pkgx = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asService: (t, ~name: string) => Service.t = "asService"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withPackages: (t, Belt.Array.t<string>) => t = "withPackages"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module File = {
  type t

  module Directory = {
    type d

    @send
    external id: d => Promise.t<string> = "id"

    @send
    external devbox: d => Devbox.t = "devbox"

    @send
    external devenv: d => Devenv.t = "devenv"

    @send
    external directory: (d, ~path: string) => d = "directory"

    @send
    external entries: d => Promise.t<Belt.Array.t<string>> = "entries"

    @send
    external envhub: d => Envhub.t = "envhub"

    @send
    external flox: d => Flox.t = "flox"

    @send
    external mise: d => Mise.t = "mise"

    @send
    external nix: (d, NixArgs.t) => Nix.t = "nix"

    @send
    external pixi: d => Pixi.t = "pixi"

    @send
    external pkgx: d => Pkgx.t = "pkgx"

    @send
    external stderr: d => Promise.t<string> = "stderr"

    @send
    external stdout: d => Promise.t<string> = "stdout"

    @send
    external tarCzvf: d => t = "tarCzvf"

    @send
    external waitOn: (d, ~port: int, ~timeout: int=?) => d = "waitOn"

    @send
    external withCache: (d, ~path: string, ~cache: Cache.t) => d = "withCache"

    @send
    external withEnvVariable: (d, ~name: string, ~value: string) => d = "withEnvVariable"

    @send
    external withExec: (d, Belt.Array.t<string>) => d = "withExec"

    @send
    external withFile: (d, ~path: string, ~fileID: string) => d = "withFile"

    @send
    external withSecretVariable: (d, ~name: string, ~secret: Secret.t) => d = "withSecretVariable"

    @send
    external withService: (t, Service.t) => t = "withService"

    @send
    external withWorkdir: (d, ~path: string) => d = "withWorkdir"

    @send
    external zip: d => t = "zip"
  }

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external chmod: (t, ~mode: string) => t = "chmod"

  @send
  external md5: t => Promise.t<string> = "md5"

  @send
  external path: t => Promise.t<string> = "path"

  @send
  external sha256: t => Promise.t<string> = "sha256"

  @send
  external tarCzvf: t => t = "tarCzvf"

  @send
  external tarXzvf: (t, ~outputDir: string=?) => Directory.d = "tarXzvf"

  @send
  external unzip: (t, ~outputDir: string=?) => Directory.d = "unzip"

  @send
  external zip: t => t = "zip"
}

module Directory = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external devbox: t => Devbox.t = "devbox"

  @send
  external devenv: t => Devenv.t = "devenv"

  @send
  external directory: (t, ~path: string) => t = "directory"

  @send
  external entries: t => Promise.t<Belt.Array.t<string>> = "entries"

  @send
  external envhub: t => Envhub.t = "envhub"

  @send
  external flox: t => Flox.t = "flox"

  @send
  external mise: t => Mise.t = "mise"

  @send
  external nix: (t, NixArgs.t) => Nix.t = "nix"

  @send
  external pixi: t => Pixi.t = "pixi"

  @send
  external pkgx: t => Pkgx.t = "pkgx"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external tarCzvf: t => File.t = "tarCzvf"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"

  @send
  external zip: t => File.t = "zip"
}

module Git = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external branch: (t, string) => t = "branch"

  @send
  external commit: t => Promise.t<string> = "commit"

  @send
  external tree: t => Directory.t = "tree"
}

module Pipeline = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asService: (t, ~name: string) => Service.t = "asService"

  @send
  external devbox: t => Devbox.t = "devbox"

  @send
  external devenv: t => Devenv.t = "devenv"

  @send
  external envhub: t => Envhub.t = "envhub"

  @send
  external flox: t => Flox.t = "flox"

  @send
  external git: (t, ~url: string) => Git.t = "git"

  @send
  external http: (t, ~url: string) => File.t = "http"

  @send
  external mise: t => Mise.t = "mise"

  @send
  external nix: (t, NixArgs.t) => Nix.t = "nix"

  @send
  external pixi: t => Pixi.t = "pixi"

  @send
  external pkgx: t => Pkgx.t = "pkgx"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external waitOn: (t, ~port: int, ~timeout: int=?) => t = "waitOn"

  @send
  external withCache: (t, ~path: string, ~cache: Cache.t) => t = "withCache"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExec: (t, Belt.Array.t<string>) => t = "withExec"

  @send
  external withFile: (t, ~path: string, ~fileID: string) => t = "withFile"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withService: (t, Service.t) => t = "withService"

  @send
  external withWorkdir: (t, ~path: string) => t = "withWorkdir"
}

module SecretManager = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external getSecret: (t, ~name: string) => Promise.t<Belt.Array.t<Secret.t>> = "getSecret"
}

module Client = {
  type t

  @send
  external awsSecretManager: (t, ~region: string) => SecretManager.t = "awsSecretManager"

  @send
  external azureKeyvault: (
    t,
    ~clientId: string,
    ~clientSecret: string,
    ~tenantId: string,
    ~keyvaultName: string,
    ~keyvaultUrl: string,
  ) => SecretManager.t = "azureKeyvault"

  @send
  external cache: (t, ~key: string) => Cache.t = "cache"

  @send
  external devbox: t => Devbox.t = "devbox"

  @send
  external devenv: t => Devenv.t = "devenv"

  @send
  external directory: (t, ~path: string) => Directory.t = "directory"

  @send
  external file: (t, ~path: string) => File.t = "file"

  @send
  external flox: t => Flox.t = "flox"

  @send
  external git: (t, ~url: string) => Git.t = "git"

  @send
  external googleCloudSecretManager: (
    t,
    ~project: string,
    ~gogleCredentialsFile: string,
  ) => SecretManager.t = "googleCloudSecretManager"

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
  external nix: (t, NixArgs.t) => Nix.t = "nix"

  @send
  external pipeline: (t, ~name: string) => Pipeline.t = "pipeline"

  @send
  external pkgx: t => Pkgx.t = "pkgx"

  @send
  external setSecret: (t, ~name: string, ~value: string) => Secret.t = "setSecret"
}

module Node = {
  @module("@fluentci/sdk") @val external dag: Client.t = "dag"
}

module Bun = {
  @module("@fluentci/sdk") @val external dag: Client.t = "dag"
}

module Deno = {
  @module("jsr:@fluentci/sdk") @val external dag: Client.t = "dag"
}
