"""End-to-end host executor tests. No daemon, package installs, or cloud credentials."""
import http.server
import json
import os
from pathlib import Path
import queue
import signal
import subprocess
import sys
import tempfile
import threading
import time
import unittest

BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", "target/release/fluentci-engine")).resolve()


class HostExecution(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fluentci-e2e-")
        self.root = Path(self.temp.name)

    def tearDown(self):
        self.temp.cleanup()

    def task(self, name, script, needs=None):
        return dict(id=name, program="/bin/sh", args=["-c", script], cwd=".", needs=needs or [], timeout_ms=5000)

    def plan(self, tasks, **options):
        path = self.root / "plan.json"
        path.write_text(json.dumps(dict(version=1, jobs=2, tasks=tasks, **options)))
        return path

    def command(self, plan, run="run", progress="json"):
        return [str(BINARY), "execute", str(plan), "--run-dir", str(self.root / run), "--progress", progress]

    def test_workspace_source_is_relative_to_plan_and_outputs_stay_in_run(self):
        source = self.root / "source"
        source.mkdir()
        (source / "input").write_text("original")
        plan = self.plan([self.task("copy", "cat input > output; printf changed > input")],
                         workspace=dict(source="source", inputs=["input"]))
        result = subprocess.run(self.command(plan), cwd="/", capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / "run/workspace/output").read_text(), "original")
        self.assertEqual((self.root / "run/workspace/input").read_text(), "changed")
        self.assertEqual((source / "input").read_text(), "original")
        self.assertFalse((source / "output").exists())

    def test_cache_diff_explains_changes_without_exposing_environment_values(self):
        cached = self.task("cached", "printf artifact > artifact")
        cached["cache"] = dict(inputs=[], outputs=["artifact"], toolchain="diff-fixture")
        cached["env"] = dict(TOKEN="baseline-secret")
        def execute(tasks, run):
            plan = self.plan(tasks, inherit_env=False, cache_dir=str(self.root / "cache"))
            result = subprocess.run(self.command(plan, run), capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
        execute([cached, self.task("removed", "true"), self.task("uncached", "true")], "before")
        cached["env"]["TOKEN"] = "current-secret"
        execute([cached, self.task("added", "true"), self.task("uncached", "true")], "after")
        command = [str(BINARY), "cache-diff", str(self.root / "before"), str(self.root / "after/result.json")]
        result = subprocess.run(command + ["--json"], capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        tasks = {task["id"]: task for task in report["tasks"]}
        self.assertEqual(report["version"], 1)
        self.assertEqual(tasks["cached"], dict(id="cached", status="changed", changed_components=["environment"]))
        self.assertEqual(tasks["removed"]["status"], "removed")
        self.assertEqual(tasks["added"]["status"], "added")
        self.assertEqual(tasks["uncached"]["status"], "unavailable")
        plain = subprocess.run(command, capture_output=True, text=True, timeout=5)
        self.assertEqual(plain.returncode, 0, plain.stderr)
        for output in [result.stdout, plain.stdout]:
            self.assertNotIn("baseline-secret", output)
            self.assertNotIn("current-secret", output)
        same = subprocess.run([str(BINARY), "cache-diff", str(self.root / "after"), str(self.root / "after"), "--json"], capture_output=True, text=True, timeout=5)
        self.assertEqual(same.returncode, 0, same.stderr)
        self.assertEqual(next(task for task in json.loads(same.stdout)["tasks"] if task["id"] == "cached")["status"], "unchanged")
        document = json.loads((self.root / "after/result.json").read_text())
        document["tasks"].append(document["tasks"][0])
        malformed = self.root / "invalid.json"
        malformed.write_text(json.dumps(document))
        invalid = subprocess.run([str(BINARY), "cache-diff", str(malformed), str(self.root / "after")], capture_output=True, text=True, timeout=5)
        self.assertNotEqual(invalid.returncode, 0)
        self.assertIn("duplicate task id", invalid.stderr)
        document["tasks"].pop()
        document["tasks"][0]["id"] = "unsafe\x1b[31m\nname"
        malformed.write_text(json.dumps(document))
        escaped = subprocess.run([str(BINARY), "cache-diff", str(malformed), str(self.root / "after")], capture_output=True, text=True, timeout=5)
        self.assertEqual(escaped.returncode, 0, escaped.stderr)
        self.assertNotIn("\x1b", escaped.stdout)
        self.assertIn("unsafe\\u001b[31m\\nname", escaped.stdout)

    def test_probe_diagnostics_are_journaled_before_service_finishes(self):
        service = self.task("db", "while :; do /bin/sleep 0.02; done")
        service.pop("timeout_ms")
        service["service"] = dict(readiness=dict(program="/bin/sh", args=["-c", "printf readiness-detail >&2; exit 1"], interval_ms=100, timeout_ms=100), startup_timeout_ms=10000, shutdown_timeout_ms=100)
        proc = subprocess.Popen(self.command(self.plan([service])), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        events = queue.Queue()
        reader = threading.Thread(target=lambda: [events.put(json.loads(line)) for line in proc.stdout], daemon=True)
        reader.start()
        try:
            deadline = time.monotonic() + 5
            while True:
                event = events.get(timeout=max(0.01, deadline - time.monotonic()))
                if event.get("type") == "log" and event.get("stream") == "probe_stderr" and "readiness-detail" in event["text"]:
                    break
                self.assertLess(time.monotonic(), deadline)
            self.assertEqual(event["id"], "db")
            self.assertIsNone(proc.poll(), "probe diagnostics must be live")
            proc.send_signal(signal.SIGINT)
            self.assertNotEqual(proc.wait(timeout=5), 0)
            reader.join(timeout=2)
            journal = [json.loads(line) for line in (self.root / "run/events.jsonl").read_text().splitlines()]
            self.assertEqual([event["sequence"] for event in journal], list(range(len(journal))))
            text = "".join(event["text"] for event in journal if event.get("type") == "log" and event.get("stream") == "probe_stderr")
            self.assertEqual(text, (self.root / "run/0.probe_stderr").read_text())
        finally:
            if proc.poll() is None:
                proc.send_signal(signal.SIGINT)
                proc.wait(timeout=5)
            reader.join(timeout=2)
            proc.stdout.close()
            proc.stderr.close()

    def test_resources_coordinate_separate_cli_processes(self):
        holder = self.task("holder", "touch holding; while :; do sleep 0.02; done")
        holder["resources"] = ["shared-checkout"]
        plan = self.plan([holder], resource_dir=str(self.root / "locks"))
        process = subprocess.Popen(self.command(plan, "holder"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            deadline = time.monotonic() + 3
            while not (self.root / "holding").exists():
                self.assertLess(time.monotonic(), deadline)
                time.sleep(0.01)
            contender = self.task("contender", "touch acquired")
            contender.update(resources=["shared-checkout"], resource_wait_timeout_ms=100)
            plan = self.plan([contender], resource_dir=str(self.root / "locks"))
            blocked = subprocess.run(self.command(plan, "blocked"), capture_output=True, text=True, timeout=10)
            self.assertNotEqual(blocked.returncode, 0)
            result = json.loads((self.root / "blocked/result.json").read_text())
            self.assertEqual(result["tasks"][0]["status"], "timed_out")
            self.assertFalse((self.root / "acquired").exists())
            self.assertTrue(any(e.get("wait_reason") == "resource: shared-checkout" for e in map(json.loads, blocked.stdout.splitlines())))
            process.send_signal(signal.SIGINT)
            process.wait(timeout=5)
            admitted = subprocess.run(self.command(plan, "admitted"), capture_output=True, text=True, timeout=10)
            self.assertEqual(admitted.returncode, 0, admitted.stderr)
            self.assertTrue((self.root / "acquired").exists())
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()

    def test_service_readiness_and_cleanup_with_single_job(self):
        daemon = "import http.server, pathlib, os; s=http.server.HTTPServer(('127.0.0.1',0),http.server.SimpleHTTPRequestHandler); pathlib.Path('port').write_text(str(s.server_port)); pathlib.Path('pid').write_text(str(os.getpid())); print('service-ready', flush=True); s.serve_forever()"
        probe = "import pathlib, urllib.request; urllib.request.urlopen('http://127.0.0.1:'+pathlib.Path('port').read_text()+'/health', timeout=0.2).read()"
        (self.root / "health").write_text("healthy")
        service = dict(id="http", program=sys.executable, args=["-c", daemon], cwd=".",
                       service=dict(readiness=dict(program=sys.executable, args=["-c", probe], interval_ms=20, timeout_ms=1000),
                                    startup_timeout_ms=3000, shutdown_timeout_ms=200))
        client = dict(id="client", needs=["http"], program=sys.executable, args=["-c", probe], cwd=".")
        plan = self.plan([service, client])
        document = json.loads(plan.read_text())
        document["jobs"] = 1
        plan.write_text(json.dumps(document))
        result = subprocess.run(self.command(plan), capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        events = [json.loads(line) for line in result.stdout.splitlines()]
        self.assertTrue(any(e.get("status") == "ready" for e in events))
        self.assertTrue(any(e.get("status") == "stopping" for e in events))
        self.assertTrue(events[-1]["success"])
        with self.assertRaises(ProcessLookupError):
            os.kill(int((self.root / "pid").read_text()), 0)

    def test_otlp_flushes_step_spans_before_failed_cli_exit(self):
        payloads = []
        class Collector(http.server.BaseHTTPRequestHandler):
            def do_POST(inner):
                payloads.append(inner.rfile.read(int(inner.headers["Content-Length"])))
                inner.send_response(200)
                inner.send_header("Content-Type", "application/x-protobuf")
                inner.send_header("Content-Length", "0")
                inner.end_headers()
            def log_message(self, *_):
                pass
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Collector)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            env = {k: v for k, v in os.environ.items() if not k.startswith("OTEL_")}
            env.update(OTEL_EXPORTER_OTLP_ENDPOINT=f"http://127.0.0.1:{server.server_port}/v1/traces",
                       OTEL_EXPORTER_OTLP_PROTOCOL="http/protobuf")
            service = self.task("exported-service", "trap 'exit 0' TERM; while :; do /bin/sleep 0.02; done")
            service["service"] = dict(readiness=dict(program="/bin/sh", args=["-c", "echo private-probe-output"],
                                                     interval_ms=10, timeout_ms=1000),
                                      shutdown_timeout_ms=100)
            plan = self.plan([service,
                              self.task("exported-step", "echo private-log; exit 9", ["exported-service"]),
                              self.task("skipped-step", "echo should-not-run", ["exported-step"])])
            result = subprocess.run(self.command(plan), capture_output=True, text=True, env=env, timeout=20)
            self.assertNotEqual(result.returncode, 0)
            data = b"".join(payloads)
            self.assertIn(b"pipeline.run", data)
            self.assertIn(b"pipeline.service.probe", data)
            self.assertIn(b"pipeline.probe.phase", data)
            self.assertIn(b"readiness", data)
            self.assertNotIn(b"private-probe-output", data)
            self.assertIn(b"exported-step", data)
            self.assertIn(b"skipped-step", data)
            self.assertNotIn(b"private-log", data)
            events = [json.loads(line) for line in result.stdout.splitlines()]
            self.assertTrue(all(len(e["trace_id"]) == 32 and len(e["span_id"]) == 16 for e in events))
            self.assertEqual(len({e["trace_id"] for e in events}), 1)
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=5)

    def test_diamond_and_machine_readable_events(self):
        tasks = [self.task("prepare", "echo prepare"), self.task("left", "echo left", ["prepare"]),
                 self.task("right", "echo right", ["prepare"]), self.task("join", "echo joined", ["left", "right"])]
        result = subprocess.run(self.command(self.plan(tasks)), capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        events = [json.loads(line) for line in result.stdout.splitlines()]
        self.assertEqual([e["sequence"] for e in events], list(range(len(events))))
        self.assertEqual(events[-1]["type"], "run_finished")
        self.assertTrue(events[-1]["success"])
        states = [e for e in events if e["type"] == "task_state"]
        joined = next(i for i, e in enumerate(states) if e["id"] == "join" and e["status"] == "running")
        for parent in ["left", "right"]:
            self.assertTrue(any(e["id"] == parent and e["status"] == "succeeded" for e in states[:joined]))
        self.assertEqual(json.loads((self.root / "run/result.json").read_text())["tasks"][-1]["status"], "succeeded")

    def test_failure_exit_code_and_skipped_dependency(self):
        result = subprocess.run(self.command(self.plan([self.task("bad", "exit 7"), self.task("skip", "touch unexpected", ["bad"])])), capture_output=True, text=True, timeout=15)
        self.assertNotEqual(result.returncode, 0)
        tasks = json.loads((self.root / "run/result.json").read_text())["tasks"]
        self.assertEqual(tasks[0]["exit_code"], 7)
        self.assertEqual(tasks[1]["status"], "skipped")
        self.assertFalse((self.root / "unexpected").exists())

    def test_log_is_visible_before_task_completes(self):
        path = self.plan([self.task("live", "printf first; while [ ! -f release ]; do /bin/sleep 0.01; done; printf second")])
        proc = subprocess.Popen(self.command(path), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        lines = queue.Queue()
        reader = threading.Thread(target=lambda: [lines.put(line) for line in proc.stdout], daemon=True)
        reader.start()
        try:
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                event = json.loads(lines.get(timeout=10))
                if event["type"] == "log" and "first" in event["text"]:
                    break
            else:
                self.fail("first output never arrived")
            self.assertIsNone(proc.poll(), "log was buffered until task exit")
            (self.root / "release").touch()
            self.assertEqual(proc.wait(timeout=10), 0)
        finally:
            if proc.poll() is None:
                proc.send_signal(signal.SIGINT)
                proc.wait(timeout=10)
            reader.join(timeout=2)
            proc.stdout.close()
            proc.stderr.close()

    def test_ctrl_c_writes_cancelled_result(self):
        proc = subprocess.Popen(self.command(self.plan([self.task("slow", "echo ready > ready; /bin/sleep 60")])), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            deadline = time.monotonic() + 10
            while not (self.root / "ready").exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue((self.root / "ready").exists())
            proc.send_signal(signal.SIGINT)
            proc.communicate(timeout=10)
            self.assertNotEqual(proc.returncode, 0)
            self.assertEqual(json.loads((self.root / "run/result.json").read_text())["tasks"][0]["status"], "cancelled")
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait()

    def test_cli_cache_restores_and_invalidates(self):
        (self.root / "input").write_text("first")
        task = self.task("build", "/bin/cat input > output")
        task["cache"] = dict(inputs=["input"], outputs=["output"], toolchain="e2e-shell-v1")
        path = self.plan([task], inherit_env=False, cache_dir="cache")
        for name, outcome in [("cold", "stored"), ("warm", "hit"), ("changed", "stored")]:
            if name == "warm":
                (self.root / "output").unlink()
            if name == "changed":
                (self.root / "input").write_text("second")
            result = subprocess.run(self.command(path, run=name), capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((self.root / name / "result.json").read_text())
            self.assertEqual(report["tasks"][0]["cache"]["outcome"], outcome)
            self.assertEqual((self.root / "output").read_text(), (self.root / "input").read_text())

    def test_invalid_plan_has_no_effects_and_plain_mode_has_no_ansi(self):
        result = subprocess.run(self.command(self.plan([self.task("bad", "touch unexpected", ["missing"])])), capture_output=True, text=True, timeout=15)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / "run").exists())
        self.assertIn("missing", result.stderr)
        result = subprocess.run(self.command(self.plan([self.task("ok", "echo hello")]), progress="plain"), capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 0)
        self.assertIn("hello", result.stdout)
        self.assertNotIn("\x1b", result.stdout)


    @unittest.skipUnless(os.name == "posix", "requires a PTY")
    def watch_command(self, run="run", progress="json", follow=False, timeout=None):
        command = [str(BINARY), "watch", str(self.root / run), "--progress", progress]
        if follow:
            command.append("--follow")
        if timeout is not None:
            command += ["--timeout-ms", str(timeout)]
        return command

    def test_watch_replays_saved_runs_without_a_plan_or_reexecution(self):
        for success in [True, False]:
            run = f"saved-{success}"
            plan = self.plan([self.task("saved-task", "echo once >> executions; echo saved-output; exit " + ("0" if success else "7"))])
            original = subprocess.run(self.command(plan, run=run), capture_output=True, text=True, timeout=10)
            self.assertEqual(original.returncode == 0, success)
            plan.unlink()
            before = (self.root / "executions").read_bytes()
            expected = [json.loads(line) for line in (self.root / run / "events.jsonl").read_text().splitlines()]
            for follow in [False, True]:
                replay = subprocess.run(self.watch_command(run, follow=follow), capture_output=True, text=True, timeout=5)
                self.assertEqual(replay.returncode == 0, success, replay.stderr)
                self.assertEqual([json.loads(line) for line in replay.stdout.splitlines()], expected)
                self.assertEqual((self.root / "executions").read_bytes(), before)

    def test_watch_follows_live_output_and_detaches_without_cancelling(self):
        plan = self.plan([self.task("live", "echo started > started; printf live-marker; while [ ! -f release ]; do /bin/sleep 0.02; done; echo done")])
        engine = subprocess.Popen(self.command(plan, progress="plain"), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        watcher = None
        reader = None
        try:
            deadline = time.monotonic() + 3
            while not (self.root / "started").exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue((self.root / "started").exists())
            watcher = subprocess.Popen(self.watch_command(follow=True, timeout=3000), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            messages = queue.Queue()
            reader = threading.Thread(target=lambda: [messages.put(json.loads(line)) for line in watcher.stdout], daemon=True)
            reader.start()
            observed = []
            while not any(e.get("type") == "log" and "live-marker" in e["text"] for e in observed):
                observed.append(messages.get(timeout=3))
            watcher.send_signal(signal.SIGINT)
            self.assertEqual(watcher.wait(timeout=3), 0, watcher.stderr.read())
            reader.join(timeout=3)
            self.assertIsNone(engine.poll(), "detaching must not stop the execution")
            timed = subprocess.run(self.watch_command(follow=True, timeout=80), capture_output=True, text=True, timeout=3)
            self.assertNotEqual(timed.returncode, 0)
            self.assertIn("watch timed out", timed.stderr)
            self.assertIsNone(engine.poll(), "viewer timeout must not stop the execution")
            # A second observer follows the same run through normal completion.
            second = subprocess.Popen(self.watch_command(follow=True, timeout=3000), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                first = json.loads(second.stdout.readline())
                self.assertEqual(first["type"], "run_started")
                (self.root / "release").write_text("go")
                output, errors = second.communicate(timeout=4)
                self.assertEqual(second.returncode, 0, errors)
                self.assertTrue(json.loads(output.splitlines()[-1])["success"])
            finally:
                if second.poll() is None:
                    second.kill()
                    second.wait()
            engine.communicate(timeout=3)
            self.assertEqual(engine.returncode, 0)
        finally:
            if watcher is not None and watcher.poll() is None:
                watcher.kill()
                watcher.wait()
            if reader is not None:
                reader.join(timeout=3)
            if watcher is not None:
                watcher.stdout.close()
                watcher.stderr.close()
            if engine.poll() is None:
                engine.send_signal(signal.SIGINT)
                engine.communicate(timeout=5)

    def test_watch_rejects_damaged_journals_and_handles_partial_records(self):
        run = self.root / "run"
        run.mkdir()
        journal = run / "events.jsonl"
        first = dict(type="run_started", sequence=0, elapsed_ms=0, tasks=0)
        journal.write_text(json.dumps(first) + "\n" + '{"type":')
        snapshot = subprocess.run(self.watch_command(), capture_output=True, text=True, timeout=3)
        self.assertEqual(snapshot.returncode, 0, snapshot.stderr)
        self.assertEqual([json.loads(line) for line in snapshot.stdout.splitlines()], [first])
        final = json.dumps(dict(type="run_finished", sequence=1, elapsed_ms=1, success=True))
        journal.write_text(json.dumps(first) + "\n" + final[:5])
        proc = subprocess.Popen(self.watch_command(follow=True, timeout=1000), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            self.assertEqual(json.loads(proc.stdout.readline()), first)
            with journal.open("a") as output:
                output.write(final[5:] + "\n")
            output, error = proc.communicate(timeout=3)
            self.assertEqual(proc.returncode, 0, error)
            self.assertEqual([json.loads(line) for line in output.splitlines()], [json.loads(final)])
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.communicate()
        for changed in ["truncate", "replace", "remove"]:
            journal.write_text(json.dumps(first) + "\n")
            proc = subprocess.Popen(self.watch_command(follow=True, timeout=1000), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                self.assertEqual(json.loads(proc.stdout.readline()), first)
                if changed == "truncate":
                    journal.write_text("")
                elif changed == "replace":
                    replacement = run / "replacement"
                    replacement.write_text(json.dumps(first) + "\n")
                    replacement.replace(journal)
                else:
                    journal.unlink()
                _, error = proc.communicate(timeout=3)
                self.assertNotEqual(proc.returncode, 0)
                self.assertIn({"truncate": "truncated", "replace": "replaced", "remove": "disappeared"}[changed], error)
            finally:
                if proc.poll() is None:
                    proc.kill()
                    proc.wait()
        journal.write_bytes(b"x" * (4 * 1024 * 1024 + 1))
        oversized = subprocess.run(self.watch_command(), capture_output=True, text=True, timeout=5)
        self.assertNotEqual(oversized.returncode, 0)
        self.assertIn("exceeds 4 MiB", oversized.stderr)
        journal.write_text(json.dumps({**first, "sequence": 4}) + "\n")
        unordered = subprocess.run(self.watch_command(), capture_output=True, text=True, timeout=3)
        self.assertNotEqual(unordered.returncode, 0)
        self.assertIn("not ordered", unordered.stderr)
        journal.unlink()
        os.mkfifo(journal)
        special = subprocess.run(self.watch_command(), capture_output=True, text=True, timeout=3)
        self.assertNotEqual(special.returncode, 0)
        self.assertIn("regular file", special.stderr)

    def test_closed_plain_progress_pipe_cancels_and_reaps_the_execution(self):
        plan = self.plan([self.task("writer", "echo $$ > writer.pid; while :; do echo output; /bin/sleep 0.02; done")])
        proc = subprocess.Popen(self.command(plan, progress="plain"), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            proc.stdout.readline()
            deadline = time.monotonic() + 3
            pid = None
            while time.monotonic() < deadline:
                if (self.root / "writer.pid").exists():
                    value = (self.root / "writer.pid").read_text().strip()
                    if value:
                        pid = int(value)
                        break
                time.sleep(0.01)
            self.assertIsNotNone(pid)
            proc.stdout.close()
            self.assertNotEqual(proc.wait(timeout=3), 0)
            self.assertNotIn("panicked", proc.stderr.read())
            with self.assertRaises(ProcessLookupError):
                os.kill(pid, 0)
        finally:
            if proc.poll() is None:
                proc.send_signal(signal.SIGINT)
                proc.wait(timeout=5)
            proc.stdout.close()
            proc.stderr.close()

    def test_watch_tui_reconstructs_rows_and_preserves_recorded_elapsed_time(self):
        import errno
        import fcntl
        import pty
        import select
        import struct
        import termios
        run = self.root / "run"
        run.mkdir()
        events = [dict(type="run_started", sequence=0, elapsed_ms=0, tasks=1),
                  dict(type="task_state", sequence=1, elapsed_ms=12345, id="saved-visible", status="failed", cache="disabled", reason="", error="saved failure reason", duration_ms=5000),
                  dict(type="run_finished", sequence=2, elapsed_ms=12345, success=False)]
        (run / "events.jsonl").write_text("".join(json.dumps(e) + "\n" for e in events))
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 20, 100, 0, 0))
        proc = subprocess.Popen(self.watch_command(progress="tui"), stdout=slave, stderr=slave,
                                env={**os.environ, "TERM": "xterm-256color"})
        os.close(slave)
        output = bytearray()
        try:
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    try:
                        chunk = os.read(master, 65536)
                    except OSError as error:
                        if error.errno == errno.EIO:
                            break
                        raise
                    if not chunk:
                        break
                    output.extend(chunk)
                elif proc.poll() is not None:
                    break
            self.assertNotEqual(proc.wait(timeout=2), 0)
            for expected in [b"0/1 succeeded", b"12.3s", b"saved-visible", b"saved failure reason", b"\x1b[?25h"]:
                self.assertIn(expected, output)
        finally:
            os.close(master)
            if proc.poll() is None:
                proc.kill()
                proc.wait()

    def test_tui_renders_live_tasks_and_restores_cursor(self):
        import errno
        import fcntl
        import pty
        import select
        import struct
        import termios
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        service = self.task("db", "while :; do /bin/sleep 0.02; done")
        service["service"] = dict(readiness=dict(program="/bin/sh", args=["-c", "printf probe-visible; exit 0"]), shutdown_timeout_ms=100)
        path = self.plan([service, self.task("visible", "printf '\\033[2Junsafe'; /bin/sleep 0.2; echo done", ["db"])])
        proc = subprocess.Popen(self.command(path, progress="tui"), stdout=slave, stderr=slave,
                                env={**os.environ, "TERM": "xterm-256color"})
        os.close(slave)
        output = bytearray()
        try:
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    try:
                        chunk = os.read(master, 65536)
                    except OSError as error:
                        if error.errno == errno.EIO:
                            break
                        raise
                    if not chunk:
                        break
                    output.extend(chunk)
                elif proc.poll() is not None:
                    break
            self.assertEqual(proc.wait(timeout=2), 0, output.decode(errors="replace"))
            self.assertIn(b"FluentCI", output)
            self.assertIn(b"visible", output)
            self.assertIn(b"Recent output:", output)
            self.assertIn(b"unsafe", output)
            self.assertIn(b"probe-visible", output)
            self.assertIn(b"probe_stdout", output)
            self.assertIn(b"\x1b[?25h", output)
            self.assertNotIn(b"\x1b[2J", output)
        finally:
            os.close(master)
            if proc.poll() is None:
                proc.kill()
                proc.wait()


    def test_tui_keeps_failure_reason_after_output_tail_rolls_over(self):
        import errno
        import fcntl
        import pty
        import select
        import struct
        import termios
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 16, 160, 0, 0))
        tasks = [self.task(f"done-{i}", "true") for i in range(8)]
        failed = self.task("broken", "")
        failed["program"] = str(self.root / "missing-program")
        tasks += [failed, self.task("noisy", "sleep 0.15; i=0; while [ $i -lt 20 ]; do echo noise-$i; i=$((i+1)); done")]
        path = self.plan(tasks, fail_fast=False)
        proc = subprocess.Popen(self.command(path, progress="tui"), stdout=slave, stderr=slave,
                                env={**os.environ, "TERM": "xterm-256color"})
        os.close(slave)
        output = bytearray()
        try:
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    try:
                        chunk = os.read(master, 65536)
                    except OSError as error:
                        if error.errno == errno.EIO:
                            break
                        raise
                    if not chunk:
                        break
                    output.extend(chunk)
                elif proc.poll() is not None:
                    break
            self.assertNotEqual(proc.wait(timeout=2), 0)
            final_frame = output.rsplit(b"\x1b[J", 1)[-1]
            rows, tail = final_frame.split(b"Recent output:", 1)
            self.assertIn(b"broken", rows)
            self.assertIn(b"Failed", rows)
            self.assertIn(b"No such file", rows)
            self.assertIn(b"noise-19", tail)
            self.assertNotIn(b"No such file", tail)
            self.assertIn(b"\x1b[?25h", output)
        finally:
            os.close(master)
            if proc.poll() is None:
                proc.kill()
                proc.wait()


if __name__ == "__main__":
    unittest.main()
