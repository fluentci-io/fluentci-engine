// Validate with XTP's official parser/normalizer, without cloud authentication.
import { parse } from "@dylibso/xtp-bindgen";
const root = new URL("./", import.meta.url);
const host = parse(Deno.readTextFileSync(new URL("host.json", root)));
if (Object.keys(host.imports ?? {}).length !== 52) {
  throw new Error("Unexpected host import count");
}
let plugins = 0;
for (const entry of Deno.readDirSync(new URL("generated/", root))) {
  if (!entry.name.endsWith(".json")) continue;
  const schema = parse(
    Deno.readTextFileSync(new URL(`generated/${entry.name}`, root)),
  );
  if (Object.keys(schema.exports ?? {}).length === 0) {
    throw new Error(`Empty plugin: ${entry.name}`);
  }
  plugins++;
}
console.log(
  `XTP validated ${
    Object.keys(host.imports ?? {}).length
  } host imports and ${plugins} plugin schemas`,
);
