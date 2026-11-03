# Envhub fixture

The engine installs the pinned, SHA-256-verified Envhub GitHub release when the
CLI is absent. Envhub 0.3 initializes Home Manager even during `--version` and
ignores some internal command failures.

The engine therefore checks Home Manager initialization before invoking Envhub,
requires nonempty `home.nix` and `flake.nix`, and enables `nix-command flakes`
through child-local `NIX_CONFIG`. Existing caller settings are retained; no
system or user Nix configuration file is rewritten. Subsequent activation and
execution use the same settings.

A private, per-invocation Nix wrapper records internal nonzero exits. If Envhub
ignores one and returns success, the engine still fails the pipeline before
running dependent steps. The wrapper invokes the existing Nix executable and is
removed after the operation; it does not replace the installed executable.

`cargo test --release --locked -p fluentci-ext --lib envhub` exercises this using
fake executables only. Real Home Manager activation and the `hello` package check
run in the existing Linux GitHub Actions e2e job. They are not run locally.
