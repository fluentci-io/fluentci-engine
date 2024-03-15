# Rust Pipeline

[![fluentci pipeline](https://img.shields.io/badge/dynamic/json?label=pkg.fluentci.io&labelColor=%23000&color=%23460cf1&url=https%3A%2F%2Fapi.fluentci.io%2Fv1%2Fpipeline%2Frust_pipeline&query=%24.version)](https://pkg.fluentci.io/rust_pipeline)
[![deno module](https://shield.deno.dev/x/rust_pipeline)](https://deno.land/x/rust_pipeline)
![deno compatibility](https://shield.deno.dev/deno/^1.37)
[![](https://jsr.io/badges/@fluentci/rust)](https://jsr.io/@fluentci/rust)
[![](https://img.shields.io/codecov/c/gh/fluent-ci-templates/rust-pipeline)](https://codecov.io/gh/fluent-ci-templates/rust-pipeline)


A ready-to-use CI/CD Pipeline for your Rust projects.

![Made with VHS](https://vhs.charm.sh/vhs-f5jk3sceXQrc55XC4fW3c.gif)

## 🚀 Usage

Run the following command in your Rust Project:

```bash
fluentci run rust_pipeline
```

Or if you want to run specific jobs:

```bash
fluentci run rust_pipeline test build
```


if you want to use it as a template:

```bash
fluentci init -t rust
```

This will create a `.fluentci` folder in your project.

Now you can run the pipeline with:

```bash
fluentci run .
```

## 🧩 Dagger Module

Use as a [Dagger](https://dagger.io) Module:

```bash
dagger install github.com/fluent-ci-templates/rust-pipeline@main
```

Call a function from the module:

```bash
dagger call clippy --src .
dagger call test --src .
dagger call llvm-cov --src .
dagger call build --src .
```

## ✨ Jobs

| Job      | Description                     |
| -------- | ------------------------------- |
| clippy   | Run Rust Clippy on your project |
| build    | Build your project              |
| test     | Run your tests                  |
| llvm_cov | Generate llvm coverage report   |

```typescript
build(
  src: string | Directory | undefined = ".",
  packageName?: string,
  target = "x86_64-unknown-linux-gnu",
  options: string[] = []
): Promise<Directory | string>

clippy(
  src: string | Directory | undefined = "."
): Promise<File | string>

test(
  src: string | Directory | undefined = ".",
  options: string[] = []
): Promise<string>

llvmCov(
  src: string | Directory | undefined = "."
): Promise<File | string>
```

## 👨‍💻 Programmatic usage

You can also use this pipeline programmatically:

```ts
import { build, test } from "jsr:@fluentci/rust";

await test();
await build();
```
