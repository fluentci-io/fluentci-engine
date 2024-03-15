import {
  build,
  test,
} from "https://pkg.fluentci.io/rust_pipeline@v0.8.4/mod.ts";

await test();
await build();
