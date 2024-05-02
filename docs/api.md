# GraphQL API

The GraphQL API is the main way to interact with FluentCI Engine.

## Queries

The query type defines GraphQL operations that retrieve data from the server.
For more information, see "[Forming calls with GraphQL](https://docs.github.com/en/graphql/guides/forming-calls-with-graphql#about-queries)."

### pipeline

Creates a named sub-pipeline.

**type**: [Pipeline](#pipeline-1)

**Arguments for `pipeline`**

| Name  | type                                                        | Description               |
| ----- | ----------------------------------------------------------- | ------------------------- |
| name  | [String](https://spec.graphql.org/October2021/#sec-String)! | The name of the pipeline. |

### cache

Constructs a cache for a given cache key.

**type**: [Cache](#cache-1)

**Arguments for `cache`**

| Name  | type     | Description |
| ----- | -------- | ----------- |
| key   | [String](https://spec.graphql.org/October2021/#sec-String)! | A unique key to identify the cache. |


### http

Returns a file containing an http remote url content.

**type**: [File](#file-1)

**Arguments for `file`**

| Name     | type     | Description  |
| -------- | -------- | ------------ |
| url      | [String](https://spec.graphql.org/October2021/#sec-String)! | The URL of the file to download. |

### directory

Constructs a directory object for a given path.

**type**: [Directory](#directory-1)

**Arguments for `directory`**

| Name    | type        | Description  |
| ------- | ----------- | ------------ |
| path    | [String](https://spec.graphql.org/October2021/#sec-String)! | The path of the directory to access |

### file

Constructs a file object for a given path.

**Arguments for `File`**

**type**: [File](#file-1)

| Name    | type        | Description  |
| ------- | ----------- | ------------ |
| path    | [String](https://spec.graphql.org/October2021/#sec-String)! | The path of the file to access |

### git

Constructs a git object for a given repository URL.

**type**: [Git](#git-1)

**Arguments for `git`**

| Name    | type        | Description  |
| ------- | ----------- | ------------ |
| url    | [String](https://spec.graphql.org/October2021/#sec-String)! | The url of the repository to access |

### devbox

Creates a devbox environment.

**type**: [Devbox](#devbox-1)

### devenv

Creates a devenv environment.

**type**: [Devenv](#devenv-1)

### flox

Creates a flox environment.

**type**: [Flox](#flox-1)

### nix

Creates a nix environment.

**type**: [Nix](#nix-1)

### pkgx

Creates a pkgx environment.

**type**: [Pkgx](#pkgx-1)

### mise

Creates a mise environment.

**type**: [Mise](#mise-1)

### pixi

Creates a pixi environment.

**type**: [Pixi](#pixi-1)

### googleCloudSecretManager

Set up Google Cloud Secret Manager Client

**type**: [SecretManager!](#secretmanager)

**Arguments for `googleCloudSecretManager`**

| Name    | type        | Description  |
| ------- | ----------- | ------------ |
| project    | [String](https://spec.graphql.org/October2021/#sec-String)! | Google Cloud Project name |
| googleCredentialsFile | [String](https://spec.graphql.org/October2021/#sec-String)! | Path to the Google Credentials file |

### awsSecretsManager

Set up AWS Secrets Manager Client

**type**: [SecretManager!](#secretmanager)

**Arguments for `awsSecretsManager`**

| Name            | type                                                        |
| --------------- | ----------------------------------------------------------- |
| region          | [String](https://spec.graphql.org/October2021/#sec-String)! |
| accessKeyId     | [String](https://spec.graphql.org/October2021/#sec-String)! |
| secretAccessKey | [String](https://spec.graphql.org/October2021/#sec-String)! |

### azureKeyvault

Set up Azure Key Vault Client

**type**: [SecretManager!](#secretmanager)

**Arguments for `azureKeyvault`**

| Name            | type                                                        |
| --------------- | ----------------------------------------------------------- |
| clientId          | [String](https://spec.graphql.org/October2021/#sec-String)! |
| clientSecret     | [String](https://spec.graphql.org/October2021/#sec-String)! |
| tenantId | [String](https://spec.graphql.org/October2021/#sec-String)! |
| keyvaultName | [String](https://spec.graphql.org/October2021/#sec-String)! |
| keyvaultUrl | [String](https://spec.graphql.org/October2021/#sec-String)! |

### hashicorpVault

Set up HashiCorp Vault Client

**type**: [SecretManager!](#secretmanager)

**Arguments for `hashicorpVault`**

| Name      | type                                                        |
| --------- | ----------------------------------------------------------- |
| address   | [String](https://spec.graphql.org/October2021/#sec-String)! |
| token     | [String](https://spec.graphql.org/October2021/#sec-String)! |
| cacerts   | [String](https://spec.graphql.org/October2021/#sec-String)  |

### setSecret

Create a new Secret

**type**: [Secret!](#secret)

**Arguments for `setSecret`**

| Name      | type                                                        |
| --------- | ----------------------------------------------------------- |
| name   | [String](https://spec.graphql.org/October2021/#sec-String)! |
| value   | [String](https://spec.graphql.org/October2021/#sec-String)! |

## Objects

[Objects](https://spec.graphql.org/June2018/#sec-Objects) in GraphQL represent the resources you can access. An object can contain a list of fields, which are specifically typed.

For example, the [Directory](#directory-1) object has a field called `path`, which is a `String`.

For more information, see "[Introduction to GraphQL](https://docs.github.com/en/graphql/guides/introduction-to-graphql)."

### Cache

Fields of the Cache object:

| Name   | Type     | Description                         |
| ------ |--------- | ----------------------------------- |
| id     | [ID](https://spec.graphql.org/October2021/#sec-ID)     | The unique identifier of the cache. |
| key    | [String](https://spec.graphql.org/October2021/#sec-String) | The key of the cache.               |

### Devbox

Fields of the Devbox object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID) | Unique identifier.                  |
| withCache   |  path: `String!`, cacheId: `ID!` | [Devbox](#devbox-1) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Devbox](#devbox-1) | Add file at the specified path.            |
| withExec    |  args: `[String!]!` | [Devbox](#devbox-1) | Add command to execute.             |
| withWorkdir |  path: `String!`     | [Devbox](#devbox-1) | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Devbox](#devbox-1) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Devbox](#devbox-1) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Devbox](#devbox-1) | Wait on a service at a specific port to be ready |
| stdout      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard output.            |
| stderr      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |

### Devenv

Fields of the Devenv object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)               | Unique identifier.                  |
| withCache   |  path: `String!`, cacheId: `ID!` | [Devenv](#devenv-1) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Devenv](#devenv-1) | Add file at the specified path.            |
| withExec    |  args: `[String!]!` | [Devenv](#devenv-1) | Add command to execute.             |
| withWorkdir |  path: `String!`     | [Devenv](#devenv-1) | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Devenv](#devenv-1) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Devenv](#devenv-1) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Devbox](#devbox-1) | Wait on a service at a specific port to be ready |
| stdout      |                     | [String](https://spec.graphql.org/October2021/#sec-String)           | Get the standard output.            |
| stderr      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |

### Directory

Fields of the Directory object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)                | Unique identifier.                  |
| path        |                     | [String](https://spec.graphql.org/October2021/#sec-String)           | Path of the directory               |
| directory   | path: `String!`     | `Directory`         | Returns the Directory at the path.  |
| entries     |                     | [[String](https://spec.graphql.org/October2021/#sec-String)!]!       | List of entries in the directory.   |
| devbox     |           | [Devbox](#devbox-1) | Setup Devbox from the directory |
| devenv     |           | [Devenv](#devenv-1) | Setup Devenv from the directory |
| flox       |           | [Flox](#flox-1)     | Setup Flox from the directory   |
| pkgx       |           | [Pkgx](#pkgx-1)     | Setup devbox from the directory |
| nix        |           | [Nix](#nix-1)       | Setup Nix from the directory    |
| pixi       |           | [Pixi](#pixi-1)     | Setup Pixi from the directory    |
| mise       |           | [Mise](#mise-1)     | Setup Mise from the directory    |
| envhub     |           | [Envhub](#envhub) | Setup Envhub from the directory    |
| withCache   |  path: `String!`, cacheId: `ID!` | [Directory](#directory-1) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Directory](#directory-1) | Add file at the specified path.            |
| withWorkdir  | path: String!           | [Directory](#directory-1)       | Change the work directory  |
| tarCzvf     |                     | [File](#file-1)              | Compress the directory to a tar gzip archive. |
| zip          |                    | [File](#file-1)              | Compress the directory to a zip archive.      |
| withExec    |  args: `[String!]!`    | [Directory](#directory)   | Add command to execute.             |
| withWorkdir |  path: `String!`        | [Directory](#directory)   | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Directory](#directory) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Directory](#directory) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Directory](#directory) | Wait on a service at a specific port to be ready |
| stdout      |                        | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard output.            |
| stderr      |                        | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |

### Envhub

Fields of the Envhub object:

| Name        | Arguments              | Type                | Description                         |
| ----------- | ---------------------- |-------------------- | ----------------------------------- |
| id          |                        | [ID](https://spec.graphql.org/October2021/#sec-ID)               | Unique identifier.                  |
| use         | environment: `String!` | [Envhub](#envhub)   | Use the environment.                |
| withCache   |  path: `String!`, cacheId: `ID!` | [Envhub](#envhub) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Envhub](#envhub) | Add file at the specified path.            |
| withExec    |  args: `[String!]!`    | [Envhub](#envhub)   | Add command to execute.             |
| withWorkdir |  path: `String!`        | [Envhub](#envhub)   | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Envhub](#envhub) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Envhub](#envhub) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Envhub](#envhub) | Wait on a service at a specific port to be ready |
| stdout      |                        | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard output.            |
| stderr      |                        | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |

### File

Fields of the File object:

| Name        | Arguments           | Type                | Description                              |
| ----------- | ------------------- |-------------------- | ---------------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)               | Unique identifier.                       |
| path        |                     | [String](https://spec.graphql.org/October2021/#sec-String)           | Path of the file.                        |
| zip         |                     | [File](#file-1)              | Compress the file to a zip archive.      |
| tarCzvf     |                     | [File](#file-1)              | Compress the file to a tar gzip archive. |
| unzip       |                     | [File](#file-1)               | Uncompress the zip archive.              |
| tarXzvf     |                     | [File](#file-1)               | Uncompress the tar gzip archive.         |
| md5         |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the MD5 checksum of the file.        |
| sha256      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the SHA256 checksum of the file.     |

### Flox

Fields of the Flox object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)             | Unique identifier.                  |
| withCache   |  path: `String!`, cacheId: `ID!` | [Flox](#flox-1) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Flox](#flox-1) | Add file at the specified path.            |
| withExec    |  args: `[String!]!` | [Flox](#flox-1)     | Add command to execute.             |
| withWorkdir |  path: `String!`     | [Flox](#flox-1)     | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Flox](#flox-1) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Flox](#flox-1) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Flox](#flox-1) | Wait on a service at a specific port to be ready |
| stdout      |                     | [String](https://spec.graphql.org/October2021/#sec-String)           | Get the standard output.            |
| stderr      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |

### Git

Fields of the Mise object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)! | Unique identifier.                  |
| branch      | name: String! | [Git](#git-1)! | Checkout the branch.                |
| commit      |               | [String](https://spec.graphql.org/October2021/#sec-String)! | Get the last commit hash.           |
| tree       |     | [Directory](#directory-1)! | Get the tree of the repository.      |


### Mise

Fields of the Mise object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)              | Unique identifier.                  |
| withCache   |  path: `String!`, cacheId: `ID!` | [Mise](#mise-1) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Mise](#mise-1) | Add file at the specified path.            |
| withExec    |  args: `[String!]!` | [Mise](#mise-1)     | Add command to execute.             |
| withWorkdir |  path: `String!`     | [Mise](#mise-1)     | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Mise](#mise-1) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Mise](#mise-1) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Mise](#mise-1) | Wait on a service at a specific port to be ready |
| stdout      |                     |[String](https://spec.graphql.org/October2021/#sec-String)           | Get the standard output.            |
| stderr      |                     | [String](https://spec.graphql.org/October2021/#sec-String)          | Get the standard error output.      |

### Nix

Fields of the Nix object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)              | Unique identifier.                  |
| withCache   |  path: `String!`, cacheId: `ID!` | [Nix](#nix-1) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Nix](#nix-1) | Add file at the specified path.            |
| withExec    |  args: `[String!]!` | [Nix](#nix-1) | Add command to execute.             |
| withWorkdir |  path: `String!`     | [Nix](#nix-1) | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Nix](#nix-1) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Nix](#nix-1) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Nix](#nix-1) | Wait on a service at a specific port to be ready |
| stdout      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard output.            |
| stderr      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |

### Pipeline

Fields of the Pipeline object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)               | Unique identifier.                  |
| withCache   |  path: `String!`, cacheId: `ID!` | [Pipeline](#pipeline-1) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Pipeline](#pipeline-1) | Add file at the specified path.            |
| withExec    |  args: `[String!]!` | [Pipeline](#pipeline-1) | Add command to execute.             |
| withWorkdir |  path: `String!`     | [Pipeline](#pipeline-1) | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Pipeline](#pipeline-1) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Pipeline](#pipeline-1) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Pipeline](#pipeline-1) | Wait on a service at a specific port to be ready |
| stdout      |                     | [String](https://spec.graphql.org/October2021/#sec-String)           | Get the standard output.            |
| stderr      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |

### Pixi

Fields of the Pixi object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)                | Unique identifier.                  |
| withCache   |  path: `String!`, cacheId: `ID!` | [Pixi](#pixi-1) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Pixi](#pixi-1) | Add file at the specified path.            |
| withExec    |  args: `[String!]!` | [Pixi](#pixi-1) | Add command to execute.             |
| withWorkdir |  path: `String!`     | [Pixi](#pixi-1) | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Pixi](#pixi-1) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Pixi](#pixi-1) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Pixi](#pixi-1) | Wait on a service at a specific port to be ready |
| stdout      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard output.            |
| stderr      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |

### Pkgx

Fields of the Pixi object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)                | Unique identifier.                  |
| withCache   |  path: `String!`, cacheId: `ID!` | [Pkgx](#pkgx-1) | Add Cache            |
| withFile    |  path: `String!`, fileId: `ID!` | [Pkgx](#pkgx-1) | Add file at the specified path.            |
| withExec    |  args: `[String!]!` | [Pkgx](#pkgx-1)     | Add command to execute.             |
| withWorkdir |  path: `String!`     | [Pkgx](#pkgx-1)     | Set the working directory.          |
| withEnvVariable |  name: `String!`, value: `String!`     | [Pkgx](#pkgx-1) | Add an Environment Variable          |
| withSecretVariable |  name: `String!`, secret: `ID!`     | [Pkgx](#pkgx-1) | Add an Secret Variable          |
| asService | name: `String!` | [Service](#service) | Convert into a Service |
| waitOn | port: `Int!`, timeout: `Int` | [Pkgx](#pkgx-1) | Wait on a service at a specific port to be ready |
| stdout      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard output.            |
| stderr      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |

### Secret

Fields of the Secret object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)                | Unique identifier.                  |
| plaintext  |                        | `String!`   | The value of the secret in plain text |
| name       |                       | `String!`  | The name of the secret |
| mount      |            | The mount point of the secret |

### Service

Fields of the Service:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)                | Unique identifier.                  |

### SecretManager

Fields of the SecretManager object:

| Name        | Arguments           | Type                | Description                         |
| ----------- | ------------------- |-------------------- | ----------------------------------- |
| id          |                     | [ID](https://spec.graphql.org/October2021/#sec-ID)                | Unique identifier.                  |
| getSecret   | name: `String` | `[Secret!]!` | Download secrets from the Secret Provider |
