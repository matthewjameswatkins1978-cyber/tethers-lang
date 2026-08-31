import unittest
from tethers import evaluate

class WrapperTests(unittest.TestCase):
    def test_missing_binary_fails_closed(self):
        self.assertEqual(evaluate("definitely-missing-tethers", {}, 0.01)["decision"], "DENY")

if __name__ == "__main__":
    unittest.main()
