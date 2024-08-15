module CacheVolume = {
  type t

  @send
  external id: t => Promise.t<string> = "id"
}

module Port = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external port: t => Promise.t<int> = "port"

  @send
  external protocol: t => Promise.t<string> = "protocol"
}

module Secret = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external value: t => Promise.t<string> = "value"

  @send
  external plaintext: t => Promise.t<string> = "plaintext"
}

module EnvVariable = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external value: t => Promise.t<string> = "value"
}

module ModuleDependency = {
  type t
}

module EnumValueTypeDef = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external description: t => Promise.t<string> = "description"

  @send
  external name: t => Promise.t<string> = "name"
}
module EnumTypeDef = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external description: t => Promise.t<string> = "description"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external sourceModuleName: t => Promise.t<string> = "sourceModuleName"

  @send
  external values: t => Promise.t<Belt.Array.t<EnumValueTypeDef.t>> = "values"
}

module FieldTypeDef = {
  type t

  module TypeDef = {
    type t

    @send
    external id: t => Promise.t<string> = "id"
  }

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external description: t => Promise.t<string> = "description"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external typeDef: t => TypeDef.t = "typeDef"
}

module InputTypeDef = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external fields: t => Promise.t<Belt.Array.t<FieldTypeDef.t>> = "fields"

  @send
  external name: t => Promise.t<string> = "name"
}

module InterfaceTypeDef = {
  type t

  module TypeDef = {
    type t

    @send
    external id: t => Promise.t<string> = "id"
  }

  module FunctionArg = {
    type t

    @send
    external id: t => Promise.t<string> = "id"

    @send
    external defaultValue: t => Promise.t<JSON.t> = "defaultValue"

    @send
    external description: t => Promise.t<string> = "description"

    @send
    external name: t => Promise.t<string> = "name"

    @send
    external typeDef: t => TypeDef.t = "typeDef"
  }

  module Function = {
    type t

    @send
    external id: t => Promise.t<string> = "id"

    @send
    external args: t => Promise.t<Belt.Array.t<FunctionArg.t>> = "args"

    @send
    external description: t => Promise.t<string> = "description"

    @send
    external name: t => Promise.t<string> = "name"

    @send
    external returnType: t => Promise.t<TypeDef.t> = "returnType"

    @send
    external withArg: (t, TypeDef.t) => t = "withArg"

    @send
    external withDescription: (t, string) => t = "withDescription"
  }

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external description: t => Promise.t<string> = "description"

  @send
  external functions: t => Promise.t<Belt.Array.t<Function.t>> = "functions"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external sourceModuleName: t => Promise.t<string> = "sourceModuleName"
}

module ObjectTypeDef = {
  type t

  module TypeDef = {
    type t

    @send
    external id: t => Promise.t<string> = "id"
  }

  module FunctionArg = {
    type t

    @send
    external id: t => Promise.t<string> = "id"

    @send
    external defaultValue: t => Promise.t<JSON.t> = "defaultValue"

    @send
    external description: t => Promise.t<string> = "description"

    @send
    external name: t => Promise.t<string> = "name"

    @send
    external typeDef: t => TypeDef.t = "typeDef"
  }

  module Function = {
    type t

    @send
    external id: t => Promise.t<string> = "id"

    @send
    external args: t => Promise.t<Belt.Array.t<FunctionArg.t>> = "args"

    @send
    external description: t => Promise.t<string> = "description"

    @send
    external name: t => Promise.t<string> = "name"

    @send
    external returnType: t => Promise.t<TypeDef.t> = "returnType"

    @send
    external withArg: (t, TypeDef.t) => t = "withArg"

    @send
    external withDescription: (t, string) => t = "withDescription"
  }

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external description: t => Promise.t<string> = "description"

  @send
  external fields: t => Promise.t<Belt.Array.t<FieldTypeDef.t>> = "fields"

  @send
  external fucntions: t => Promise.t<Belt.Array.t<Function.t>> = "fucntions"
}

module ScalarTypeDef = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external sourceModuleName: t => Promise.t<string> = "sourceModuleName"
}

module TypeDefKind = {
  type t =
    | BooleanKind
    | EnumKind
    | InputKind
    | IntegerKind
    | InterfaceKind
    | ListKind
    | ObjectKind
    | ScalarKind
    | StringKind
    | VoidKind
}

module TypeDef = {
  type t

  module FunctionArg = {
    type f

    @send
    external id: f => Promise.t<string> = "id"

    @send
    external defaultValue: f => Promise.t<JSON.t> = "defaultValue"

    @send
    external description: f => Promise.t<string> = "description"

    @send
    external name: f => Promise.t<string> = "name"

    @send
    external typeDef: f => t = "typeDef"
  }
  module Function = {
    type f

    @send
    external id: f => Promise.t<string> = "id"

    @send
    external args: f => Promise.t<Belt.Array.t<FunctionArg.f>> = "args"

    @send
    external description: f => Promise.t<string> = "description"

    @send
    external name: f => Promise.t<string> = "name"

    @send
    external returnType: f => Promise.t<t> = "returnType"

    @send
    external withArg: (f, t) => f = "withArg"

    @send
    external withDescription: (f, string) => f = "withDescription"
  }

  module ListTypeDef = {
    type l

    @send
    external id: l => Promise.t<string> = "id"

    @send
    external elementTypeDef: l => Promise.t<t> = "elementTypeDef"
  }

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external asEnum: t => EnumTypeDef.t = "asEnum"

  @send
  external asInput: t => InputTypeDef.t = "asInput"

  @send
  external asInterface: t => InterfaceTypeDef.t = "asInterface"

  @send
  external asList: t => ListTypeDef.l = "asList"

  @send
  external asObject: t => ObjectTypeDef.t = "asObject"

  @send
  external asScalar: t => ScalarTypeDef.t = "asScalar"

  @send
  external kind: t => Promise.t<TypeDefKind.t> = "kind"

  @send
  external optional: t => Promise.t<bool> = "optional"

  @send
  external withConstructor: (t, Function.f) => t = "withConstructor"

  @send
  external withEnum: (t, string) => t = "withEnum"

  @send
  external withEnumValue: (t, string) => t = "withEnumValue"

  @send
  external withField: (t, ~name: string, ~typeDef: t) => t = "withField"

  @send
  external withFunction: (t, Function.f) => t = "withFunction"

  @send
  external withInterface: (t, string) => t = "withInterface"

  @send
  external withKind: (t, TypeDefKind.t) => t = "withKind"

  @send
  external withListOf: (t, t) => t = "withListOf"

  @send
  external withObject: (t, string) => t = "withObject"

  @send
  external withOptional: (t, bool) => t = "withOptional"

  @send
  external withScalar: (t, string) => t = "withScalar"
}

module Service = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external endpoint: t => Promise.t<string> = "endpoint"

  @send
  external hostname: t => Promise.t<string> = "hostname"

  @send
  external ports: t => Promise.t<Belt.Array.t<Port.t>> = "ports"

  @send
  external start: t => Promise.t<t> = "start"

  @send
  external stop: t => Promise.t<t> = "stop"

  @send
  external up: t => Promise.t<t> = "up"
}

module Socket = {
  type t

  @send
  external id: t => Promise.t<string> = "id"
}

module File = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external contents: t => Promise.t<string> = "contents"

  @send
  external digest: t => Promise.t<string> = "digest"

  @send
  external export: (t, ~path: string) => Promise.t<string> = "export"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external size: t => Promise.t<int> = "size"

  @send
  external sync: t => Promise.t<t> = "sync"

  @send
  external withName: (t, string) => t = "withName"

  @send
  external withTimestamps: (t, int) => t = "withTimestamps"
}

module Directory = {
  type d

  module Container = {
    type t

    @send
    external id: t => Promise.t<string> = "id"

    @send
    external from: (t, string) => t = "from"

    @send
    external withExec: (t, Belt.Array.t<string>) => t = "withExec"

    @send
    external stdout: t => Promise.t<string> = "stdout"

    @send
    external stderr: t => Promise.t<string> = "stderr"

    @send
    external rootfs: t => Promise.t<string> = "rootfs"

    @send
    external sync: t => Promise.t<t> = "sync"

    @send
    external terminal: t => t = "terminal"

    @send
    external user: t => Promise.t<string> = "user"

    @send
    external withDefaultArgs: (t, Belt.Array.t<string>) => t = "withDefaultArgs"

    @send
    external withDefaultTerminalCmd: (t, string) => t = "withDefaultTerminalCmd"

    @send
    external withDirectory: (t, ~path: string, ~directory: string) => t = "withDirectory"

    @send
    external withEntrypoint: (t, Belt.Array.t<string>) => t = "withEntrypoint"

    @send
    external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

    @send
    external withExposedPort: (t, int) => t = "withExposedPort"

    @send
    external withFile: (t, ~path: string, ~source: string) => t = "withFile"

    @send
    external withFiles: (t, ~path: string, ~sources: Belt.Array.t<string>) => t = "withFiles"

    @send
    external withFocus: t => t = "withFocus"

    @send
    external withLabel: (t, ~name: string, ~value: string) => t = "withLabel"

    @send
    external withMountedCache: (t, ~path: string, ~cache: CacheVolume.t) => t = "withMountedCache"

    @send
    external withMountedDirectory: (t, ~path: string, ~source: d) => t = "withMountedDirectory"

    @send
    external withMountedFile: (t, ~path: string, ~source: File.t) => t = "withMountedFile"

    @send
    external withMountedSecret: (t, ~path: string, ~secret: Secret.t) => t = "withMountedSecret"

    @send
    external withMountedTemp: (t, string) => t = "withMountedTemp"

    @send
    external withNewFile: (t, ~path: string, ~contents: string) => t = "withNewFile"

    @send
    external withRegistryAuth: (t, ~address: string, ~username: string, ~secret: Secret.t) => t =
      "withRegistryAuth"

    @send
    external withRootfs: (t, ~directory: d) => t = "withRootfs"

    @send
    external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

    @send
    external withServiceBinding: (t, ~alias: string, ~service: Service.t) => t =
      "withServiceBinding"

    @send
    external withUnixSocket: (t, ~path: string, ~source: Socket.t) => t = "withUnixSocket"

    @send
    external withUser: (t, string) => t = "withUser"

    @send
    external withWorkdir: (t, string) => t = "withWorkdir"

    @send
    external withoutDefaultArgs: t => t = "withoutDefaultArgs"

    @send
    external withoutDirectory: (t, string) => t = "withoutDirectory"

    @send
    external withoutEntrypoint: t => t = "withoutEntrypoint"

    @send
    external withoutEnvVariable: (t, string) => t = "withoutEnvVariable"

    @send
    external withoutExposedPort: (t, int) => t = "withoutExposedPort"

    @send
    external withoutFile: (t, string) => t = "withoutFile"

    @send
    external withoutFocus: t => t = "withoutFocus"

    @send
    external withoutLabel: (t, string) => t = "withoutLabel"

    @send
    external withoutMount: (t, string) => t = "withoutMount"

    @send
    external withoutRegistryAuth: (t, string) => t = "withoutRegistryAuth"

    @send
    external withoutSecretVariable: (t, string) => t = "withoutSecretVariable"

    @send
    external withoutUnixSocket: (t, string) => t = "withoutUnixSocket"

    @send
    external withoutUser: t => t = "withoutUser"

    @send
    external withoutWorkdir: t => t = "withoutWorkdir"

    @send
    external workdir: t => Promise.t<string> = "workdir"
  }

  module Module = {
    type t

    @send
    external id: t => Promise.t<string> = "id"

    @send
    external dependencies: t => Promise.t<Belt.Array.t<t>> = "dependencies"

    @send
    external dependencyConfig: t => Promise.t<ModuleDependency.t> = "dependencyConfig"

    @send
    external description: t => Promise.t<string> = "description"

    @send
    external enums: t => Promise.t<Belt.Array.t<TypeDef.t>> = "enums"

    @send
    external generatedContextDiff: t => d = "generatedContextDiff"

    @send
    external generatedContextDirectory: t => d = "generatedContextDirectory"
  }

  @send
  external id: d => Promise.t<string> = "id"

  @send
  external asModule: d => Module.t = "asModule"

  @send
  external diff: (d, ~other: d) => d = "diff"

  @send
  external directory: (d, string) => d = "directory"

  @send
  external dockerBuild: d => Container.t = "dockerBuild"

  @send
  external entries: d => Promise.t<Belt.Array.t<string>> = "entries"

  @send
  external export: (d, string) => Promise.t<string> = "export"

  @send
  external file: (d, string) => File.t = "file"

  @send
  external glob: (d, string) => Promise.t<Belt.Array.t<string>> = "glob"

  @send
  external pipeline: (d, string) => Container.t = "pipeline"

  @send
  external sync: d => Promise.t<d> = "sync"

  @send
  external terminal: d => d = "terminal"

  @send
  external withDirectory: (d, ~path: string, ~source: d) => d = "withDirectory"

  @send
  external withFile: (d, ~path: string, ~source: File.t) => d = "withFile"

  @send
  external withFiles: (d, ~path: string, ~sources: Belt.Array.t<File.t>) => d = "withFiles"

  @send
  external withNewDirectory: (d, ~path: string) => d = "withNewDirectory"

  @send
  external withNewFile: (d, ~path: string, ~contents: string) => d = "withNewFile"

  @send
  external withTimestamps: (d, int) => d = "withTimestamps"

  @send
  external withoutDirectory: (d, string) => d = "withoutDirectory"

  @send
  external withoutFile: (d, string) => d = "withoutFile"
}

module Host = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external directory: (t, string) => Directory.d = "directory"

  @send
  external file: (t, string) => File.t = "file"

  @send
  external service: t => Service.t = "service"

  @send
  external setSecretFile: (t, ~name: string, ~path: string) => Secret.t = "setSecretFile"

  @send
  external tunnel: (t, Service.t) => Service.t = "tunnel"

  @send
  external unixSocket: (t, string) => Socket.t = "unixSocket"
}

module GitRef = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external commit: t => Promise.t<string> = "commit"

  @send
  external tree: t => Directory.d = "tree"
}

module GitRepository = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external branch: (t, string) => GitRef.t = "branch"

  @send
  external commit: (t, string) => GitRef.t = "commit"

  @send
  external head: t => GitRef.t = "head"

  @send
  external ref: (t, string) => GitRef.t = "ref"

  @send
  external tag: (t, string) => GitRef.t = "tag"

  @send
  external tags: t => Promise.t<Belt.Array.t<string>> = "tags"

  @send
  external withAuthHeader: (t, Secret.t) => t = "withAuthHeader"

  @send
  external withAuthToken: (t, Secret.t) => t = "withAuthToken"
}

module Label = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external value: t => Promise.t<string> = "value"
}

module ContainerExecOpts = {
  type t = {
    skipEntrypoint?: bool,
    useEntrypoint?: bool,
    stdin?: string,
    redirectStdout?: bool,
    redirectStderr?: bool,
    experimentalPrivilegeNesting?: bool,
    insecureRootCapabilities?: bool,
  }
}

module Container = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external from: (t, string) => t = "from"

  @send
  external withExec: (t, Belt.Array.t<string>, ~opts: ContainerExecOpts.t=?) => t = "withExec"

  @send
  external stdout: t => Promise.t<string> = "stdout"

  @send
  external stderr: t => Promise.t<string> = "stderr"

  @send
  external asTarball: t => Promise.t<string> = "asTarball"

  @send
  external build: (t, ~context: Directory.d) => t = "build"

  @send
  external defaultArgs: t => Promise.t<Belt.Array.t<string>> = "defaultArgs"

  @send
  external directory: t => Directory.d = "directory"

  @send
  external entrypoint: t => Promise.t<Belt.Array.t<string>> = "entrypoint"

  @send
  external envVariable: (t, string) => Promise.t<string> = "envVariable"

  @send
  external envVariables: t => Promise.t<Belt.Array.t<EnvVariable.t>> = "envVariables"

  @send
  external export: (t, ~path: string) => Promise.t<string> = "export"

  @send
  external exposedPorts: t => Promise.t<Belt.Array.t<Port.t>> = "exposedPorts"

  @send
  external file: (t, string) => File.t = "file"

  @send
  external imageRef: t => Promise.t<string> = "imageRef"

  @send
  external import: (t, string) => t = "import_"

  @send
  external label: (t, string) => Promise.t<string> = "label"

  @send
  external labels: t => Promise.t<Belt.Array.t<Label.t>> = "labels"

  @send
  external mounts: t => Promise.t<Belt.Array.t<string>> = "mounts"

  @send
  external pipeline: (t, string) => t = "pipeline"

  @send
  external platform: t => Promise.t<string> = "platform"

  @send
  external publish: (t, ~address: string) => Promise.t<string> = "publish"

  @send
  external rootfs: t => Promise.t<string> = "rootfs"

  @send
  external sync: t => Promise.t<t> = "sync"

  @send
  external terminal: t => t = "terminal"

  @send
  external user: t => Promise.t<string> = "user"

  @send
  external withDefaultArgs: (t, Belt.Array.t<string>) => t = "withDefaultArgs"

  @send
  external withDefaultTerminalCmd: (t, string) => t = "withDefaultTerminalCmd"

  @send
  external withDirectory: (t, ~path: string, ~directory: string) => t = "withDirectory"

  @send
  external withEntrypoint: (t, Belt.Array.t<string>) => t = "withEntrypoint"

  @send
  external withEnvVariable: (t, ~name: string, ~value: string) => t = "withEnvVariable"

  @send
  external withExposedPort: (t, int) => t = "withExposedPort"

  @send
  external withFile: (t, ~path: string, ~source: string) => t = "withFile"

  @send
  external withFiles: (t, ~path: string, ~sources: Belt.Array.t<string>) => t = "withFiles"

  @send
  external withFocus: t => t = "withFocus"

  @send
  external withLabel: (t, ~name: string, ~value: string) => t = "withLabel"

  @send
  external withMountedCache: (t, ~path: string, ~cache: CacheVolume.t) => t = "withMountedCache"

  @send
  external withMountedDirectory: (t, ~path: string, ~source: Directory.d) => t =
    "withMountedDirectory"

  @send
  external withMountedFile: (t, ~path: string, ~source: File.t) => t = "withMountedFile"

  @send
  external withMountedSecret: (t, ~path: string, ~secret: Secret.t) => t = "withMountedSecret"

  @send
  external withMountedTemp: (t, string) => t = "withMountedTemp"

  @send
  external withNewFile: (t, ~path: string, ~contents: string) => t = "withNewFile"

  @send
  external withRegistryAuth: (t, ~address: string, ~username: string, ~secret: Secret.t) => t =
    "withRegistryAuth"

  @send
  external withRootfs: (t, ~directory: Directory.d) => t = "withRootfs"

  @send
  external withSecretVariable: (t, ~name: string, ~secret: Secret.t) => t = "withSecretVariable"

  @send
  external withServiceBinding: (t, ~alias: string, ~service: Service.t) => t = "withServiceBinding"

  @send
  external withUnixSocket: (t, ~path: string, ~source: Socket.t) => t = "withUnixSocket"

  @send
  external withUser: (t, string) => t = "withUser"

  @send
  external withWorkdir: (t, string) => t = "withWorkdir"

  @send
  external withoutDefaultArgs: t => t = "withoutDefaultArgs"

  @send
  external withoutDirectory: (t, string) => t = "withoutDirectory"

  @send
  external withoutEntrypoint: t => t = "withoutEntrypoint"

  @send
  external withoutEnvVariable: (t, string) => t = "withoutEnvVariable"

  @send
  external withoutExposedPort: (t, int) => t = "withoutExposedPort"

  @send
  external withoutFile: (t, string) => t = "withoutFile"

  @send
  external withoutFocus: t => t = "withoutFocus"

  @send
  external withoutLabel: (t, string) => t = "withoutLabel"

  @send
  external withoutMount: (t, string) => t = "withoutMount"

  @send
  external withoutRegistryAuth: (t, string) => t = "withoutRegistryAuth"

  @send
  external withoutSecretVariable: (t, string) => t = "withoutSecretVariable"

  @send
  external withoutUnixSocket: (t, string) => t = "withoutUnixSocket"

  @send
  external withoutUser: t => t = "withoutUser"

  @send
  external withoutWorkdir: t => t = "withoutWorkdir"

  @send
  external workdir: t => Promise.t<string> = "workdir"
}

module ModuleSource = {
  type t

  @send
  external id: t => Promise.t<string> = "id"
}

module Module = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external dependencies: t => Promise.t<Belt.Array.t<t>> = "dependencies"

  @send
  external dependencyConfig: t => Promise.t<ModuleDependency.t> = "dependencyConfig"

  @send
  external description: t => Promise.t<string> = "description"

  @send
  external enums: t => Promise.t<Belt.Array.t<TypeDef.t>> = "enums"

  @send
  external generatedContextDiff: t => Directory.d = "generatedContextDiff"

  @send
  external generatedContextDirectory: t => Directory.d = "generatedContextDirectory"

  @send
  external intialize: t => t = "intialize"

  @send
  external interfaces: t => Promise.t<Belt.Array.t<TypeDef.t>> = "interfaces"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external objects: t => Promise.t<Belt.Array.t<TypeDef.t>> = "objects"

  @send
  external runtime: t => Promise.t<Container.t> = "runtime"

  @send
  external sdk: t => Promise.t<string> = "sdk"

  @send
  external serve: t => Promise.t<unit> = "serve"

  @send
  external source: t => ModuleSource.t = "source"

  @send
  external withDescription: (t, string) => t = "withDescription"

  @send
  external withEnum: (t, TypeDef.t) => t = "withEnum"

  @send
  external withInterface: (t, TypeDef.t) => t = "withInterface"

  @send
  external withObject: (t, TypeDef.t) => t = "withObject"

  @send
  external withSource: (t, ModuleSource.t) => t = "withSource"
}

module JSON = {
  type t
}

module FunctionArg = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external defaultValue: t => Promise.t<JSON.t> = "defaultValue"

  @send
  external description: t => Promise.t<string> = "description"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external typeDef: t => TypeDef.t = "typeDef"
}
module Function = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external args: t => Promise.t<Belt.Array.t<FunctionArg.t>> = "args"

  @send
  external description: t => Promise.t<string> = "description"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external returnType: t => Promise.t<TypeDef.t> = "returnType"

  @send
  external withArg: (t, TypeDef.t) => t = "withArg"

  @send
  external withDescription: (t, string) => t = "withDescription"
}

module FunctionCallArgValue = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external value: t => Promise.t<JSON.t> = "value"
}

module FunctionCall = {
  type t

  @send
  external id: t => Promise.t<string> = "id"

  @send
  external inputArgs: t => Promise.t<Belt.Array.t<FunctionCallArgValue.t>> = "inputArgs"

  @send
  external name: t => Promise.t<string> = "name"

  @send
  external parent: t => Promise.t<JSON.t> = "parent"

  @send
  external parentName: t => Promise.t<string> = "parentName"

  @send
  external returnValue: (t, JSON.t) => Promise.t<unit> = "returnValue"
}

module Client = {
  type t

  @send
  external pipeline: (t, string) => t = "pipeline"

  @send
  external container: t => Container.t = "container"

  @send
  external cacheVolume: t => CacheVolume.t = "cacheVolume"

  @send
  external directory: t => Directory.d = "directory"

  @send
  external git: (t, string) => GitRepository.t = "git"

  @send
  external host: t => Host.t = "host"

  @send
  external http: (t, string) => File.t = "http"
}

@module("jsr:@fluentci/sdk/dagger") @val external dag: Client.t = "dag"

module Node = {
  @module("@fluentci/sdk/dagger") @val external dag: Client.t = "dag"
}

module Bun = {
  @module("@fluentci/sdk/dagger") @val external dag: Client.t = "dag"
}

module Deno = {
  @module("jsr:@fluentci/sdk/dagger") @val external dag: Client.t = "dag"
}
