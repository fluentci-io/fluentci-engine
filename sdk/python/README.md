# FluentCI Python PDK

Typed Python plugins that compile to Wasm and call FluentCI's exposed PDK functions.
Pipeline and service builders are immutable; `stdout()` and `stderr()` execute.
No Extism host runtime is embedded.

```python
import extism
from fluentci import pipeline

@extism.plugin_fn
def greet() -> None:
    result = pipeline("greet").exec("/usr/bin/printf", "%s", extism.input_str()).stdout()
    extism.output_str(result)
```

Compose `with_workdir`, `with_env`, `with_cache`, `using(Provider.NIX, impure=True)`,
and `with_service`. `exec(program, *arguments)` preserves literal argument boundaries;
`shell(command)` explicitly accepts shell syntax. `file(path).sha256()` and
`get_env(key)` are explicit guest effects.

## Build with uv

Install [extism-py v0.1.5](https://github.com/extism/python-pdk) and Binaryen
(`wasm-merge`, `wasm-opt`) as build tools. Do not install the PyPI `extism` host SDK:
extism-py supplies the guest module inside Wasm. uv manages dependencies, while
extism-py compiles the guest.

```sh
cd sdk/python
uv sync --locked
uv run mypy
uv run python -m unittest discover -s tests -v
uv run fluentci-plugin demo/plugin.py -o dist/plugin.wasm
fluentci-engine call -m dist/plugin.wasm greet 'hello from Python'
```

Use `fluentci-plugin`, rather than compiling the source directly: it generates an
entry file containing all canonical guest ABI imports required by the compiler's
entry-only AST scanner. The generated `.entry.py` is retained next to the Wasm.
The SDK ships `py.typed` and strict mypy guest stubs. CPython 3.12 is embedded by
the compiler; dependencies must be pure Python. Native extensions and arbitrary
host OS access are unavailable inside the guest. See the plugin PDK CI workflow
for isolated compiler setup and real engine validation.

## Parallel matrix jobs (unreleased)

See the [matrix guide](../../docs/matrix.md) and the `matrix` export in this SDK's
plugin example. The API expands named typed cases into native tasks and submits
one complete plan via `execute_plan`. It supports bounded concurrency, independent
case caching/logs, prerequisites, joins, failure reports and owning-run cancellation.
Use matching engine/SDK source builds; engine 0.6.0 does not contain this new import.
