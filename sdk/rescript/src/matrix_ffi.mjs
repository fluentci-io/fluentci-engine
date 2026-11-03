const frozen = Object.freeze;
const component = value => encodeURIComponent(value).replace(/[!'()*]/g, char => "%" + char.charCodeAt(0).toString(16).toUpperCase());
const result = (ok, error, f) => { try { return ok(f()); } catch (e) { return error(e instanceof Error ? e.message : String(e)); } };
export const named = (name, value) => frozen({ key: component(name), value });
export const axis = (name, values) => frozen(values.map(value => frozen({ key: `${component(name)}=${component(value)}`, value })));
export const include = (cases, extra) => frozen([...cases, ...extra]);
export const exclude = (cases, predicate) => frozen(cases.filter(c => !predicate(c.value)));
export const product = (left, right, ok, error) => result(ok, error, () => {
  if (left.length * right.length > 4096) throw Error("matrix expansion exceeds 4096 cases");
  return frozen(left.flatMap(a => right.map(b => frozen({ key: `${a.key},${b.key}`, value: frozen([a.value, b.value]) }))));
});
export const task = (id, program, args, cwd) => frozen({ id, program, args: frozen([...args]), cwd, env: frozen({}), needs: frozen([]) });
export const withEnv = (task, key, value) => frozen({ ...task, env: frozen({ ...task.env, [key]: value }) });
export const after = (task, parents) => frozen({ ...task, needs: frozen([...new Set([...task.needs, ...parents.map(parent => parent.id)])]) });
export const parallel = (tasks, jobs) => frozen({ version: 1, inherit_env: false, tasks: frozen([...tasks]), jobs, fail_fast: true });
export const withFailFast = (plan, fail_fast) => frozen({ ...plan, fail_fast });
export const compile = plan => {
  if (!Number.isInteger(plan.jobs) || plan.jobs < 1) throw Error("jobs must be positive");
  if (new Set(plan.tasks.map(task => task.id)).size !== plan.tasks.length) throw Error("duplicate task IDs");
  return JSON.stringify(plan);
};
export const matrix = (name, cases, build, ok, error) => result(ok, error, () => {
  if (!/^[A-Za-z0-9_.-]{1,64}$/.test(name)) throw Error("invalid matrix name");
  if (cases.length > 4096) throw Error("matrix expansion exceeds 4096 cases");
  const keys = new Set();
  for (const c of cases) {
    if (!c.key || c.key.includes("\0") || keys.has(c.key)) throw Error("matrix cases require unique, nonempty keys without NUL");
    keys.add(c.key);
  }
  return frozen(cases.map(c => frozen({ ...build(c.value), id: `${name}[${c.key}]` })));
});
export const run = (plan, directory, ok, error) => result(ok, error, () => {
  const fn = Host.getFunctions().execute_plan;
  if (!fn) throw Error("This engine does not expose execute_plan; upgrade the engine");
  const request = JSON.stringify({ plan: JSON.parse(compile(plan)), run_dir: directory });
  const report = JSON.parse(Memory.find(fn(Memory.fromString(request).offset)).readString());
  if (!["succeeded", "failed", "cancelled"].includes(report?.outcome) || !Array.isArray(report.tasks)) throw Error("invalid terminal plan report");
  return report;
});

export const cached = (task, inputs, outputs, toolchain) => frozen({ ...task, cache: frozen({ inputs: frozen([...inputs]), outputs: frozen([...outputs]), toolchain }) });

export const withCacheDirectory = (plan, cache_dir) => frozen({ ...plan, cache_dir });
