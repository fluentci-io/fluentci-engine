import { query } from "./src/introspect.ts";

async function main() {
  const body = JSON.stringify({
    query,
  });

  const FLUENTCI_ENDPOINT =
    Deno.env.get("FLUENTCI_ENDPOINT") || "http://localhost:6880/graphql";
  const data = await fetch(FLUENTCI_ENDPOINT, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body,
  })
    .then((res) => res.json())
    .catch((err) => console.error(err));

  console.log(data);

  Deno.writeTextFile("schema.json", JSON.stringify(data, null, 2));
}

// Learn more at https://deno.land/manual/examples/module_metadata#concepts
if (import.meta.main) {
  await main();
}
