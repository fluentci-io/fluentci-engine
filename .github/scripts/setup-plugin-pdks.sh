#!/usr/bin/env bash
# CI-only Linux x86_64 guest compilers; no system-wide installation.
set -euo pipefail
if [[ "$(/usr/bin/uname -s)-$(/usr/bin/uname -m)" != Linux-x86_64 ]]; then
  echo 'This CI compiler setup supports Linux x86_64 only' >&2
  exit 1
fi
root="${RUNNER_TEMP:?}/fluentci-guest-compilers"
mkdir -p "$root/bin"
cd "$root"
download() {
  local url="$1" file="$2" digest="$3"
  /usr/bin/curl --fail --location --retry 3 --proto '=https' "$url" -o "$file"
  printf '%s  %s\n' "$digest" "$file" | /usr/bin/sha256sum --check --strict
}
download https://github.com/extism/python-pdk/releases/download/v0.1.5/extism-py-x86_64-linux-v0.1.5.tar.gz python.tar.gz 24fa8a4a2f2fe25e3a95a4dc1dbd10a4c30e97697102fec45c59bfc743b2d9d4
/usr/bin/tar -xzf python.tar.gz
download https://github.com/WebAssembly/binaryen/releases/download/version_116/binaryen-version_116-x86_64-linux.tar.gz binaryen.tar.gz c55b74f3109cdae97490faf089b0286d3bba926bb6ea5ed00c8c784fc53718fd
/usr/bin/tar -xzf binaryen.tar.gz
download https://github.com/extism/js-pdk/releases/download/v1.7.0/extism-js-x86_64-linux-v1.7.0.gz javascript.gz 63b72da2f5e88655522dc21477de549f238a2f40546a69ce4e0fce7e78654035
/usr/bin/gzip -dc javascript.gz > bin/extism-js
chmod +x bin/extism-js
printf '%s\n' "$root/bin" "$root/extism-py/bin" "$root/binaryen-version_116/bin" >> "${GITHUB_PATH:?}"
printf 'EXTISM_PYTHON_WASI_DEPS_DIR=%s\n' "$root/extism-py/share/extism-py" >> "${GITHUB_ENV:?}"
