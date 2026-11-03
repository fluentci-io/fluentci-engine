/** Pure native-plan matrices. run() calls execute_plan once for the whole plan. */
export type Case<A> = Readonly<{ key: string; value: A }>;
export type Task = Readonly<{
  id: string; program: string; args: readonly string[]; cwd: string;
  env: Readonly<Record<string, string>>; needs: readonly string[];
  timeout_ms?: number;
  cache?: Readonly<{ inputs: readonly string[]; outputs: readonly string[]; toolchain: string }>;
}>;
export type Plan = Readonly<{ version: 1; tasks: readonly Task[]; jobs: number; inherit_env: false; fail_fast: boolean; cache_dir?: string }>;
export type TaskResult = Readonly<{ id: string; status: string; exit_code: number | null; stdout: string; stderr: string; error: string | null }>;
export type RunResult = Readonly<{ outcome: "succeeded" | "failed" | "cancelled"; tasks: readonly TaskResult[] }>;
const component = (value: string) => encodeURIComponent(value).replace(/[!'()*]/g, char => "%" + char.charCodeAt(0).toString(16).toUpperCase());
export const named = <A>(name: string, value: A): Case<A> => Object.freeze({ key: component(name), value });
export const axis = (name: string, values: readonly string[]): readonly Case<string>[] => Object.freeze(values.map(value => Object.freeze({ key: `${component(name)}=${component(value)}`, value })));
export function product<A, B>(left: readonly Case<A>[], right: readonly Case<B>[]): readonly Case<readonly [A, B]>[] {
  if (left.length * right.length > 4096) throw Error("matrix expansion exceeds 4096 cases");
  return Object.freeze(left.flatMap(a => right.map(b => Object.freeze({ key: `${a.key},${b.key}`, value: Object.freeze([a.value, b.value]) as readonly [A, B] }))));
}
export const include = <A>(cases: readonly Case<A>[], extra: readonly Case<A>[]): readonly Case<A>[] => Object.freeze([...cases, ...extra]);
export const exclude = <A>(cases: readonly Case<A>[], predicate: (value: A) => boolean): readonly Case<A>[] => Object.freeze(cases.filter(item => !predicate(item.value)));
export const task = (id: string, program: string, args: readonly string[], cwd = "."): Task => Object.freeze({ id, program, args: Object.freeze([...args]), cwd, env: Object.freeze({}), needs: Object.freeze([]) });
export function matrix<A>(name: string, cases: readonly Case<A>[], build: (value: A) => Task): readonly Task[] {
  if (!/^[A-Za-z0-9_.-]{1,64}$/.test(name)) throw Error("invalid matrix name");
  if (cases.length > 4096) throw Error("matrix expansion exceeds 4096 cases");
  const keys = new Set<string>();
  for (const item of cases) {
    if (!item.key || item.key.includes("\0") || keys.has(item.key)) throw Error("matrix cases require unique, nonempty keys without NUL");
    keys.add(item.key);
  }
  return Object.freeze(cases.map(item => Object.freeze({ ...build(item.value), id: `${name}[${item.key}]` })));
}
export const after = (parents: readonly Task[], value: Task): Task => Object.freeze({ ...value, needs: Object.freeze([...new Set([...value.needs, ...parents.map(parent => parent.id)])]) });
export const parallel = (tasks: readonly Task[], jobs: number): Plan => Object.freeze({ version: 1, inherit_env: false, tasks: Object.freeze([...tasks]), jobs, fail_fast: true });
export function compile(plan: Plan): string {
  if (!Number.isInteger(plan.jobs) || plan.jobs < 1) throw Error("jobs must be positive");
  if (new Set(plan.tasks.map(task => task.id)).size !== plan.tasks.length) throw Error("duplicate task IDs");
  return JSON.stringify(plan);
}
interface GuestMemory { offset: bigint; readString(): string }
declare const Host: { getFunctions(): { execute_plan?: (ptr: bigint) => bigint } };
declare const Memory: { fromString(value: string): GuestMemory; find(ptr: bigint): GuestMemory };
export function run(plan: Plan, runDir: string): RunResult {
  const fn = Host.getFunctions().execute_plan;
  if (!fn) throw Error("This engine does not expose execute_plan; upgrade the engine");
  const request = JSON.stringify({ plan: JSON.parse(compile(plan)), run_dir: runDir });
  const report = JSON.parse(Memory.find(fn(Memory.fromString(request).offset)).readString());
  if (!["succeeded", "failed", "cancelled"].includes(report?.outcome) || !Array.isArray(report.tasks)) throw Error("invalid terminal plan report");
  return report as RunResult;
}
