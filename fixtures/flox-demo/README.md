# Flox plugin fixture

Provision native Flox before running this fixture, using the packages documented
at https://flox.dev/download/. CI pins Flox 1.19.3 using its official install
action. The extension verifies the installed executable and stops on setup or
initialization failures. It no longer installs Nix or changes `/etc/nix/nix.conf`.

This Linux x86_64 smoke fixture uses manifest format 1. Its obsolete format-0
lockfile has been removed. First activation resolves jq through the Flox catalog;
this fixture therefore requires network access and is not a pinned toolchain.
A successful native CI run is needed to validate catalog resolution and activation.
