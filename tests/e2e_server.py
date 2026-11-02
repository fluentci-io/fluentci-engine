"""Authenticated real HTTP run-server tests against the release CLI."""
import concurrent.futures
import http.server
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time
import threading
import uuid
import unittest
import urllib.request
import urllib.error

BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", "target/release/fluentci-engine")).resolve()
TOKEN = "test-only-run-server-token"

class RunServer(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fluentci-server-e2e-")
        self.root = Path(self.temp.name)
        self.log = open(self.root / "server.log", "w+")
        self.proc = None
        self.extra_env = {}
        self.start()

    def start(self):
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            port = sock.getsockname()[1]
        self.base = f"http://127.0.0.1:{port}"
        env = {k:v for k,v in os.environ.items() if not k.startswith("OTEL_")}
        env["FLUENTCI_RUN_TOKEN"] = TOKEN
        env.update(self.extra_env)
        self.proc = subprocess.Popen([str(BINARY), "serve-runs", "--listen", f"127.0.0.1:{port}", "--data-dir", str(self.root / "runs"), "--max-runs", "2"], env=env, stdout=self.log, stderr=self.log)
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            self.assertIsNone(self.proc.poll(), "server exited during startup")
            try:
                if self.request("GET", "/v1/runs/missing", token=False)[0] == 401:
                    return
            except OSError:
                time.sleep(0.02)
        self.fail("server did not become ready")

    def stop(self):
        if self.proc and self.proc.poll() is None:
            self.proc.send_signal(signal.SIGINT)
            try:
                self.proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                self.proc.wait()
                self.fail("server failed to shut down")

    def tearDown(self):
        self.stop()
        self.log.close()
        self.temp.cleanup()

    def request(self, method, path, data=None, token=True, extra_headers=None):
        headers = {"Content-Type":"application/json"}
        if token:
            headers["Authorization"] = "Bearer " + TOKEN
        headers.update(extra_headers or {})
        request = urllib.request.Request(self.base + path, data=None if data is None else json.dumps(data).encode(), headers=headers, method=method)
        try:
            response = urllib.request.urlopen(request, timeout=5)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            return response.status, response.read(), response.headers

    def plan(self, script, env=None):
        return dict(inherit_env=False, tasks=[dict(id="task", program="/bin/sh", args=["-c", script], cwd=str(self.root), env=env or {})])

    def submit(self, plan):
        code, body, headers = self.request("POST", "/v1/runs", plan)
        self.assertEqual(code, 202, body)
        self.assertEqual(headers["Cache-Control"], "no-store")
        return headers["Location"]

    def terminal(self, path):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            code, body, _ = self.request("GET", path)
            self.assertEqual(code, 200)
            state = json.loads(body)
            if state["status"] != "running":
                return state
            time.sleep(0.02)
        self.fail("run did not reach terminal state")

    def test_workspace_runs_keep_source_and_artifacts_separate(self):
        (self.root / "input").write_text("original")
        plan = self.plan("/bin/cat input > output; printf changed > input")
        plan["tasks"][0]["cwd"] = "."
        plan["workspace"] = dict(source=str(self.root), inputs=["input"])
        locations = [self.submit(plan), self.submit(plan)]
        for location in locations:
            self.assertEqual(self.terminal(location)["status"], "succeeded")
            run_id = location.rsplit("/", 1)[1]
            workspace = self.root / "runs" / run_id / "execution/workspace"
            self.assertEqual((workspace / "output").read_text(), "original")
            self.assertEqual((workspace / "input").read_text(), "changed")
        self.assertEqual((self.root / "input").read_text(), "original")
        self.assertFalse((self.root / "output").exists())

    def test_restart_recovers_complete_reports_and_isolates_invalid_reports(self):
        cases = ["success", "failure", "torn", "missing", "empty", "wrong-id", "nonterminal", "bad-plan"]
        locations = {}
        for case in cases:
            location = self.submit(self.plan("printf x >> recovery-count; exit " + ("1" if case == "failure" else "0")))
            self.terminal(location)
            locations[case] = location
        self.stop()
        for case, location in locations.items():
            directory = self.root / "runs" / location.rsplit("/", 1)[1]
            state = json.loads((directory / "state.json").read_text())
            state.update(status="running", error="stale error")
            (directory / "state.json").write_text(json.dumps(state))
            path = directory / "execution/result.json"
            report = json.loads(path.read_text())
            if case == "torn":
                path.write_text('{"tasks":[')
            elif case == "missing":
                path.unlink()
            elif case == "bad-plan":
                (directory / "plan.json").write_text('{')
            elif case in ["empty", "wrong-id", "nonterminal"]:
                if case == "empty":
                    report["tasks"] = []
                elif case == "wrong-id":
                    report["tasks"][0]["id"] = "unrelated"
                else:
                    report["tasks"][0]["status"] = "running"
                path.write_text(json.dumps(report))
        self.start()
        for case, location in locations.items():
            state = self.terminal(location)
            expected = {"success":"succeeded", "failure":"failed"}.get(case, "interrupted")
            self.assertEqual(state["status"], expected, case)
            if expected == "interrupted":
                self.assertIn("complete result", state["error"])
            else:
                self.assertIsNone(state["error"])
        self.assertEqual((self.root / "recovery-count").read_text(), "x" * len(cases))
        self.assertEqual(self.terminal(self.submit(self.plan("exit 0")))["status"], "succeeded")

    def test_idempotent_submission_replays_conflicts_and_survives_restart(self):
        headers = {"Idempotency-Key": "release-123"}
        plan = self.plan("printf executed >> count")
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            replies = list(pool.map(lambda _: self.request("POST", "/v1/runs", plan, extra_headers=headers), range(8)))
        self.assertTrue(all(reply[0] == 202 for reply in replies), replies)
        locations = {reply[2]["Location"] for reply in replies}
        self.assertEqual(len(locations), 1)
        self.assertTrue(all(reply[2]["Idempotency-Key"] == "release-123" for reply in replies))
        location = locations.pop()
        self.assertEqual(self.terminal(location)["status"], "succeeded")
        self.assertEqual((self.root / "count").read_text(), "executed")
        conflict = self.request("POST", "/v1/runs", self.plan("touch duplicate"), extra_headers=headers)
        self.assertEqual(conflict[0], 409)
        self.assertFalse((self.root / "duplicate").exists())
        self.stop()
        self.start()
        replay = self.request("POST", "/v1/runs", plan, extra_headers=headers)
        self.assertEqual(replay[0], 202)
        self.assertEqual(replay[2]["Location"], location)
        self.assertEqual(json.loads(replay[1])["status"], "succeeded")
        self.assertEqual((self.root / "count").read_text(), "executed")
        # Simulate persisted state left by a crash before completion was recorded.
        self.stop()
        run_dir = self.root / "runs" / location.rsplit("/", 1)[1]
        state = json.loads((run_dir / "state.json").read_text())
        state["status"] = "running"
        (run_dir / "state.json").write_text(json.dumps(state))
        (run_dir / "execution/result.json").unlink()
        self.start()
        replay = self.request("POST", "/v1/runs", plan, extra_headers=headers)
        self.assertEqual(replay[0], 202)
        self.assertEqual(json.loads(replay[1])["status"], "interrupted")
        self.assertEqual((self.root / "count").read_text(), "executed")
        for invalid in ["", "../escape", "space key", "x" * 129]:
            self.assertEqual(self.request("POST", "/v1/runs", plan, extra_headers={"Idempotency-Key":invalid})[0], 400)

    def test_idempotency_keys_are_case_sensitive_and_accept_maximum_length(self):
        plan = self.plan("printf executed >> key-count")
        locations = []
        for key in ["CaseSensitive", "casesensitive", "x" * 128]:
            accepted = self.request("POST", "/v1/runs", plan, extra_headers={"Idempotency-Key":key})
            self.assertEqual(accepted[0], 202)
            location = accepted[2]["Location"]
            self.assertEqual(self.terminal(location)["status"], "succeeded")
            locations.append(location)
        self.assertEqual(len(set(locations)), 3)
        self.assertEqual((self.root / "key-count").read_text(), "executed" * 3)

    def test_idempotent_replay_bypasses_capacity_and_revalidation(self):
        work = self.root / "workspace"
        work.mkdir()
        plan = self.plan("printf done")
        plan["tasks"][0]["cwd"] = str(work)
        headers = {"Idempotency-Key":"completed-work"}
        accepted = self.request("POST", "/v1/runs", plan, extra_headers=headers)
        self.assertEqual(accepted[0], 202)
        self.terminal(accepted[2]["Location"])
        work.rmdir()
        busy = [self.submit(self.plan("/bin/sleep 30")) for _ in range(2)]
        replay = self.request("POST", "/v1/runs", plan, extra_headers=headers)
        self.assertEqual(replay[0], 202)
        self.assertEqual(replay[2]["Location"], accepted[2]["Location"])
        self.assertEqual(self.request("POST", "/v1/runs", self.plan("exit 0"), extra_headers={"Idempotency-Key":"capacity-retry"})[0], 429)
        for path in busy:
            self.request("POST", path + "/cancel")
            self.terminal(path)
        admitted = self.request("POST", "/v1/runs", self.plan("exit 0"), extra_headers={"Idempotency-Key":"capacity-retry"})
        self.assertEqual(admitted[0], 202)

    def test_auth_validation_and_single_store_owner(self):
        self.assertEqual(self.request("POST", "/v1/runs", self.plan("touch unexpected"), token=False)[0], 401)
        self.assertEqual(self.request("GET", "/v1/runs/missing", extra_headers={"Authorization":"Bearer " + "x" * len(TOKEN)})[0], 401)
        bad = self.plan("touch unexpected")
        bad["tasks"][0]["needs"] = ["missing"]
        self.assertEqual(self.request("POST", "/v1/runs", bad)[0], 400)
        inherited = self.plan("touch unexpected")
        inherited["inherit_env"] = True
        self.assertEqual(self.request("POST", "/v1/runs", inherited)[0], 400)
        self.assertFalse((self.root / "unexpected").exists())
        env = dict(os.environ, FLUENTCI_RUN_TOKEN=TOKEN)
        duplicate = subprocess.run([str(BINARY), "serve-runs", "--listen", "127.0.0.1:0", "--data-dir", str(self.root / "runs")], env=env, capture_output=True, timeout=10)
        self.assertNotEqual(duplicate.returncode, 0)
        self.assertIn(b"another run server", duplicate.stderr)

    def test_concurrency_capacity_live_events_cancellation_and_restart(self):
        paths = [self.submit(self.plan("echo $VALUE; while :; do /bin/sleep 0.02; done", {"VALUE":value})) for value in ["left", "right"]]
        self.assertEqual(self.request("POST", "/v1/runs", self.plan("exit 0"))[0], 429)
        for path, expected in zip(paths, ["left", "right"]):
            deadline = time.monotonic() + 5
            body = b""
            while time.monotonic() < deadline:
                code, body, headers = self.request("GET", path + "/events")
                self.assertEqual(code, 200)
                if expected.encode() in body:
                    break
                time.sleep(0.02)
            self.assertIn(expected.encode(), body)
            code, log, log_headers = self.request("GET", path + "/logs/0/stdout")
            self.assertEqual(code, 200)
            self.assertEqual(log, expected.encode() + b"\n")
            self.assertEqual(self.request("GET", path + "/logs/0/stdout?cursor=" + log_headers["X-Next-Cursor"])[1], b"")
            self.assertEqual(self.request("GET", path + "/logs/0/invalid")[0], 404)
            self.assertEqual(self.request("GET", path + "/events?cursor=1")[0], 400)
            self.assertEqual(self.request("POST", path + "/cancel")[0], 202)
            self.assertEqual(self.terminal(path)["status"], "cancelled")
            tail = self.request("GET", path + "/events?cursor=" + headers["X-Next-Cursor"])[1]
            self.assertEqual(body + tail, self.request("GET", path + "/events")[1])
            self.assertEqual(json.loads(self.request("GET", path + "/result")[1])["tasks"][0]["status"], "cancelled")
        self.stop()
        interrupted = str(uuid.uuid4())
        abandoned = self.root / "runs" / interrupted
        abandoned.mkdir()
        (abandoned / "state.json").write_text(json.dumps(dict(id=interrupted, status="running", error=None)))
        self.start()
        self.assertEqual(self.terminal("/v1/runs/" + interrupted)["status"], "interrupted")
        for path in paths:
            self.assertEqual(self.terminal(path)["status"], "cancelled")
            self.assertEqual(self.request("GET", path + "/result")[0], 200)

    def test_resource_leases_coordinate_server_and_local_cli(self):
        holder = self.plan("touch holding; while :; do /bin/sleep 0.02; done")
        holder["resource_dir"] = str(self.root / "locks")
        holder["tasks"][0]["resources"] = ["shared-workspace"]
        path = self.submit(holder)
        deadline = time.monotonic() + 3
        while not (self.root / "holding").exists():
            self.assertLess(time.monotonic(), deadline)
            time.sleep(0.01)
        contender = self.plan("touch acquired")
        contender["resource_dir"] = holder["resource_dir"]
        contender["tasks"][0].update(resources=["shared-workspace"], resource_wait_timeout_ms=100)
        plan_file = self.root / "local.json"
        plan_file.write_text(json.dumps(contender))
        def execute(name):
            return subprocess.run([str(BINARY), "execute", str(plan_file), "--run-dir", str(self.root / name), "--progress", "json"], capture_output=True, timeout=10)
        blocked = execute("blocked")
        self.assertNotEqual(blocked.returncode, 0)
        self.assertEqual(json.loads((self.root / "blocked/result.json").read_text())["tasks"][0]["status"], "timed_out")
        self.assertFalse((self.root / "acquired").exists())
        self.request("POST", path + "/cancel")
        self.assertEqual(self.terminal(path)["status"], "cancelled")
        self.assertEqual(execute("admitted").returncode, 0)
        self.assertTrue((self.root / "acquired").exists())

    def test_remote_trace_context_and_export_flush(self):
        self.stop()
        payloads = []
        class Collector(http.server.BaseHTTPRequestHandler):
            def do_POST(inner):
                payloads.append(inner.rfile.read(int(inner.headers["Content-Length"])))
                inner.send_response(200)
                inner.send_header("Content-Length", "0")
                inner.end_headers()
            def log_message(self, *_):
                pass
        collector = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Collector)
        thread = threading.Thread(target=collector.serve_forever, daemon=True)
        thread.start()
        try:
            self.extra_env = dict(OTEL_EXPORTER_OTLP_ENDPOINT=f"http://127.0.0.1:{collector.server_port}/v1/traces", OTEL_EXPORTER_OTLP_PROTOCOL="http/protobuf")
            self.start()
            trace = "1234567890abcdef1234567890abcdef"
            code, _, headers = self.request("POST", "/v1/runs", self.plan("echo traced"), extra_headers={"traceparent":f"00-{trace}-1234567890abcdef-01"})
            self.assertEqual(code, 202)
            path = headers["Location"]
            self.assertEqual(self.terminal(path)["status"], "succeeded")
            _, body, _ = self.request("GET", path + "/events")
            events = [json.loads(line) for line in body.splitlines()]
            self.assertTrue(all(e["trace_id"] == trace for e in events))
            self.stop()
            self.assertIn(b"pipeline.step", b"".join(payloads))
        finally:
            collector.shutdown()
            collector.server_close()
            thread.join(timeout=5)

    def test_server_shutdown_cancels_services_and_persists_results(self):
        plan = self.plan("echo $$ > service.pid; while :; do /bin/sleep 0.02; done")
        plan["tasks"][0]["service"] = dict(readiness=dict(program="/bin/sh",args=["-c","exit 0"]), shutdown_timeout_ms=100)
        plan["tasks"].append(dict(id="consumer", needs=["task"], program="/bin/sleep", args=["20"], cwd=str(self.root)))
        path = self.submit(plan)
        deadline = time.monotonic() + 5
        while not (self.root / "service.pid").exists():
            self.assertLess(time.monotonic(), deadline)
            time.sleep(0.02)
        pid = int((self.root / "service.pid").read_text())
        self.stop()
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)
        self.start()
        self.assertEqual(self.terminal(path)["status"], "cancelled")

if __name__ == "__main__":
    unittest.main()
