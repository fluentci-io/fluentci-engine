// Foreground loopback service: no ICMP privileges or external network required.
const [name, portFile] = Deno.args;
if (!name || !portFile) throw new Error("expected service name and port file");
const server = Deno.serve({
  hostname: "127.0.0.1",
  port: 0,
  onListen({ port }) {
    Deno.writeTextFileSync(portFile, String(port));
  },
}, () => new Response(name));
await server.finished;
