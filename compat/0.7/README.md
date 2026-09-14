# Tethers 0.7 compatibility seed corpus

This is the first durable 0.7 compatibility corpus. The seed files are exact
copies of representative protocol evidence from the tagged `tethers-v0.7.0`
release line (`51ef9fd53f353c0413c4900f337071cbca6f921a`). They protect the
0.1 protocol envelope and the 0.7 host-facing evidence shape while the broader
R6 migration corpus is assembled.

| Area | Fixture | Protected property |
| --- | --- | --- |
| configuration / CLI | `config/evaluation-request.json`, `cli/evaluation-request.json` | explicit protocol and language version axes |
| Human Tether input | `config/evaluation-request.json` (`tether.source`) | supported 0.7 source remains representable |
| Capability manifest | `manifests/fixture-ping.json` | manifest schema and digest-bearing capability identity |
| Plan | `plans/record-completed-task.json` | ordered Action and argument shape |
| Trail / receipt | `trails/record-completed-task.json` | ordered evaluation evidence |
| MCP | `mcp/validate-valid.*.jsonl` | stable initialize and validate transcript |

Each fixture’s provenance is this tagged release and the exact path above. The
seed harness validates structure and required version identifiers; it does not
claim that the complete 0.7-to-1.0 migration audit is finished.
