
![Cover](./.github/assets/api.png)

# FluentCI Engine

[![ci](https://github.com/fluentci-io/fluentci-engine/actions/workflows/ci.yml/badge.svg)](https://github.com/fluentci-io/fluentci-engine/actions/workflows/ci.yml)
[![discord](https://img.shields.io/discord/1132020671262773358?label=discord&logo=discord&color=5865F2)](https://discord.gg/V4U6dPskKc)

FluentCI Engine is a programmable CI/CD engine that is designed to be simple, flexible, and easy to use. It is supposed to run on the host machine without containerization or virtualization, and it is designed to be used with [Nix](https://nixos.org), [Pkgx](https://pkgx.sh), [Devbox](https://www.jetpack.io/devbox/), [Flox](https://flox.dev), and [Devenv](https://devenv.sh).

> [!NOTE]
> **Project Status: 🐲 Unstable, alpha-ish quality.**
> This project is still in the early stages of development, 
> and it is not yet ready for production use. 
> It is not feature-complete, and it is not yet stable. Use at your own risk.

## ✨ Features

- [x] Simple and easy to use
- [x] Flexible
- [x] No containerization or virtualization
- [x] Built-in support for Nix, Pkgx, Devbox, Flox and Devenv
- [ ] Cache support
- [x] SDK for writing pipelines in TypeScript, see [@fluentci/sdk](./sdk/typescript)
- [x] GraphQL API

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/fluentci-io/fluentci-engine.git
# Go to the project directory
cd fluentci-engine
# Install dependencies
nix develop
cargo run -p fluentci-engine
# Open the browser and go to http://localhost:6880/graphiql
# See ./fixtures for some GraphQL queries examples
```

> [!TIP]
> Quickly setup Nix on your machine with [DeterminateSystems Nix installer](https://github.com/DeterminateSystems/nix-installer)