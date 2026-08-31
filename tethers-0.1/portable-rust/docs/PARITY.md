# Windows/Linux parity

The release corpus in `tests/parity-corpus.json` is the same JSON input set on
Windows x64 and Linux x64 musl. `scripts/run_parity.py` compares the decision,
matched rule, reason, policy version, policy fingerprint, and error fields. A
platform is not release-ready if any comparison differs.

The separate `tests/adversarial-corpus.json` adds malformed JSON, schema abuse,
invalid comparators, traversal/absolute paths, wildcard selectors, and
hard-denied container/database/secret operations.
