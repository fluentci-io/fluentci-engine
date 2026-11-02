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
            plan = self.plan([self.task("exported-step", "echo private-log; exit 9"),
                              self.task("skipped-step", "echo should-not-run", ["exported-step"])])
            result = subprocess.run(self.command(plan), capture_output=True, text=True, env=env, timeout=20)
            self.assertNotEqual(result.returncode, 0)
            data = b"".join(payloads)
            self.assertIn(b"pipeline.run", data)
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
    def test_tui_renders_live_tasks_and_restores_cursor(self):
        import errno
        import fcntl
        import pty
        import select
        import struct
        import termios
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        path = self.plan([self.task("visible", "printf '\\033[2Junsafe'; /bin/sleep 0.2; echo done")])
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
            self.assertIn(b"\x1b[?25h", output)
            self.assertNotIn(b"\x1b[2J", output)
        finally:
            os.close(master)
            if proc.poll() is None:
                proc.kill()
                proc.wait()


if __name__ == "__main__":
    unittest.main()
