import unittest
from fluentci import matrix as M
from fluentci.plan import Task

class MatrixTests(unittest.TestCase):
    def test_products_filters_ids_and_join_preserve_dependencies(self):
        cases = M.product(M.axis("node", ["22", "24"]), M.axis("suite", ["unit", "integration"]))
        cases = M.exclude(cases, lambda pair: pair == ("22", "integration"))
        cases = M.include(cases, [M.named("experimental", ("26", "unit"))])
        tasks = M.matrix("test", cases, lambda pair: Task("ignored", "/missing-program", pair, "/missing-directory"))
        self.assertEqual([task.id for task in tasks], ["test[node=22,suite=unit]", "test[node=24,suite=unit]", "test[node=24,suite=integration]", "test[experimental]"])
        joined = M.after(tasks, Task("join", "test", needs=("prepare",)))
        self.assertEqual(joined.needs, ("prepare", *(task.id for task in tasks)))
        self.assertEqual(M.parallel(tasks, 2).jobs, 2)
        self.assertEqual(M.named("a,b=世界", None).key, "a%2Cb%3D%E4%B8%96%E7%95%8C")

    def test_duplicates_and_expansion_limits_fail_before_callbacks(self):
        def build(_):
            self.fail("invalid cases invoked the task builder")
        with self.assertRaises(ValueError):
            M.matrix("test", (M.named("same", 1), M.named("same", 2)), build)
        with self.assertRaises(ValueError):
            M.product(M.axis("a", [str(i) for i in range(65)]), M.axis("b", [str(i) for i in range(65)]))
