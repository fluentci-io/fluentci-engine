import json
import unittest
from fluentci import files as F
from fluentci.plan import Plan

class FileTests(unittest.TestCase):
    def test_builders_are_pure_and_env_files_do_not_mutate_the_original(self):
        task = F.write_bytes("bytes", "path ' 世界", b"\0\xff")
        configured = F.env_file(task, "settings.env")
        self.assertEqual(task.env_files, ())
        self.assertEqual(configured.env_files, ("settings.env",))
        value = json.loads(Plan((configured,)).to_json())["tasks"][0]
        self.assertEqual(value["program"], "")
        self.assertEqual(value["file_operation"], {"op":"write_bytes", "path":"path ' 世界", "bytes":[0,255], "overwrite":False})

    def test_all_operations_have_the_shared_wire_format(self):
        tasks = (F.read_text("a","p"), F.read_bytes("b","p"), F.write_text("c","p","literal"), F.copy_file("d","a","b"), F.copy_tree("e","a","b"), F.move("f","a","b"), F.mkdir("g","dir"), F.remove("h","dir"), F.stat("i","p"), F.exists("j","p"), F.list_entries("k"), F.glob("l","**/*.txt"), F.hash("m","p"), F.chmod("n","p",0o700), F.symlink("o","target","link"))
        self.assertEqual(len(tasks),15)
        self.assertTrue(all(task.to_dict()["file_operation"]["op"] for task in tasks))
