// CallPermit authority-envelope evaluator
// Maps Tethers 0.7 outcomes to ALLOW / ASK / DENY.
// Fail closed on malformed input.

import { createHash } from "node:crypto";
import type {
  AuthorityDecision,
  AuthorityEnvelope,
  AuthorityResult,
  CapabilityPolicy,
  TetherRequest,
  TetherResponse,
} from "./types.js";

// ---------------------------------------------------------------------------
// Canonical JSON (deterministic key order, no whitespace)
// ---------------------------------------------------------------------------

function canonicalJson(value: unknown): string {
  if (value === null || value === undefined) return "null";
  if (typeof value === "string") return JSON.stringify(value);
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  if (Array.isArray(value)) {
    return "[" + value.map(canonicalJson).join(",") + "]";
  }
  if (typeof value === "object") {
    const keys = Object.keys(value as Record<string, unknown>).sort();
    const pairs = keys.map((k) => JSON.stringify(k) + ":" + canonicalJson((value as Record<string, unknown>)[k]));
    return "{" + pairs.join(",") + "}";
  }
  return "null";
}

// ---------------------------------------------------------------------------
// Digest computation
// ---------------------------------------------------------------------------

function sha256Hex(data: string): string {
  return createHash("sha256").update(data, "utf8").digest("hex");
}

/**
 * Compute the stable identity digest for an authority envelope.
 * Digest covers all fields except `digest` itself.
 */
export function computeEnvelopeDigest(
  envelope: Omit<AuthorityEnvelope, "digest">
): string {
  const payload = canonicalJson({
    envelope_version: envelope.envelope_version,
    policy_id: envelope.policy_id,
    capabilities: envelope.capabilities,
    default_decision: envelope.default_decision,
  });
  return "sha256:" + sha256Hex(payload);
}

// ---------------------------------------------------------------------------
// Envelope builder
// ---------------------------------------------------------------------------

export function buildEnvelope(params: {
  policy_id: string;
  capabilities: CapabilityPolicy[];
  default_decision?: AuthorityDecision;
}): AuthorityEnvelope {
  const envelope: Omit<AuthorityEnvelope, "digest"> = {
    envelope_version: "1.0",
    policy_id: params.policy_id,
    capabilities: params.capabilities,
    default_decision: params.default_decision ?? "ASK",
  };
  const digest = computeEnvelopeDigest(envelope);
  return { ...envelope, digest };
}

// ---------------------------------------------------------------------------
// Envelope verification
// ---------------------------------------------------------------------------

export function verifyEnvelopeDigest(envelope: AuthorityEnvelope): boolean {
  const { digest, ...rest } = envelope;
  return digest === computeEnvelopeDigest(rest);
}

// ---------------------------------------------------------------------------
// Authority evaluation
// ---------------------------------------------------------------------------

/**
 * Look up the policy for a capability name in the envelope.
 * Returns the policy entry or null if not listed.
 */
function findCapabilityPolicy(
  envelope: AuthorityEnvelope,
  capabilityName: string
): CapabilityPolicy | null {
  return (
    envelope.capabilities.find((c) => c.capability_name === capabilityName) ??
    null
  );
}

/**
 * Evaluate a single planned action against the authority envelope.
 */
function evaluateAction(
  envelope: AuthorityEnvelope,
  action: { action_id: string; capability: string; effects: string[] }
): { decision: AuthorityDecision; reason: string } {
  const policy = findCapabilityPolicy(envelope, action.capability);

  if (policy === null) {
    return {
      decision: envelope.default_decision,
      reason: `capability "${action.capability}" not listed in envelope; using default ${envelope.default_decision}`,
    };
  }

  // Check effect allowlist if present
  if (policy.allowed_effects !== undefined && policy.allowed_effects.length > 0) {
    const denied = action.effects.filter((e) => !policy.allowed_effects!.includes(e));
    if (denied.length > 0) {
      return {
        decision: "DENY",
        reason: `effects [${denied.join(", ")}] not in allowed list for "${action.capability}"`,
      };
    }
  }

  return {
    decision: policy.decision,
    reason: `policy for "${action.capability}" returns ${policy.decision}`,
  };
}

/**
 * Evaluate a TetherResponse against an AuthorityEnvelope.
 *
 * Mapping:
 *  - Tethers error          → DENY  (fail closed)
 *  - Tethers not_matched    → ASK   (no plan; nothing to grant or deny)
 *  - Tethers matched + plan → aggregate per-action decisions:
 *      any DENY  → DENY
 *      any ASK   → ASK
 *      all ALLOW → ALLOW
 */
export function evaluateAuthority(
  envelope: AuthorityEnvelope,
  response: TetherResponse
): AuthorityResult {
  // Guard: envelope integrity
  if (!verifyEnvelopeDigest(envelope)) {
    return {
      decision: "DENY",
      reason: "envelope digest mismatch; envelope integrity unverifiable",
    };
  }

  // Guard: response structural validation
  if (
    !response ||
    typeof response.status !== "string" ||
    !["matched", "not_matched", "error"].includes(response.status)
  ) {
    return {
      decision: "DENY",
      reason: "malformed or missing Tethers response status",
    };
  }

  // Error → fail closed
  if (response.status === "error") {
    return {
      decision: "DENY",
      reason: `Tethers evaluation error: ${response.error?.code ?? "unknown"}`,
      trail: response.trail,
    };
  }

  // Not matched → ASK (nothing proposed)
  if (response.status === "not_matched") {
    return {
      decision: "ASK",
      reason: "Tethers did not match; no plan produced",
      trail: response.trail,
    };
  }

  // Matched but no plan (shouldn't happen per spec, but guard)
  if (response.plan === null || response.plan.actions.length === 0) {
    return {
      decision: "ASK",
      reason: "Tethers matched but produced no actions",
      trail: response.trail,
    };
  }

  // Evaluate each action
  const actionResults = response.plan.actions.map((action) => {
    const { decision, reason } = evaluateAction(envelope, {
      action_id: action.action_id,
      capability: action.capability,
      effects: action.effects,
    });
    return {
      action_id: action.action_id,
      capability: action.capability,
      decision,
      reason,
    };
  });

  // Aggregate: DENY wins, then ASK, then ALLOW
  let overall: AuthorityDecision = "ALLOW";
  let overallReason = "all planned actions are allowed";

  for (const ar of actionResults) {
    if (ar.decision === "DENY") {
      overall = "DENY";
      overallReason = `action ${ar.action_id} denied: ${ar.reason}`;
      break;
    }
    if (ar.decision === "ASK") {
      overall = "ASK";
      overallReason = `action ${ar.action_id} requires confirmation: ${ar.reason}`;
      // Don't break yet; a later DENY still wins
    }
  }

  return {
    decision: overall,
    reason: overallReason,
    action_results: actionResults,
    trail: response.trail,
  };
}

// ---------------------------------------------------------------------------
// Request validation (fail closed on malformed input)
// ---------------------------------------------------------------------------

export function validateRequest(request: unknown): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (request === null || typeof request !== "object") {
    return { valid: false, errors: ["request is not an object"] };
  }

  const r = request as Record<string, unknown>;

  if (typeof r.protocol_version !== "string") {
    errors.push("missing or non-string protocol_version");
  }
  if (typeof r.language_version !== "string") {
    errors.push("missing or non-string language_version");
  }
  if (typeof r.evaluation_id !== "string") {
    errors.push("missing or non-string evaluation_id");
  }
  if (r.tether === null || typeof r.tether !== "object") {
    errors.push("missing or invalid tether");
  } else {
    const t = r.tether as Record<string, unknown>;
    if (typeof t.id !== "string") errors.push("tether.id missing");
    if (typeof t.version !== "string") errors.push("tether.version missing");
    if (typeof t.source !== "string") errors.push("tether.source missing");
  }
  if (r.event === null || typeof r.event !== "object") {
    errors.push("missing or invalid event");
  } else {
    const e = r.event as Record<string, unknown>;
    if (typeof e.id !== "string") errors.push("event.id missing");
    if (typeof e.name !== "string") errors.push("event.name missing");
  }
  if (r.facts === null || typeof r.facts !== "object") {
    errors.push("missing or invalid facts");
  }
  if (!Array.isArray(r.capabilities)) {
    errors.push("missing or non-array capabilities");
  }

  return { valid: errors.length === 0, errors };
}

// ---------------------------------------------------------------------------
// Validate a TetherRequest against the authority envelope
// before evaluation. Fail closed on missing/unknown capabilities.
// ---------------------------------------------------------------------------

export function validateRequestCapabilities(
  request: TetherRequest,
  envelope: AuthorityEnvelope
): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  const envelopeNames = new Set(
    envelope.capabilities.map((c) => c.capability_name)
  );

  for (const cap of request.capabilities) {
    if (!envelopeNames.has(cap.name)) {
      errors.push(
        `capability "${cap.name}" is not listed in the authority envelope`
      );
    }
  }

  return { valid: errors.length === 0, errors };
}
