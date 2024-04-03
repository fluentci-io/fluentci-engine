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
| withExec    |  args: `[String!]!`    | [Envhub](#envhub)   | Add command to execute.             |
| withWorkdir |  path: `String!`        | [Envhub](#envhub)   | Set the working directory.          |
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
| stdout      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard output.            |
| stderr      |                     | [String](https://spec.graphql.org/October2021/#sec-String)            | Get the standard error output.      |