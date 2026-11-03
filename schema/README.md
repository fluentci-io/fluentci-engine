# Extism plugins and XTP schemas

The engine uses Extism **1.41.0** and the Rust PDK **1.4.1**, pinned once in the
workspace dependencies. Plugins target `wasm32-unknown-unknown`; the engine also
enables WASI for plugins that require it.

`host.json` is the canonical XTP `v1-draft` description of the engine's 52 host
imports and their JSON/text payloads. `plugins.json` defines example exports;
`pdk-modules.json` selects imports for the existing fluent Rust API. Change these
sources and regenerate:

```sh
python3 scripts/xtp_codegen.py
python3 scripts/xtp_codegen.py --check
deno task --config schema/deno.json check
```

Python 3.9+ and the pinned Rust toolchain's `rustfmt` are required. The generator
writes the existing PDK's import declarations, one shared host registration
function for direct **and nested** calls, and standalone XTP schemas in
`generated/`. It rejects missing or duplicate host implementations and mismatched
input/output pointer counts. `--check` fails on stale generated files. The Deno
check uses the official `@dylibso/xtp-bindgen@1.0.0-rc.17` parser and normalizer;
its dependencies are locked in `deno.lock`. CI runs both checks.

The fluent Rust PDK remains source compatible. The schema preserves the existing
wire protocol; it does not replace the GraphQL schemas used by the other SDKs.

## Generate a plugin with XTP

Each `generated/<example>.json` contains both imports and exports, ready for
`xtp plugin init`. No XTP cloud account or extension point is needed for local
code generation. For example, from the repository root:

```sh
xtp plugin init \
  --schema-file "$PWD/schema/generated/xtp-smoke.json" \
  --template https://github.com/dylibso/xtp-rust-bindgen/releases/download/v0.0.1-rc7/bundle.zip \
  --path /tmp/fluentci-xtp-smoke --name xtp-smoke --yes
```

Review the CLI's validation messages: the tested CLI can continue after schema
errors when `--yes` is supplied. The checked-in smoke plugin's `src/pdk.rs` is the
output of that official Rust template, formatted by `cargo fmt`. To refresh it,
copy `/tmp/fluentci-xtp-smoke/src/pdk.rs` to
`examples/xtp-smoke/src/pdk.rs`, then run `cargo fmt --all`. Its Cargo manifest
uses the workspace PDK pin rather than the template's older default.

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --locked -p fluentci-engine
cargo build --release --locked --target wasm32-unknown-unknown \
  -p env -p hash -p module -p xtp-smoke
python3 tests/e2e_wasm.py -v
```

These end-to-end tests execute real plugins under the release runtime, including
legacy imports, XTP-generated JSON exports, Unicode and whitespace, nested host
calls, file hashing, invalid JSON, and missing exports. They install no package
managers. CI also compiles every Rust example with the upgraded PDK.

## Compatibility details

XTP v1 is still a draft and cannot express every existing Rust constraint:

- `set_envs` retains UTF-8 JSON bytes representing an array of `[name, value]`
  string pairs. XTP describes it as a binary buffer because nested tuple arrays
  are unsupported. Generated clients must JSON-encode the pairs before calling
  this import; `xtp-smoke` demonstrates that conversion.
- Secret providers retain the externally tagged Rust enum encoding, represented
  by optional nullable variant properties. Exactly one variant must be non-null;
  Rust deserialization enforces the enum constraint.
- Port values remain unsigned 32-bit integers at the host boundary. Their schema
  description states this range; the XTP draft models the list using `int64`.

The existing Rust types remain responsible for these payload constraints.
