import unittest
from tethers import decode_response, evaluate

class WrapperTests(unittest.TestCase):
    def test_missing_binary_fails_closed(self):
        self.assertEqual(evaluate("definitely-missing-tethers", {}, 0.01)["decision"], "DENY")

    def test_schema_mismatch_fails_closed(self):
        self.assertEqual(decode_response('{"schema_version":"2","decision":"ALLOW"}') ["decision"], "DENY")

if __name__ == "__main__":
    unittest.main()
