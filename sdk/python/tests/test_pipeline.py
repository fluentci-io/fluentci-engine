import ast
from dataclasses import FrozenInstanceError
import json
from pathlib import Path
import sys
from types import ModuleType
import unittest

from fluentci import Provider, file, pipeline
from fluentci.build import entry_source


class Guest(ModuleType):
    def __init__(self):
        super().__init__("extism")
        self.calls = []

    def import_fn(self, namespace, name):
        def decorate(function):
            def invoke(*arguments):
                self.calls.append((namespace, name, arguments))
                if name in {"pipeline", "as_service", "cache", "file", "nix", "flox"}:
                    return json.dumps({"id": f"{name}-id"})
                if name == "stdout":
                    return "plugin output\n"
                if name == "sha256":
                    return "digest"
                return None
            return invoke
        return decorate

    def plugin_fn(self, function):
        return function


class PipelineTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.guest = Guest()
        sys.modules["extism"] = cls.guest

    def setUp(self):
        self.guest.calls.clear()

    def test_builders_are_immutable_and_do_not_call_engine(self):
        original = pipeline("build")
        configured = original.exec("printf", "%s", "hello world").with_env("MODE", "release")
        self.assertEqual(original.steps, ())
        self.assertEqual(len(configured.steps), 2)
        self.assertEqual(self.guest.calls, [])
        with self.assertRaises(FrozenInstanceError):
            configured.name = "mutated"

    def test_literal_arguments_and_unicode_reach_guest_imports(self):
        output = pipeline("build").exec("printf", "%s", "space ' quote; $(touch unexpected) 世界").stdout()
        self.assertEqual(output, "plugin output\n")
        arguments = next(arguments for _, name, arguments in self.guest.calls if name == "with_exec")
        words = json.loads(arguments[0])
        self.assertEqual(words[0], "'printf'")
        self.assertIn("'\\''", words[2])
        self.assertEqual({namespace for namespace, _, _ in self.guest.calls}, {"extism:host/user"})

    def test_services_are_values_and_lower_before_client_pipeline(self):
        service = pipeline("database").exec("database-server").as_service("db")
        client = pipeline("client").with_service(service).exec("check")
        self.assertEqual(self.guest.calls, [])
        client.stdout()
        names = [name for _, name, _ in self.guest.calls]
        self.assertLess(names.index("as_service"), names.index("with_service"))

    def test_cache_uses_the_canonical_object_wire_format(self):
        pipeline("cached").with_cache("cargo", "target").stdout()
        arguments = next(arguments for _, name, arguments in self.guest.calls if name == "with_cache")
        self.assertEqual(json.loads(arguments[0]), {"id": "cache-id", "key": "cargo", "path": "target"})

    def test_file_hash_passes_path_not_handle_id(self):
        self.assertEqual(file("file with spaces").sha256(), "digest")
        self.assertEqual(self.guest.calls[-1][2], ("file with spaces",))

    def test_provider_settings_are_preserved(self):
        pipeline("nix").using(Provider.NIX, impure=True).exec("true").stdout()
        args = next(args for _, name, args in self.guest.calls if name == "nix")
        self.assertEqual(json.loads(args[0]), {"impure": True})

    def test_entry_declares_all_abi_imports_at_top_level(self):
        root = Path(__file__).resolve().parents[1]
        bindings = (root / "src/fluentci/_imports.py").read_text()
        source = "from __future__ import annotations\nimport extism\n@extism.plugin_fn\ndef greet() -> None:\n    pass\n"
        result = entry_source(source, bindings)
        tree = ast.parse(result)
        imports = [node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name.startswith("_fluentci_") and node.name != "_fluentci_annotations"]
        self.assertEqual(len(imports), 52)
        self.assertIsInstance(tree.body[0], ast.ImportFrom)
        voids = [node for node in imports if node.name == "_fluentci_with_exec"]
        self.assertIsNone(voids[0].returns, "void imports must have zero Wasm results")
        self.assertIn("sys.modules", result)


if __name__ == "__main__":
    unittest.main()
