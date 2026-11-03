import unittest
from fluentci.plan import Task, Probe
from fluentci import workflow as W

class WorkflowTests(unittest.TestCase):
    def test_immutable_output_references_and_bounded_finalizers(self):
        original = Task("produce", "/missing", ("literal; $(touch unexpected)",))
        produced = W.resource(W.artifact(W.output(W.retry(original, 3, 10), "version", "version.txt"), "report", "report.txt"), "checkout")
        consumer = W.output_env(Task("consume", "/missing"), "VERSION", produced, "version")
        cleanup = W.always(consumer)
        optional = W.allow_failure(produced)
        self.assertIsNone(original.policy)
        self.assertEqual(consumer.needs, ("produce",))
        self.assertEqual(cleanup.timeout_ms, 30_000)
        self.assertEqual(cleanup.resource_wait_timeout_ms, 30_000)
        data = produced.to_dict()
        self.assertEqual(data["policy"]["retry"]["max_attempts"], 3)
        self.assertEqual(consumer.to_dict()["policy"]["bindings"]["VERSION"], {"task": "produce", "name": "version"})
        self.assertEqual(optional.to_dict()["policy"]["artifacts"], {"report": "report.txt"})

    def test_service_values_preserve_literal_probe_arguments(self):
        original = Task("database", "/missing")
        service = W.service(original, Probe("/missing-probe", ("two words",)))
        self.assertIsNone(original.service)
        self.assertEqual(service.to_dict()["service"]["readiness"]["args"], ("two words",))
