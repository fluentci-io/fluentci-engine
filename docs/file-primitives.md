# Native file primitives

These primitives require FluentCI Engine **0.6.1** or later and matching SDKs.
Engine 0.6.0 rejects the `file_operation` and `env_files` task fields.
Constructors are pure: a plan describes work; running it performs the effects.
No shell, `cp`, `find`, `chmod`, or other external program implements these tasks.

| Operation | Result in `TaskResult.file_result` |
| --- | --- |
| `read_text` | UTF-8 string |
| `read_bytes` | Array of byte integers |
| `write_text`, `write_bytes` | null |
| `copy_file`, `copy_tree`, `move` | null |
| `mkdir`, `remove` | null |
| `stat` | `{kind, size, mode}`; kind is file/directory/symlink/special |
| `exists` | Boolean; inspects the link itself, including dangling links |
| `list` | Sorted immediate entry names |
| `glob` | Sorted workspace-relative matches |
| `hash` | SHA-256 hexadecimal string |
| `chmod`, `symlink` | null |

Serialized query results are bounded to 8 MiB. Query results are also written as a JSON document to the task's stdout log.
Filesystem tasks have no process exit code (`exit_code: null`). Their failures
identify the operation, paths, and OS error in the result, journal and progress
output; the CLI exits nonzero. Write payloads are omitted from error context.
Process jobs that consume env files continue to forward their actual exit codes.

## Composition

File tasks share the executor's job limit, dependencies, named resource leases,
conditions, retries, timeouts, cancellation, tracing and artifact publication.
Workers are cancelled and joined before leases are released or dependents start.
Retries without an exit-code filter can retry native failures; an exit-code filter
applies only to process failures. Constructors below accept explicit flags rather
than silently overwriting or recursively deleting content.

PureScript (`FluentCI.Host.Files`):

```purescript
let
  directory = Files.mkdir "directory" "build" true "."
  write = Files.writeText "write" "build/version.txt" "0.6.1" false "."
    # Plan.after [directory]
    # Plan.resource "checkout"
  digest = Files.hash "digest" "build/version.txt" "."
    # Plan.after [write]
  build = Plan.task "build" "cargo" "."
    # Plan.withArgs ["build", "--release", "--locked"]
    # Plan.envFile "ci.env"
```

Python (`from fluentci import files as F`):

```python
write = F.write_bytes("write", "artifact.bin", b"\x00\xff", overwrite=False)
build = F.env_file(Task("build", "cargo", ("build", "--release", "--locked")), "ci.env")
```

| SDK | Module | Naming |
| --- | --- | --- |
| Rust | `fluentci_executor::primitives` | snake_case; `move_path` |
| PureScript | `FluentCI.Host.Files`, `Host.Plan.envFile` | camelCase |
| Python | `fluentci.files` | snake_case; `list_entries` |
| TypeScript | `src/files.ts` | camelCase |
| ReScript | `Files` | camelCase |
| Gleam | `fluentci/host/files` | snake_case; `list_entries` |
| Elixir | `FluentCI.Files` | snake_case |

## Filesystem contracts

- Paths stay relative to task `cwd`. Absolute paths, `..`, and operations that
  replace/remove `cwd` itself are rejected before any task runs. `stat`, `list`,
  and `exists` can inspect `.`. Constructors do not inspect the filesystem.
- Parents cannot be symlinks. Reads, copies, hashing and chmod reject final
  symlinks and special files. `stat`, `exists`, `move`, and `remove` inspect or
  operate on a final link itself. Recursive copies reject links and special files;
  recursive removal unlinks links without traversing them.
- Symlink targets are workspace-relative and stored relative to the link's parent.
  Symlinks currently require Unix, matching the executor's supported hosts.
- Reads default to 1 MiB and accept an explicit positive limit up to 1 MiB.
  `read_text` rejects invalid UTF-8. Bytes remain exact integers 0–255.
- Writes use a temporary file in the destination directory, sync it, and publish
  atomically. Overwrite defaults to false. Parent directories must already exist.
- File copies preserve permissions and publish each file atomically. A new tree
  is staged before publication. With overwrite enabled, copying into an existing
  directory merges entries: it does not delete extra files or roll back earlier
  copied files if a later copy fails. Source/destination overlap is rejected.
- Move uses native rename, so cross-filesystem moves fail with the OS error.
  Overwrite is explicit. Concurrent writers must share a named resource lease;
  path validation is not a sandbox against hostile concurrent filesystem changes.
- `mkdir` has an explicit `parents` flag. `remove` has explicit `recursive` and
  `missing_ok` flags. Cancellation of recursive mutation can leave partial work.
- Glob supports `*`, `?`, and `**` path segments, including dotfiles. It does not
  follow directory links. Lists/globs are sorted, require UTF-8 names, and stop
  after 10,000 returned entries. Bracket expressions and backslash escapes are
  rejected rather than interpreted differently across operating systems.
- `chmod` accepts permission bits 0000–0777. Ownership changes and special mode
  bits are not part of this primitive.

## Caching

Native operation identity, including literal write contents and options, joins the
existing cache key. File queries declare consumed paths as inputs; copies declare
source inputs and destination outputs; writes/mkdir declare destination outputs.
Query-only actions may have an empty output list: verified cached stdout restores
the structured query result. Glob caching requires a literal directory prefix
covered by inputs. Move, remove, chmod and symlink cannot be action cached because
restoring files does not reproduce their destructive/link semantics.

The usual cache contract applies: no undeclared inputs, outputs or external
effects. Named resources coordinate shared writable paths across runs.

## Env files

`envFile` / `env_file` appends a workspace-relative dotenv path to a task. Files
are read just before task execution/cache preparation. Later files override
earlier files; explicit task environment entries override all file values;
explicit output bindings override the resulting environment. The resolved values
participate in the cache identity, including when an env file changes.

Supported grammar:

```dotenv
# Comments and blank lines
export VERSION='literal value'
MODE=release # Trailing comment
MESSAGE="line one\nline two"
EMPTY=
```

Keys use `[A-Za-z_][A-Za-z0-9_]*`. Single quotes preserve literal contents;
double quotes support `\n`, `\r`, `\t`, `\"` and `\\`. Unquoted inline comments
start at `#` preceded by whitespace. Multiline quoted values, interpolation and
shell evaluation are deliberately unsupported: `$(command)` and `$NAME` remain
literal strings. Files must be regular UTF-8 files no larger than 1 MiB.

Loading a file never prints its values. Parse errors contain the path and line
number, not the rejected line. Commands can still print their own environment;
use the existing secret facilities for secret injection/redaction.
