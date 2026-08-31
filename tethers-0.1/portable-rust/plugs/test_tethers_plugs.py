import hashlib
import json
import tempfile
import unittest

from tethers_plugs import filesystem, mcp_tool, secret


class PlugTests(unittest.TestCase):
    def test_filesystem_resolves_relative_path(self):
        with tempfile.TemporaryDirectory() as root:
            result = filesystem("read", actor="agent", path="src/main.rs", workspace_root=root)
            self.assertEqual(result["action"], "filesystem.read")
            self.assertEqual(result["context"]["path"], "src/main.rs")

    def test_filesystem_marks_escape(self):
        with tempfile.TemporaryDirectory() as root:
            result = filesystem("write", actor="agent", path="../secret", workspace_root=root)
            self.assertTrue(result["context"]["outside_workspace"])

    def test_mcp_fingerprint_is_canonical(self):
        definition = {"name": "ping", "description": "safe", "inputSchema": {"type": "object"}}
        result = mcp_tool("ping", actor="agent", definition=definition)
        expected = hashlib.sha256(json.dumps(definition, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        self.assertEqual(result["context"]["tool_definition_sha256"], expected)

    def test_secret_never_contains_value(self):
        result = secret("use", actor="agent", name="OPENAI_API_KEY")
        self.assertNotIn("value", json.dumps(result).lower())


if __name__ == "__main__":
    unittest.main()
