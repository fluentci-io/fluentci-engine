const esbuild = require("esbuild");

esbuild.build({
  alias: { "@fluentci/sdk/plugin": "../src/plugin.ts" },
  entryPoints: ["src/index.ts", "src/hash.ts", "src/module.ts", "src/sys.ts"],
  outdir: "dist",
  bundle: true,
  sourcemap: true,
  minify: false, // might want to use true for production build
  format: "cjs", // needs to be CJS for now
  target: ["es2020"], // don't go over es2020 because quickjs doesn't support it
});
