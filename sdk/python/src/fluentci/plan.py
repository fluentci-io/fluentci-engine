"""Native task/plan values. run() is a guest PDK effect, not an embedded host."""
from __future__ import annotations
from dataclasses import dataclass, replace
import json
from typing import Any, Literal

@dataclass(frozen=True, slots=True)
class Cache:
    inputs: tuple[str, ...]
    outputs: tuple[str, ...]
    toolchain: str

@dataclass(frozen=True, slots=True)
class Task:
    id: str
    program: str
    args: tuple[str, ...] = ()
    cwd: str = "."
    env: tuple[tuple[str, str], ...] = ()
    needs: tuple[str, ...] = ()
    timeout_ms: int | None = None
    cache: Cache | None = None

    def with_env(self, key: str, value: str) -> Task:
        return replace(self, env=(*self.env, (key, value)))

    def to_dict(self) -> dict[str, Any]:
        value: dict[str, Any] = {"id": self.id, "program": self.program, "args": list(self.args),
                                 "cwd": self.cwd, "env": dict(self.env), "needs": list(self.needs)}
        if self.timeout_ms is not None:
            value["timeout_ms"] = self.timeout_ms
        if self.cache is not None:
            value["cache"] = {"inputs": list(self.cache.inputs), "outputs": list(self.cache.outputs), "toolchain": self.cache.toolchain}
        return value

@dataclass(frozen=True, slots=True)
class TaskResult:
    id: str
    status: Literal["succeeded", "failed", "skipped", "cancelled", "timed_out"]
    exit_code: int | None
    stdout: str
    stderr: str
    error: str | None

@dataclass(frozen=True, slots=True)
class RunResult:
    outcome: Literal["succeeded", "failed", "cancelled"]
    tasks: tuple[TaskResult, ...]

    @property
    def succeeded(self) -> bool:
        return self.outcome == "succeeded"

@dataclass(frozen=True, slots=True)
class Plan:
    tasks: tuple[Task, ...]
    jobs: int = 1
    fail_fast: bool = True
    cache_dir: str | None = None

    def to_json(self) -> str:
        if self.jobs < 1:
            raise ValueError("jobs must be positive")
        ids = [task.id for task in self.tasks]
        if len(ids) != len(set(ids)) or any(not key for key in ids):
            raise ValueError("tasks require unique, nonempty IDs")
        value: dict[str, Any] = {"version": 1, "inherit_env": False, "jobs": self.jobs,
                                 "fail_fast": self.fail_fast, "tasks": [task.to_dict() for task in self.tasks]}
        if self.cache_dir is not None:
            value["cache_dir"] = self.cache_dir
        return json.dumps(value)

    def run(self, run_dir: str) -> RunResult:
        from . import _imports as api
        request = json.dumps({"plan": json.loads(self.to_json()), "run_dir": run_dir})
        raw: Any = json.loads(api.execute_plan(request))
        if not isinstance(raw, dict) or raw.get("outcome") not in {"succeeded", "failed", "cancelled"} or not isinstance(raw.get("tasks"), list):
            raise RuntimeError("invalid terminal plan report")
        tasks: list[TaskResult] = []
        for row in raw["tasks"]:
            if not isinstance(row, dict) or row.get("status") not in {"succeeded", "failed", "skipped", "cancelled", "timed_out"}:
                raise RuntimeError("invalid terminal task report")
            tasks.append(TaskResult(row["id"], row["status"], row["exit_code"], row["stdout"], row["stderr"], row["error"]))
        return RunResult(raw["outcome"], tuple(tasks))
