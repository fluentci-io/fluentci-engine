// Assert both services are usable, rather than sleeping for an arbitrary period.
for (const [index, portFile] of Deno.args.entries()) {
  const port = Number(await Deno.readTextFile(portFile));
  const response = await fetch(`http://127.0.0.1:${port}/health`);
  const body = await response.text();
  if (!response.ok || body !== `service-${index}`) {
    throw new Error(
      `service ${index} is unhealthy: ${response.status} ${body}`,
    );
  }
}
console.log("services healthy");
