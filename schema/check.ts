// Validate with XTP's official parser/normalizer, without cloud authentication.
import { parse } from "@dylibso/xtp-bindgen";
const root = new URL("./", import.meta.url);
const source = Deno.readTextFileSync(new URL("host.json", root));
const canonical = JSON.parse(source);
const expectedImports = Object.keys(canonical.imports).sort();
const host = parse(source);
function checkImports(schema: ReturnType<typeof parse>, name: string) {
  const actual = Object.keys(schema.imports ?? {}).sort();
  if (JSON.stringify(actual) !== JSON.stringify(expectedImports)) {
    throw new Error(`Host imports differ from the canonical schema: ${name}`);
  }
}
for (const name of ["with_exec", "stdout", "execute_plan"]) {
  if (!expectedImports.includes(name)) {
    throw new Error(`Required host import is missing: ${name}`);
  }
}
checkImports(host, "host.json");
let plugins = 0;
for (const entry of Deno.readDirSync(new URL("generated/", root))) {
  if (!entry.name.endsWith(".json")) continue;
  const schema = parse(
    Deno.readTextFileSync(new URL(`generated/${entry.name}`, root)),
  );
  checkImports(schema, entry.name);
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
