/**
 * @module rust
 * @description This module provides a set of functions to build, test, and run clippy on a Rust project 🦀
 */

import { dag, env, Directory, DirectoryID, File } from "../../deps.ts";
import { buildRustFlags } from "./lib.ts";

export enum Job {
  clippy = "clippy",
  test = "test",
  build = "build",
  llvmCov = "llvm_cov",
  e2e = "e2e",
  typescriptE2e = "typescript_e2e",
}

export const exclude = ["target", ".git", ".devbox", ".fluentci"];

export const getDirectory = async (
  src: string | Directory | undefined = "."
) => {
  if (src instanceof Directory) {
    return src;
  }
  if (typeof src === "string") {
    try {
      const directory = dag.loadDirectoryFromID(src as DirectoryID);
      await directory.id();
      return directory;
    } catch (_) {
      return dag.host
        ? dag.host().directory(src)
        : dag.currentModule().source().directory(src);
    }
  }
  return dag.host
    ? dag.host().directory(src)
    : dag.currentModule().source().directory(src);
};

/**
 * Run clippy
 *
 * @function
 * @description Run clippy
 * @param {string | Directory | undefined} src
 * @returns {string}
 */
export async function clippy(
  src: string | Directory | undefined = "."
): Promise<File | string> {
  const context = await getDirectory(src);
  const ctr = dag
    .pipeline(Job.clippy)
    .container()
    .from("rust:1.73-bookworm")
    .withExec(["apt-get", "update"])
    .withExec(["apt-get", "install", "-y", "build-essential", "pkg-config"])
    .withExec(["rustup", "component", "add", "clippy"])
    .withExec(["cargo", "install", "clippy-sarif", "--version", "0.3.0"])
    .withExec(["cargo", "install", "sarif-fmt", "--version", "0.3.0"])
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app")
    .withMountedCache("/app/target", dag.cacheVolume("target"))
    .withMountedCache("/root/cargo/registry", dag.cacheVolume("registry"))
    .withExec([
      "sh",
      "-c",
      "cargo clippy \
        --all-features \
        --message-format=json | clippy-sarif | tee rust-clippy-results.sarif | sarif-fmt",
    ])
    .withExec(["ls", "-la", "/app"]);

  const results = await ctr.file("/app/rust-clippy-results.sarif");
  results.export("./rust-clippy-results.sarif");
  await ctr.stdout();
  return results.id();
}

/**
 * Generate llvm coverage report
 *
 * @function
 * @description Generate llvm coverage report
 * @param {string | Directory | undefined} src
 * @returns {string}
 */
export async function llvmCov(
  src: string | Directory | undefined = "."
): Promise<File | string> {
  const context = await getDirectory(src);
  const ctr = dag
    .pipeline(Job.llvmCov)
    .container()
    .from("rust:1.73-bookworm")
    .withExec(["apt-get", "update"])
    .withExec([
      "apt-get",
      "install",
      "-y",
      "build-essential",
      "wget",
      "pkg-config",
    ])
    .withExec(["rustup", "component", "add", "llvm-tools"])
    .withExec([
      "wget",
      "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.5.36/cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz",
    ])
    .withExec(["tar", "xvf", "cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz"])
    .withExec(["mv", "cargo-llvm-cov", "/usr/local/bin"])
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app")
    .withMountedCache("/app/target", dag.cacheVolume("target"))
    .withMountedCache("/root/cargo/registry", dag.cacheVolume("registry"))
    .withExec([
      "sh",
      "-c",
      "cargo llvm-cov \
        --all-features \
        --lib \
        --workspace \
        --lcov \
        --output-path \
          lcov.info",
    ])
    .withExec(["ls", "-la", "/app"]);

  const lcov = ctr.file("/app/lcov.info");
  await lcov.export("./lcov.info");
  await ctr.stdout();
  return lcov.id();
}

/**
 * Run tests
 *
 * @function
 * @description Run tests
 * @param {string | Directory | undefined} src
 * @param {string[]} options
 * @returns {string}
 */
export async function test(
  src: string | Directory | undefined = ".",
  options: string[] = []
): Promise<string> {
  const context = await getDirectory(src);
  const ctr = dag
    .pipeline(Job.test)
    .container()
    .from("rust:latest")
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app")
    .withMountedCache("/app/target", dag.cacheVolume("target"))
    .withMountedCache("/root/cargo/registry", dag.cacheVolume("registry"))
    .withExec(["cargo", "test", ...options]);

  return ctr.stdout();
}

/**
 * Build the project
 *
 * @function
 * @description Build the project
 * @param {string | Directory | undefined} src
 * @param {string} packageName
 * @param {string} target
 * @param {string[]} options
 * @returns {Promise<string>}
 */
export const build = async (src = "."): Promise<string> => {
  const rustflags = buildRustFlags();
  const context = await getDirectory(src);
  const ctr = dag
    .pipeline(Job.build)
    .container()
    .from("rust:1.76-bullseye")
    .withExec(["dpkg", "--add-architecture", "armhf"])
    .withExec(["dpkg", "--add-architecture", "arm64"])
    .withExec(["apt-get", "update"])
    .withExec([
      "apt-get",
      "install",
      "-y",
      "build-essential",
      "protobuf-compiler",
    ])
    .withExec([
      "apt-get",
      "install",
      "-y",
      "-qq",
      "gcc-arm-linux-gnueabihf",
      "libc6-armhf-cross",
      "libc6-dev-armhf-cross",
      "gcc-aarch64-linux-gnu",
      "libc6-arm64-cross",
      "libc6-dev-arm64-cross",
      "libc6-armel-cross",
      "libc6-dev-armel-cross",
      "binutils-arm-linux-gnueabi",
      "gcc-arm-linux-gnueabi",
      "libncurses5-dev",
      "bison",
      "flex",
      "libssl-dev",
      "bc",
      "pkg-config",
      "libudev-dev",
    ])
    .withExec(["mkdir", "-p", "/build/sysroot"])
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app")
    .withMountedCache("/app/target", dag.cacheVolume("target"))
    .withMountedCache("/root/cargo/registry", dag.cacheVolume("registry"))
    .withMountedCache("/assets", dag.cacheVolume("gh-release-assets"))
    .withEnvVariable("RUSTFLAGS", rustflags)
    .withEnvVariable(
      "PKG_CONFIG_ALLOW_CROSS",
      env.get("TARGET") !== "x86_64-unknown-linux-gnu" ? "1" : "0"
    )
    .withEnvVariable(
      "C_INCLUDE_PATH",
      env.get("TARGET") !== "x86_64-unknown-linux-gnu"
        ? "/build/sysroot/usr/include"
        : "/usr/include"
    )
    .withEnvVariable("TAG", env.get("TAG") || "latest")
    .withEnvVariable("TARGET", env.get("TARGET") || "x86_64-unknown-linux-gnu")
    .withExec(["sh", "-c", "rustup target add $TARGET"])
    .withExec([
      "sh",
      "-c",
      "cargo build -p fluentci-engine --release --target $TARGET",
    ])
    .withExec(["sh", "-c", "cp target/${TARGET}/release/fluentci-engine ."])
    .withExec([
      "sh",
      "-c",
      "tar czvf /assets/fluentci-engine_${TAG}_${TARGET}.tar.gz fluentci-engine",
    ])
    .withExec([
      "sh",
      "-c",
      "shasum -a 256 /assets/fluentci-engine_${TAG}_${TARGET}.tar.gz > /assets/fluentci-engine_${TAG}_${TARGET}.tar.gz.sha256",
    ])
    .withExec([
      "sh",
      "-c",
      "cp /assets/fluentci-engine_${TAG}_${TARGET}.tar.gz .",
    ])
    .withExec([
      "sh",
      "-c",
      "cp /assets/fluentci-engine_${TAG}_${TARGET}.tar.gz.sha256 .",
    ]);

  const exe = await ctr.file(
    `/app/fluentci-engine_${env.get("TAG")}_${env.get("TARGET")}.tar.gz`
  );
  await exe.export(
    `./fluentci-engine_${env.get("TAG")}_${env.get("TARGET")}.tar.gz`
  );

  const sha = await ctr.file(
    `/app/fluentci-engine_${env.get("TAG")}_${env.get("TARGET")}.tar.gz.sha256`
  );
  await sha.export(
    `./fluentci-engine_${env.get("TAG")}_${env.get("TARGET")}.tar.gz.sha256`
  );
  return ctr.stdout();
};

/**
 * Run e2e tests
 *
 * @function
 * @description Run e2e tests
 * @param {string | Directory | undefined} src
 * @param {string[]} options
 * @returns {string}
 */
export async function e2e(
  src: string | Directory | undefined = ".",
  options: string[] = []
): Promise<string> {
  const context = await getDirectory(env.get("WORK_DIR") || src);
  const engine = dag
    .container()
    .from("debian:bookworm")
    .withExec(["apt-get", "update"])
    .withExec(["apt-get", "install", "-y", "curl", "ca-certificates", "git"])
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app")
    .withFile(
      "/fluentci-engine",
      dag.host().file("./target/release/fluentci-engine")
    )
    .withEnvVariable("FLUENTCI_ENGINE_HOST", "0.0.0.0")
    .withExec(["/fluentci-engine"])
    .withExposedPort(6880)
    .asService();

  const democontext = await getDirectory("./sdk/typescript");
  const democtr = await dag
    .pipeline(Job.typescriptE2e)
    .container()
    .from("denoland/deno:1.41.1")
    .withDirectory("/app", democontext, { exclude })
    .withWorkdir("/app/demo")
    .withServiceBinding("fluentci-engine", engine)
    .sync();

  const demo = await democtr
    .withEnvVariable("FLUENTCI_SESSION_HOST", "fluentci-engine")
    .withExec(["deno", "run", "-A", "main.ts"])
    .stdout();

  console.log(demo);

  const ctr = await dag
    .pipeline(Job.e2e)
    .container()
    .from("pkgxdev/pkgx:latest")
    .withExec(["pkgx", "install", "httpie"])
    .withExec(["http", "--version"])
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app")
    .withServiceBinding("fluentci-engine", engine)
    .sync();

  const git = ctr.withExec([
    "bash",
    "-c",
    `http POST http://fluentci-engine:6880/graphql Content-Type:application/json query="$(cat git.graphql)" --ignore-stdin --pretty format`,
  ]);

  console.log(await git.stdout());

  const directory = ctr.withExec([
    "bash",
    "-c",
    `http POST http://fluentci-engine:6880/graphql Content-Type:application/json query="$(cat directory.graphql)" --ignore-stdin --pretty format`,
  ]);

  console.log(await directory.stdout());

  const pkgx = ctr.withExec([
    "bash",
    "-c",
    `http POST http://fluentci-engine:6880/graphql Content-Type:application/json query="$(cat pkgx.graphql)" --ignore-stdin --pretty format`,
  ]);

  console.log(await pkgx.stdout());

  const nix = ctr.withExec([
    "bash",
    "-c",
    `http POST http://fluentci-engine:6880/graphql Content-Type:application/json query="$(cat nix.graphql)" --ignore-stdin --pretty format`,
  ]);

  console.log(await nix.stdout());

  const flox = ctr.withExec([
    "bash",
    "-c",
    `http POST http://fluentci-engine:6880/graphql Content-Type:application/json query="$(cat flox.graphql)" --ignore-stdin --pretty format`,
  ]);

  console.log(await flox.stdout());

  const devenv = ctr.withExec([
    "bash",
    "-c",
    `http POST http://fluentci-engine:6880/graphql Content-Type:application/json query="$(cat devenv.graphql)" --ignore-stdin --pretty format`,
  ]);

  console.log(await devenv.stdout());

  const devbox = ctr.withExec([
    "bash",
    "-c",
    `http POST http://fluentci-engine:6880/graphql Content-Type:application/json query="$(cat devbox.graphql)" --ignore-stdin --pretty format`,
  ]);

  const stdout = await devbox.stdout();

  console.log(stdout);

  return stdout;
}

/**
 *  Run e2e tests for typescript sdk
 *
 * @function
 * @description Run e2e tests for typescript sdk
 * @param {string | Directory | undefined} src
 * @param {string[]} options
 * @returns {string}
 */
export async function typescriptE2e(
  src: string | Directory | undefined = ".",
  options: string[] = []
): Promise<string> {
  let context = await getDirectory("./fixtures");
  const engine = dag
    .container()
    .from("debian:bookworm")
    .withExec(["apt-get", "update"])
    .withExec(["apt-get", "install", "-y", "curl", "ca-certificates", "git"])
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app")
    .withFile(
      "/fluentci-engine",
      dag.host().file("./target/release/fluentci-engine")
    )
    .withEnvVariable("FLUENTCI_ENGINE_HOST", "0.0.0.0")
    .withExec(["/fluentci-engine"])
    .withExposedPort(6880)
    .asService();

  context = await getDirectory(env.get("WORK_DIR"));

  const ctr = await dag
    .pipeline(Job.typescriptE2e)
    .container()
    .from("denoland/deno:1.41.1")
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app/demo")
    .withServiceBinding("fluentci-engine", engine)
    .sync();

  const stdout = await ctr
    .withEnvVariable("FLUENTCI_SESSION_HOST", "fluentci-engine")
    .withExec(["deno", "run", "-A", "main.ts"])
    .stdout();

  console.log(stdout);

  return stdout;
}

export type JobExec =
  | ((src?: string | Directory | undefined) => Promise<Directory | string>)
  | ((src?: string | Directory | undefined) => Promise<File | string>)
  | ((src?: string | Directory | undefined) => Promise<string>)
  | ((src?: string) => Promise<string>);

export const runnableJobs: Record<Job, JobExec> = {
  [Job.clippy]: clippy,
  [Job.test]: test,
  [Job.e2e]: e2e,
  [Job.typescriptE2e]: typescriptE2e,
  [Job.build]: build,
  [Job.llvmCov]: llvmCov,
};

export const jobDescriptions: Record<Job, string> = {
  [Job.clippy]: "Run clippy",
  [Job.test]: "Run tests",
  [Job.e2e]: "Run e2e tests",
  [Job.typescriptE2e]: "Run e2e tests for typescript sdk",
  [Job.build]: "Build the project",
  [Job.llvmCov]: "Generate llvm coverage report",
};
