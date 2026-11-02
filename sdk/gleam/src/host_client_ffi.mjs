export function valid_id(id) {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(id);
}

// Bound decoded response bytes too: Content-Length alone cannot constrain
// chunked or compressed replies. The deadline covers headers AND the body.
async function read_body(response) {
  const maximum = 16 * 1024 * 1024;
  if (!response.body) return "";
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let total = 0, body = "";
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) return body + decoder.decode();
      total += value.byteLength;
      if (total > maximum) throw Error("response limit");
      body += decoder.decode(value, { stream: true });
    }
  } finally {
    await reader.cancel().catch(() => {});
    reader.releaseLock();
  }
}

export async function request(config, method, id, suffix, body, key, ok, error) {
  const controller = new AbortController();
  let timer, response, payload, problem = "invalid client configuration";
  try {
    const base = new URL(config.base_url);
    if (!["http:", "https:"].includes(base.protocol) || base.username || base.password ||
        base.search || base.hash || base.pathname !== "/" ||
        !Number.isSafeInteger(config.timeout_ms) || config.timeout_ms <= 0 || config.timeout_ms > 2147483647 ||
        !config.token || /[\r\n]/.test(config.token)) throw Error("configuration");
    const submission = method === "POST" && suffix === "" && id === "";
    if (!submission && !valid_id(id)) throw Error("invalid run id");
    if (key !== "" && !/^[A-Za-z0-9_-]{1,128}$/.test(key)) throw Error("invalid key");
    problem = submission ? "request failed; submission outcome may be unknown" : "request failed";
    timer = setTimeout(() => controller.abort(), config.timeout_ms);
    response = await fetch(new URL(`/v1/runs${submission ? "" : `/${id}${suffix}`}`, base), {
      method,
      headers: { Authorization: `Bearer ${config.token}`, "Content-Type": "application/json",
        ...(key === "" ? {} : { "Idempotency-Key": key }) },
      body: body === "" ? undefined : body,
      signal: controller.signal,
      redirect: "error",
    });
    payload = await read_body(response);
  } catch (_) {
    return error(controller.signal.aborted ? `${problem}; request timed out` : problem);
  } finally {
    clearTimeout(timer);
  }
  // Keep user callbacks outside the transport catch so programming failures are
  // not silently converted into network errors.
  return ok(response.status, payload, response.headers.get("Idempotency-Key") ?? "");
}
