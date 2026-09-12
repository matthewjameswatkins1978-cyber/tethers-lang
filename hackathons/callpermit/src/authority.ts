// CallPermit authority-envelope evaluator
// Maps Tethers 0.7 outcomes to ALLOW / ASK / DENY.
// Fail closed on malformed input.

import { createHash } from "node:crypto";
import type {
  AuthorityDecision,
  AuthorityEnvelope,
  AuthorityResult,
  AppointmentPolicy,
  AppointmentTerms,
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

function deepFreeze<T>(value: T): T {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    Object.freeze(value);
    for (const child of Object.values(value as Record<string, unknown>)) deepFreeze(child);
  }
  return value;
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
    appointment_policy: envelope.appointment_policy ?? null,
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
  appointment_policy?: AppointmentPolicy;
}): AuthorityEnvelope {
  const envelope: Omit<AuthorityEnvelope, "digest"> = {
    envelope_version: "1.0",
    policy_id: params.policy_id,
    capabilities: params.capabilities.map((capability) => ({
      ...capability,
      allowed_effects: capability.allowed_effects ? [...capability.allowed_effects] : undefined,
      scope: capability.scope ? { ...capability.scope, allowed_prefixes: capability.scope.allowed_prefixes ? [...capability.scope.allowed_prefixes] : undefined } : undefined,
    })),
    default_decision: params.default_decision ?? "ASK",
    appointment_policy: params.appointment_policy
      ? {
          ...params.appointment_policy,
          allowed_weekdays: [...params.appointment_policy.allowed_weekdays],
          allowed_extras: [...params.appointment_policy.allowed_extras],
          forbidden_commitments: [...params.appointment_policy.forbidden_commitments],
        }
      : undefined,
  };
  const digest = computeEnvelopeDigest(envelope);
  return deepFreeze({ ...envelope, digest });
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
  if (
    !response.plan ||
    !Array.isArray(response.plan.actions) ||
    response.plan.actions.length === 0 ||
    response.plan.actions.some(
      (action) =>
        !action ||
        typeof action.action_id !== "string" ||
        typeof action.capability !== "string" ||
        !Array.isArray(action.effects)
    )
  ) {
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
  if (r.tether === null || typeof r.tether !== "object" || Array.isArray(r.tether)) {
    errors.push("missing or invalid tether");
  } else {
    const t = r.tether as Record<string, unknown>;
    if (typeof t.id !== "string") errors.push("tether.id missing");
    if (typeof t.version !== "string") errors.push("tether.version missing");
    if (typeof t.source !== "string") errors.push("tether.source missing");
  }
  if (r.event === null || typeof r.event !== "object" || Array.isArray(r.event)) {
    errors.push("missing or invalid event");
  } else {
    const e = r.event as Record<string, unknown>;
    if (typeof e.id !== "string") errors.push("event.id missing");
    if (typeof e.name !== "string") errors.push("event.name missing");
  }
  if (r.facts === null || typeof r.facts !== "object" || Array.isArray(r.facts)) {
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

export function validateResponseIdentity(
  request: TetherRequest,
  response: TetherResponse
): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  if (!response || typeof response !== "object") return { valid: false, errors: ["response is not an object"] };
  if (response.evaluation_id !== request.evaluation_id) errors.push("evaluation_id does not match request");
  if (response.event_id !== request.event.id) errors.push("event_id does not match request");
  if (response.tether_id !== request.tether.id) errors.push("tether_id does not match request");
  if (response.tether_version !== request.tether.version) errors.push("tether_version does not match request");
  return { valid: errors.length === 0, errors };
}

// ---------------------------------------------------------------------------
// CallPermit appointment authority
// ---------------------------------------------------------------------------

/** The synthetic, low-stakes task used by the offline CallPermit build. */
export const DEMO_APPOINTMENT_POLICY: AppointmentPolicy = {
  capability_name: "appointment.book",
  service: "standard service",
  allowed_weekdays: [1, 2, 3, 4, 5],
  earliest_local_time: "13:00",
  maximum_price_minor: 6000,
  currency: "GBP",
  allowed_extras: [],
  forbidden_commitments: ["subscription", "membership", "deposit", "purchase"],
};

export function buildAppointmentEnvelope(
  policy_id = "callpermit.demo.appointment"
): AuthorityEnvelope {
  return buildEnvelope({
    policy_id,
    capabilities: [
      {
        capability_name: DEMO_APPOINTMENT_POLICY.capability_name,
        capability_version: 1,
        decision: "ALLOW",
        allowed_effects: ["appointment.book"],
      },
    ],
    default_decision: "DENY",
    appointment_policy: DEMO_APPOINTMENT_POLICY,
  });
}

function validDate(value: string): boolean {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const parsed = new Date(`${value}T00:00:00Z`);
  return !Number.isNaN(parsed.getTime()) && parsed.toISOString().slice(0, 10) === value;
}

function parseLocalMinutes(value: string): number | null {
  const match = /^(\d{2}):(\d{2})$/.exec(value);
  if (!match) return null;
  const hours = Number(match[1]);
  const minutes = Number(match[2]);
  if (hours > 23 || minutes > 59) return null;
  return hours * 60 + minutes;
}

function isAppointmentPolicy(value: unknown): value is AppointmentPolicy {
  if (!value || typeof value !== "object") return false;
  const policy = value as Partial<AppointmentPolicy>;
  return (
    typeof policy.capability_name === "string" &&
    typeof policy.service === "string" &&
    Array.isArray(policy.allowed_weekdays) &&
    policy.allowed_weekdays.every((day) => Number.isInteger(day) && day >= 0 && day <= 6) &&
    typeof policy.earliest_local_time === "string" &&
    typeof policy.maximum_price_minor === "number" &&
    Number.isSafeInteger(policy.maximum_price_minor) &&
    policy.maximum_price_minor >= 0 &&
    typeof policy.currency === "string" &&
    Array.isArray(policy.allowed_extras) &&
    policy.allowed_extras.every((extra) => typeof extra === "string") &&
    Array.isArray(policy.forbidden_commitments) &&
    policy.forbidden_commitments.every((item) => typeof item === "string")
  );
}

/**
 * Evaluate untrusted offered terms against the immutable appointment policy.
 * This is the host-side mapping around the Tethers result: it never changes
 * the envelope and never treats provider language as permission.
 */
export function evaluateAppointmentTerms(
  envelope: AuthorityEnvelope,
  terms: unknown
): AuthorityResult {
  if (!verifyEnvelopeDigest(envelope)) {
    return { decision: "DENY", reason: "envelope digest mismatch; envelope integrity unverifiable" };
  }
  const policy = envelope.appointment_policy;
  if (!isAppointmentPolicy(policy)) return { decision: "DENY", reason: "appointment policy is missing or malformed in the frozen envelope" };
  if (!terms || typeof terms !== "object") {
    return { decision: "DENY", reason: "malformed appointment terms; expected an object" };
  }

  const candidate = terms as Partial<AppointmentTerms>;
  if (
    typeof candidate.offered_date !== "string" ||
    typeof candidate.offered_time !== "string" ||
    typeof candidate.offered_price_minor !== "number" ||
    !Number.isSafeInteger(candidate.offered_price_minor) ||
    candidate.offered_price_minor < 0 ||
    typeof candidate.currency !== "string" ||
    typeof candidate.service !== "string" ||
    !(candidate.extra_requested === null || typeof candidate.extra_requested === "string")
  ) {
    return { decision: "DENY", reason: "malformed appointment terms; required fields are missing or invalid" };
  }

  const forbidden = new Set(policy.forbidden_commitments.map((item) => item.toLowerCase()));
  if (
    (candidate.commitment_kind !== undefined && typeof candidate.commitment_kind !== "string") ||
    (candidate.subscription !== undefined && typeof candidate.subscription !== "boolean") ||
    (candidate.deposit_minor !== undefined &&
      candidate.deposit_minor !== null &&
      (!Number.isSafeInteger(candidate.deposit_minor) || candidate.deposit_minor < 0))
  ) {
    return { decision: "DENY", reason: "malformed appointment commitment fields" };
  }
  if (candidate.subscription === true || (candidate.commitment_kind && forbidden.has(candidate.commitment_kind.toLowerCase()))) {
    return { decision: "DENY", reason: "explicitly forbidden commitment offered" };
  }
  if (candidate.extra_requested && forbidden.has(candidate.extra_requested.toLowerCase())) {
    return { decision: "DENY", reason: "explicitly forbidden commitment offered as an extra" };
  }
  if (candidate.deposit_minor !== undefined && candidate.deposit_minor !== null && candidate.deposit_minor > 0) {
    return { decision: "DENY", reason: "deposit is explicitly forbidden by the frozen authority" };
  }
  if (!validDate(candidate.offered_date)) {
    return { decision: "DENY", reason: "offered date is malformed" };
  }
  if (candidate.currency !== policy.currency) {
    return { decision: "ASK", reason: `currency ${candidate.currency} is outside the frozen ${policy.currency} authority` };
  }
  if (candidate.service !== policy.service) {
    return { decision: "ASK", reason: `service ${candidate.service} differs from the authorised standard service` };
  }
  const time = parseLocalMinutes(candidate.offered_time);
  if (time === null) return { decision: "DENY", reason: "offered time is malformed" };
  const weekday = new Date(`${candidate.offered_date}T00:00:00Z`).getUTCDay();
  if (!policy.allowed_weekdays.includes(weekday) || time < (parseLocalMinutes(policy.earliest_local_time) ?? 0)) {
    return { decision: "ASK", reason: "date or time is outside the weekday-after-1pm authority" };
  }
  if (candidate.extra_requested !== null && candidate.extra_requested.trim() !== "") {
    if (!policy.allowed_extras.includes(candidate.extra_requested)) {
      return { decision: "ASK", reason: `unrequested extra offered: ${candidate.extra_requested}` };
    }
  }
  if (candidate.offered_price_minor > policy.maximum_price_minor) {
    return { decision: "ASK", reason: `price ${candidate.offered_price_minor} exceeds maximum ${policy.maximum_price_minor}` };
  }
  return { decision: "ALLOW", reason: "offered terms are inside the frozen appointment authority" };
}

/** Build the Tethers 0.1 request that represents the pre-call assessment. */
export function buildAppointmentTethersRequest(
  envelope: AuthorityEnvelope,
  terms: AppointmentTerms,
  evaluation_id = "callpermit-evaluation-1"
): TetherRequest {
  const policy = envelope.appointment_policy ?? DEMO_APPOINTMENT_POLICY;
  return {
    protocol_version: "0.1",
    language_version: "0.1",
    evaluation_id,
    tether: {
      id: "callpermit-appointment-authority",
      version: "1",
      source: `tether "CallPermit appointment authority"\n\nanchor\n    appointment.offer\n\ndo\n    appointment.book\n        date: anchor.offered_date\n        time: anchor.offered_time\n        price_minor: anchor.offered_price_minor\n`,
    },
    event: {
      id: `${evaluation_id}-event`,
      name: "appointment.offer",
      data: { ...terms },
    },
    facts: {
      "appointment.date": terms.offered_date,
      "appointment.time": terms.offered_time,
      "appointment.price_minor": terms.offered_price_minor,
      "appointment.currency": terms.currency,
      "appointment.service": terms.service,
      "appointment.extra_requested": terms.extra_requested,
      "callpermit.authority_digest": envelope.digest,
    },
    capabilities: [
      {
        name: policy.capability_name,
        version: "1.0.0",
        inputs: { date: "string", time: "string", price_minor: "integer" },
        effects: ["appointment.book"],
        reversibility: "compensatable",
      },
    ],
  };
}

/** Combine the real Tethers 0.7 response with bounded appointment facts. */
export function evaluateCallAuthority(
  envelope: AuthorityEnvelope,
  response: TetherResponse,
  terms: unknown
): AuthorityResult {
  const tethers = evaluateAuthority(envelope, response);
  if (tethers.decision === "DENY") return tethers;
  const termsResult = evaluateAppointmentTerms(envelope, terms);
  if (termsResult.decision === "DENY") return termsResult;
  if (tethers.decision === "ASK" || termsResult.decision === "ASK") {
    return {
      decision: "ASK",
      reason: `${termsResult.reason}; Tethers preflight is ${tethers.decision}`,
      action_results: tethers.action_results,
      trail: tethers.trail,
    };
  }
  return {
    decision: "ALLOW",
    reason: `Tethers preflight ALLOW; ${termsResult.reason}`,
    action_results: tethers.action_results,
    trail: tethers.trail,
  };
}
