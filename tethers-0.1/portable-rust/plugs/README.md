# Official thin plugs

The plug collection translates host or agent intent into the canonical Tethers
request shape. It does not execute actions, contain policy logic, start a
service, or retain state. Hosts must submit the returned request to `tethers`
and treat malformed output, timeouts, or schema mismatches as `DENY`.

`tethers_plugs.py` is deliberately standard-library-only. The functions cover
Git, filesystem/path safety, process and shell, HTTP/network, secrets,
containers, generic tools, MCP, databases, messaging/email, and deployment.
The MCP helper can attach a deterministic SHA-256 fingerprint of a tool
definition; it never invokes the tool.
