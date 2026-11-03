# Python projects with uv

This Wasm module composes two steps through the shared host executor: set up uv
when missing, then run a Python project. An existing uv on PATH is reused;
otherwise the official pinned 0.13.7 installer downloads a binary. It uses native
OS curl and an unmanaged installation, without modifying shell profiles.
See [uv installer options](https://docs.astral.sh/uv/reference/installer/).

Build the module in release mode:

```sh
cargo build --release --locked --target wasm32-unknown-unknown -p uv
```

Set up uv alone:

```sh
fluentci-engine call -m target/wasm32-unknown-unknown/release/uv.wasm setup
```

Run the included project, passing arguments as a JSON array:

```sh
fluentci-engine call -m target/wasm32-unknown-unknown/release/uv.wasm run \
  '["fixtures/uv-demo", "main.py", "hello world", "世界"]'
```

The first value selects the project directory; the second selects the Python
script. Remaining values are literal script arguments, including spaces, quotes,
and shell metacharacters. The project is run with `uv run --project . python ...`.
uv creates the project's virtual environment and resolves its dependencies.

`UV_UNMANAGED_INSTALL` selects the installation directory (default:
`$HOME/.local/bin`). `UV_CACHE_DIR` and `UV_PYTHON` configure uv's cache and Python
interpreter. The example targets Linux/macOS, where the standalone installer is
supported. No Python packages are required by the demo project.
