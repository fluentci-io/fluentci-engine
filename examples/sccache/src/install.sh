set -eu
if ! command -v sccache >/dev/null 2>&1; then
  case "$(/usr/bin/uname -s):$(/usr/bin/uname -m)" in
    Linux:x86_64) target=x86_64-unknown-linux-musl; checksum=45f1447fbe231e3037bde351ef70677dd212216c8d62ae7ca409fecc4d6acc89 ;;
    Linux:aarch64) target=aarch64-unknown-linux-musl; checksum=2b3284d5da3b46a47dc4229e75bb7b88ac4aa99c8d754fb7d2f84997e5a4354a ;;
    Darwin:x86_64) target=x86_64-apple-darwin; checksum=1dade83cc49eeb42337565eccd534b05982820a8e44b851bd7937467a18c7aef ;;
    Darwin:arm64) target=aarch64-apple-darwin; checksum=308184519b646f5125289e8515b36f6ca65a13a041923994aebe702348674e8e ;;
    *) echo 'sccache provisioning supports Linux/macOS x86_64 and arm64' >&2; exit 1 ;;
  esac
  mkdir -p "$SCCACHE_INSTALL_DIR"
  stage=$(mktemp -d "$SCCACHE_INSTALL_DIR/.sccache-XXXXXX")
  trap 'rm -rf "$stage"' EXIT
  archive="sccache-v0.18.0-$target"
  /usr/bin/curl --fail --silent --show-error --location --retry 3 \
    "https://github.com/mozilla/sccache/releases/download/v0.18.0/$archive.tar.gz" --output "$stage/archive.tar.gz"
  if [ -x /usr/bin/sha256sum ]; then
    actual=$(/usr/bin/sha256sum "$stage/archive.tar.gz")
  else
    actual=$(/usr/bin/shasum -a 256 "$stage/archive.tar.gz")
  fi
  test "${actual%% *}" = "$checksum" || { echo 'sccache checksum mismatch' >&2; exit 1; }
  /usr/bin/tar -xzf "$stage/archive.tar.gz" -C "$stage" "$archive/sccache"
  chmod 755 "$stage/$archive/sccache"
  mv "$stage/$archive/sccache" "$SCCACHE_INSTALL_DIR/sccache"
fi
sccache --version
