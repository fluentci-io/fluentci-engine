"""Execute the PureScript Buildx guest through the engine; optional real Docker build."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", ROOT / "target/release/fluentci-engine")).resolve()
PLUGIN = ROOT / "examples/buildx/dist/plugin.wasm"


class BuildxGuest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="fluentci buildx ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.project = self.root / "project ' 世界"
        self.project.mkdir()
        self.docker = self.root / "docker tool"
        self.docker.write_text(
            f"#!{sys.executable}\n"
            "import json, os, pathlib, sys\n"
            "with pathlib.Path('invocations').open('a') as log: log.write('invoked\\n')\n"
            "print(json.dumps({'argv':sys.argv[1:],'cwd':os.getcwd(),'experimental':os.environ.get('BUILDX_EXPERIMENTAL')}),flush=True)\n"
            "sys.exit(int(os.environ.get('BUILDX_TEST_EXIT', '0')))\n"
        )
        self.docker.chmod(0o755)

    def call(self, function, arguments, *, success=True, environment=None):
        result = subprocess.run(
            [str(BINARY), "call", "-m", str(PLUGIN), function, json.dumps(arguments)],
            cwd=self.root, capture_output=True, text=True, env=environment, timeout=60)
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def request(self, run, **arguments):
        return dict(project=str(self.project), runDir=str(self.root / run), docker=str(self.docker), **arguments)

    def task_output(self, run):
        report = json.loads((self.root / run / "result.json").read_text())
        self.assertEqual(report["outcome"], "succeeded", report)
        self.assertEqual(len(report["tasks"]), 1)
        return json.loads(Path(report["tasks"][0]["stdout"]).read_text())

    def test_named_build_and_bake_arguments_reach_native_processes_literally(self):
        value = "VALUE=héllo 世界; $(touch injected) 'quotes'"
        self.call("build", self.request("build-run", context="--literal-context", platform=["linux/amd64", "linux/arm64"],
                                        tag=["example:first", "example:second"], buildArg=[value],
                                        output=["type=local,dest=out"], pull=False, experimental=True))
        output = self.task_output("build-run")
        self.assertEqual(Path(output["cwd"]).resolve(), self.project.resolve())
        self.assertEqual(output["experimental"], "1")
        self.assertEqual(output["argv"][-2:], ["--", "--literal-context"])
        for argument in ["--platform=linux/amd64", "--platform=linux/arm64", "--build-arg=" + value, "--pull=false"]:
            self.assertIn(argument, output["argv"])
        self.assertFalse((self.project / "injected").exists())
        self.call("bake", self.request("bake-run", file=["docker-bake.hcl", "override ' 世界.hcl"],
                                       targets=["app", "test"], set=["*.platform=linux/arm64"], print=True))
        baked = self.task_output("bake-run")["argv"]
        self.assertIn("--file=override ' 世界.hcl", baked)
        self.assertEqual(baked[-3:], ["--", "app", "test"])

    def test_nested_and_default_true_options_and_native_exit_codes(self):
        self.call("imagetools_create", self.request("images", sources=["example:one", "example:two"], preferIndex=False))
        args = self.task_output("images")["argv"]
        self.assertEqual(args[:3], ["buildx", "imagetools", "create"])
        self.assertIn("--prefer-index=false", args)
        environment = dict(os.environ, BUILDX_TEST_EXIT="7")
        result = self.call("build", self.request("failed"), success=False, environment=environment)
        self.assertEqual(result.returncode, 7, result.stdout + result.stderr)
        report = json.loads((self.root / "failed/result.json").read_text())
        self.assertEqual(report["outcome"], "failed")
        self.assertEqual(report["tasks"][0]["exit_code"], 7)

    def test_invalid_arguments_and_reused_run_directories_do_not_start_docker(self):
        for index, invalid in enumerate([{"platform": "linux/arm64"}, {"push": "true"}, {"timeoutMs": 0},
                                          {"unknown": "flag"}, {"context": "bad\0path"}]):
            run = f"invalid-{index}"
            self.call("build", self.request(run, **invalid), success=False)
            self.assertFalse((self.root / run).exists())
        self.assertFalse((self.project / "invocations").exists())
        self.call("version", self.request("version"))
        self.call("version", self.request("version"), success=False)
        self.assertEqual((self.project / "invocations").read_text(), "invoked\n")


@unittest.skipUnless(os.environ.get("FLUENTCI_BUILDX_REAL") == "1", "set FLUENTCI_BUILDX_REAL=1 for a real Docker daemon build")
class RealBuildx(unittest.TestCase):
    def test_wasm_guest_builds_and_exports_a_real_scratch_image(self):
        docker = shutil.which("docker")
        self.assertIsNotNone(docker, "Docker CLI and Buildx are required")
        with tempfile.TemporaryDirectory(prefix="fluentci buildx real ") as directory:
            root = Path(directory)
            project = root / "project ' 世界"
            project.mkdir()
            (project / "Dockerfile").write_text("FROM scratch\nCOPY hello.txt /hello.txt\n")
            (project / "hello.txt").write_text("built through the PureScript Wasm guest 世界\n")
            output = root / "exported"
            run = root / "run"
            request = {"project": str(project), "runDir": str(run), "docker": docker,
                       "context": ".", "progress": "plain", "output": [f"type=local,dest={output}"]}
            result = subprocess.run([str(BINARY), "call", "-m", str(PLUGIN), "build", json.dumps(request)],
                                    cwd=root, capture_output=True, text=True, timeout=120)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            report = json.loads((run / "result.json").read_text())
            self.assertEqual(report["outcome"], "succeeded", report)
            self.assertEqual((output / "hello.txt").read_bytes(), (project / "hello.txt").read_bytes())


if __name__ == "__main__":
    unittest.main()
