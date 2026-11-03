# Flox plugin fixture

If Flox is missing, the extension downloads https://get.flox.dev into a private
temporary directory and runs the official native installer. It defaults to
Flox 1.19.3; set `FLOX_VERSION` to select another release. Installation requires
root or passwordless sudo, and `/usr/bin/curl`. Only Linux and macOS are supported.

Installer downloads have connection/total timeouts, require HTTPS, and must
succeed before the script runs. The installer child prefers system utilities to
avoid stale toolchain curl binaries. Installer metrics are disabled with `FLOX_DISABLE_METRICS=true` (the CLI expects
a boolean, not `1`). Existing
working installations are reused; a broken installed CLI is reported instead of
being overwritten automatically. Installation success is followed by a version
check. Native Flox must be discoverable on the caller's PATH after installation.

CI separately tests fresh automatic provisioning on a disposable Ubuntu runner.
The legacy Wasm pipeline pins Flox via its official install action. No local Flox
installation is performed during fixture unit tests. The extension no longer
bootstraps a separate Nix or writes system trust/cache settings itself.

This Linux x86_64 smoke fixture uses manifest format 1. Its obsolete format-0
lockfile has been removed. First activation resolves jq through the Flox catalog;
this fixture therefore requires network access and is not a pinned toolchain.
A successful native CI run is needed to validate catalog resolution and activation.
