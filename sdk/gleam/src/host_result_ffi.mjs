export function parse_json(text, ok, error) {
  let data;
  try { data = JSON.parse(text); } catch { return error(); }
  return ok(data);
}
