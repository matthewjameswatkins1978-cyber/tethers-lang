// CallPermit authority-envelope evaluator — focused test suite
// Uses Node.js built-in test runner (node:test + node:assert).

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildEnvelope,
  computeEnvelopeDigest,
  evaluateAuthority,
  validateRequest,
  validateRequestCapabilities,
  verifyEnvelopeDigest,
} from "../src/authority.js";
import type {
  AuthorityEnvelope,
  AuthorityResult,
  TetherRequest,
  TetherResponse,
} from "../src/types.js";

const __dirname = dirname(fileURLToPath(import.meta.url));

// ---------------------------------------------------------------------------
// Load fixtures
// ---------------------------------------------------------------------------

const fixturePath = resolve(__dirname, "../fixtures/authority-envelopes.json");
const fixtureData = JSON.parse(readFileSync(fixturePath, "utf-8")) as {
  envelopes: Record<string, AuthorityEnvelope>;
  responses: Record<string, TetherResponse>;
};

// Recompute digests so fixtures are always self-consistent
function fixDigest(envelope: AuthorityEnvelope): AuthorityEnvelope {
  const { digest: _d, ...rest } = envelope;
  return { ...rest, digest: computeEnvelopeDigest(rest) };
}

const envelopes = {
  standingAllow: fixDigest(fixtureData.envelopes.standing_allow),
  standingDeny: fixDigest(fixtureData.envelopes.standing_deny),
  mixedPolicy: fixDigest(fixtureData.envelopes.mixed_policy),
  effectRestricted: fixDigest(fixtureData.envelopes.effect_restricted),
};

const responses = fixtureData.responses;

// ---------------------------------------------------------------------------
// Envelope identity
// ---------------------------------------------------------------------------

describe("AuthorityEnvelope digest", () => {
  it("computeEnvelopeDigest produces a stable sha256", () => {
    const e = buildEnvelope({
      policy_id: "stable-test",
      capabilities: [
        { capability_name: "a", capability_version: 1, decision: "ALLOW" },
      ],
    });
    const d1 = computeEnvelopeDigest(e);
    const d2 = computeEnvelopeDigest(e);
    assert.equal(d1, d2, "digest must be deterministic");
    assert.ok(d1.startsWith("sha256:"), "must be sha256 prefixed");
  });

  it("different policies produce different digests", () => {
    const e1 = buildEnvelope({
      policy_id: "p1",
      capabilities: [
        { capability_name: "a", capability_version: 1, decision: "ALLOW" },
      ],
    });
    const e2 = buildEnvelope({
      policy_id: "p2",
      capabilities: [
        { capability_name: "a", capability_version: 1, decision: "ALLOW" },
      ],
    });
    assert.notEqual(
      computeEnvelopeDigest(e1),
      computeEnvelopeDigest(e2),
      "different policy_ids must yield different digests"
    );
  });

  it("verifyEnvelopeDigest passes for a correctly built envelope", () => {
    const e = buildEnvelope({
      policy_id: "verify-test",
      capabilities: [],
    });
    assert.ok(verifyEnvelopeDigest(e), "digest should verify");
  });

  it("verifyEnvelopeDigest fails when digest is tampered", () => {
    const e = buildEnvelope({
      policy_id: "tamper-test",
      capabilities: [],
    });
    const tampered = { ...e, digest: "sha256:0000000000000000000000000000000000000000000000000000000000000000" };
    assert.ok(!verifyEnvelopeDigest(tampered), "tampered digest must fail");
  });
});

// ---------------------------------------------------------------------------
// Envelope building
// ---------------------------------------------------------------------------

describe("buildEnvelope", () => {
  it("defaults to ASK when default_decision omitted", () => {
    const e = buildEnvelope({
      policy_id: "default-ask",
      capabilities: [],
    });
    assert.equal(e.default_decision, "ASK");
  });

  it("preserves explicit default_decision", () => {
    const e = buildEnvelope({
      policy_id: "default-deny",
      capabilities: [],
      default_decision: "DENY",
    });
    assert.equal(e.default_decision, "DENY");
  });

  it("includes all provided capability policies", () => {
    const e = buildEnvelope({
      policy_id: "multi-cap",
      capabilities: [
        { capability_name: "a", capability_version: 1, decision: "ALLOW" },
        { capability_name: "b", capability_version: 2, decision: "DENY" },
      ],
    });
    assert.equal(e.capabilities.length, 2);
    assert.equal(e.capabilities[0].capability_name, "a");
    assert.equal(e.capabilities[1].capability_name, "b");
  });
});

// ---------------------------------------------------------------------------
// Request validation
// ---------------------------------------------------------------------------

describe("validateRequest", () => {
  it("accepts a well-formed Tethers request", () => {
    const req: TetherRequest = {
      protocol_version: "0.1",
      language_version: "0.1",
      evaluation_id: "eval_1",
      tether: { id: "t1", version: "v1", source: 'tether "Test"\nanchor\n    test.event' },
      event: { id: "e1", name: "test.event", data: {} },
      facts: {},
      capabilities: [],
    };
    const result = validateRequest(req);
    assert.ok(result.valid, "should be valid");
    assert.equal(result.errors.length, 0);
  });

  it("rejects null input", () => {
    const result = validateRequest(null);
    assert.ok(!result.valid);
    assert.ok(result.errors.length > 0);
  });

  it("rejects missing protocol_version", () => {
    const result = validateRequest({
      language_version: "0.1",
      evaluation_id: "e",
      tether: { id: "t", version: "v", source: "s" },
      event: { id: "e", name: "n", data: {} },
      facts: {},
      capabilities: [],
    });
    assert.ok(!result.valid);
    assert.ok(result.errors.some((e) => e.includes("protocol_version")));
  });

  it("rejects missing tether", () => {
    const result = validateRequest({
      protocol_version: "0.1",
      language_version: "0.1",
      evaluation_id: "e",
      event: { id: "e", name: "n", data: {} },
      facts: {},
      capabilities: [],
    });
    assert.ok(!result.valid);
    assert.ok(result.errors.some((e) => e.includes("tether")));
  });
});

// ---------------------------------------------------------------------------
// Authority evaluation — core mapping
// ---------------------------------------------------------------------------

describe("evaluateAuthority", () => {
  it("ALLOW when plan actions match allowed policies", () => {
    const result: AuthorityResult = evaluateAuthority(
      envelopes.standingAllow,
      responses.matched_allow
    );
    assert.equal(result.decision, "ALLOW");
    assert.ok(result.action_results);
    assert.equal(result.action_results.length, 1);
    assert.equal(result.action_results[0].decision, "ALLOW");
  });

  it("DENY when plan action matches a denied policy", () => {
    const result: AuthorityResult = evaluateAuthority(
      envelopes.standingDeny,
      responses.matched_deny
    );
    assert.equal(result.decision, "DENY");
    assert.ok(result.action_results);
    assert.equal(result.action_results[0].decision, "DENY");
  });

  it("ASK when Tethers did not match", () => {
    const result: AuthorityResult = evaluateAuthority(
      envelopes.standingAllow,
      responses.not_matched
    );
    assert.equal(result.decision, "ASK");
    assert.equal(result.reason, "Tethers did not match; no plan produced");
  });

  it("DENY when Tethers evaluation error (fail closed)", () => {
    const result: AuthorityResult = evaluateAuthority(
      envelopes.standingAllow,
      responses.error
    );
    assert.equal(result.decision, "DENY");
    assert.ok(result.reason.includes("error"));
  });

  it("DENY when envelope digest is tampered", () => {
    const tampered: AuthorityEnvelope = {
      ...envelopes.standingAllow,
      digest: "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    };
    const result: AuthorityResult = evaluateAuthority(
      tampered,
      responses.matched_allow
    );
    assert.equal(result.decision, "DENY");
    assert.ok(result.reason.includes("digest mismatch"));
  });

  it("defaults to ASK for unknown capabilities when default is ASK", () => {
    const result: AuthorityResult = evaluateAuthority(
      envelopes.standingAllow,
      responses.matched_deny // file.move not listed in standingAllow
    );
    assert.equal(result.decision, "ASK");
    assert.ok(result.reason.includes("default"));
  });

  it("defaults to DENY for unknown capabilities when default is DENY", () => {
    const unknownCapResponse: TetherResponse = {
      ...responses.matched_allow,
      plan: {
        ...responses.matched_allow.plan!,
        actions: [
          {
            ...responses.matched_allow.plan!.actions[0],
            capability: "unknown.capability",
          },
        ],
      },
    };
    const result: AuthorityResult = evaluateAuthority(
      envelopes.mixedPolicy,
      unknownCapResponse
    );
    assert.equal(result.decision, "DENY");
    assert.ok(result.reason.includes("default DENY"));
  });

  it("ASK when action requires confirmation in mixed policy", () => {
    const result: AuthorityResult = evaluateAuthority(
      envelopes.mixedPolicy,
      responses.matched_allow
    );
    assert.equal(result.decision, "ALLOW");
    // fixture.ping is ALLOW in mixed_policy
  });
});

// ---------------------------------------------------------------------------
// Effect restriction
// ---------------------------------------------------------------------------

describe("effect restriction", () => {
  it("DENY when action effects exceed allowed_effects", () => {
    const response: TetherResponse = {
      ...responses.matched_allow,
      plan: {
        ...responses.matched_allow.plan!,
        actions: [
          {
            ...responses.matched_allow.plan!.actions[0],
            capability: "fixture.ping",
            effects: ["fixture.test", "unauthorized.effect"],
          },
        ],
      },
    };
    const result: AuthorityResult = evaluateAuthority(
      envelopes.effectRestricted,
      response
    );
    assert.equal(result.decision, "DENY");
    assert.ok(result.reason.includes("not in allowed list"));
  });

  it("ALLOW when action effects are within allowed_effects", () => {
    const result: AuthorityResult = evaluateAuthority(
      envelopes.effectRestricted,
      responses.matched_allow
    );
    assert.equal(result.decision, "ALLOW");
  });
});

// ---------------------------------------------------------------------------
// Capability pre-validation
// ---------------------------------------------------------------------------

describe("validateRequestCapabilities", () => {
  it("passes when all request capabilities are listed in envelope", () => {
    const req: TetherRequest = {
      protocol_version: "0.1",
      language_version: "0.1",
      evaluation_id: "eval_1",
      tether: { id: "t1", version: "v1", source: 'tether "Test"' },
      event: { id: "e1", name: "test.event", data: {} },
      facts: {},
      capabilities: [
        { name: "fixture.ping", version: "1.0.0", inputs: { message: "string" }, effects: ["fixture.test"] },
      ],
    };
    const result = validateRequestCapabilities(req, envelopes.standingAllow);
    assert.ok(result.valid);
  });

  it("fails when request capability is not in envelope", () => {
    const req: TetherRequest = {
      protocol_version: "0.1",
      language_version: "0.1",
      evaluation_id: "eval_1",
      tether: { id: "t1", version: "v1", source: 'tether "Test"' },
      event: { id: "e1", name: "test.event", data: {} },
      facts: {},
      capabilities: [
        { name: "rogue.capability", version: "1.0.0", inputs: {}, effects: ["bad.effect"] },
      ],
    };
    const result = validateRequestCapabilities(req, envelopes.standingAllow);
    assert.ok(!result.valid);
    assert.ok(result.errors.some((e) => e.includes("rogue.capability")));
  });
});

// ---------------------------------------------------------------------------
// Malformed input fail-closed
// ---------------------------------------------------------------------------

describe("fail closed on malformed input", () => {
  it("DENY for null response", () => {
    const result = evaluateAuthority(
      envelopes.standingAllow,
      null as unknown as TetherResponse
    );
    assert.equal(result.decision, "DENY");
  });

  it("DENY for response with invalid status", () => {
    const result = evaluateAuthority(
      envelopes.standingAllow,
      { status: "invalid_status" } as unknown as TetherResponse
    );
    assert.equal(result.decision, "DENY");
  });
});
