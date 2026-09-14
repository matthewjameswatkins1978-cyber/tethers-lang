// CallPermit orchestration for one atomic appointment call.
// The Tethers response is supplied by the current Tethers 0.7 machine path;
// the CALL-E transport is injected so this build can use the offline fake.

import {
  buildAppointmentTethersRequest,
  evaluateCallAuthority,
  validateRequest,
  validateRequestCapabilities,
  validateResponseIdentity,
} from "./authority.js";
import { dispatchCall, type CreateCallOptions } from "./calle.js";
import { reconcileCallResult, type ReconciliationResult } from "./reconcile.js";
import type {
  AppointmentTerms,
  AuthorityEnvelope,
  AuthorityResult,
  CalleClient,
  CalleCredentials,
  CallParams,
  CallRegistry,
  CallResult,
  TetherResponse,
} from "./types.js";

export interface CallPermitRun {
  authority: AuthorityResult;
  call_params: CallParams;
  call_result: CallResult | null;
  reconciliation: ReconciliationResult | null;
}

export interface RunAppointmentOptions {
  envelope: AuthorityEnvelope;
  terms: AppointmentTerms;
  tethers_response: TetherResponse;
  credentials: CalleCredentials;
  registry: CallRegistry;
  destination: string;
  recipient_region?: string;
  recipient_locale?: string;
  prompt_override?: string;
  timeout_ms?: number;
  evaluation_id?: string;
  client: CalleClient;
}

/** Compile one bounded call task from the frozen envelope. */
export function compileAppointmentCall(
  envelope: AuthorityEnvelope,
  terms: AppointmentTerms,
  destination: string,
  recipient: { region?: string; locale?: string } = {},
  overrides: { prompt?: string; timeout_ms?: number } = {},
): CallParams {
  const policy = envelope.appointment_policy;
  if (!policy) throw new Error("cannot compile a call without appointment policy");
  return {
    to: destination,
    ...(recipient.region ? { region: recipient.region } : {}),
    ...(recipient.locale ? { locale: recipient.locale } : {}),
    capability: policy.capability_name,
    capability_version: "1.0.0",
    prompt: overrides.prompt ?? [
      `Arrange one ${policy.service} appointment.`,
      `Offer only ${terms.offered_date} at or after ${policy.earliest_local_time}.`,
      `Do not agree above ${policy.maximum_price_minor} pence ${policy.currency}.`,
      "Do not accept extras, subscriptions, memberships, deposits, purchases, or materially different commitments.",
      "If the recipient proposes anything outside these terms, do not commit; say you need to check with Matthew and return the terms as evidence.",
    ].join(" "),
    ...(overrides.timeout_ms !== undefined ? { timeout_ms: overrides.timeout_ms } : {}),
    metadata: {
      authority_digest: envelope.digest,
      authority_policy_id: envelope.policy_id,
      atomic_call: true,
    },
  };
}

/**
 * Run the preflight -> one injected CALL-E transport -> reconciliation path.
 * A non-ALLOW Tethers decision never reaches the transport.
 */
export async function runAppointmentCall(options: RunAppointmentOptions): Promise<CallPermitRun> {
  const request = buildAppointmentTethersRequest(
    options.envelope,
    options.terms,
    options.evaluation_id
  );
  const requestValidation = validateRequest(request);
  if (!requestValidation.valid) {
    const authority: AuthorityResult = {
      decision: "DENY",
      reason: `malformed Tethers request: ${requestValidation.errors.join("; ")}`,
    };
    return {
      authority,
      call_params: compileAppointmentCall(options.envelope, options.terms, options.destination, {
        region: options.recipient_region,
        locale: options.recipient_locale,
      }, { prompt: options.prompt_override, timeout_ms: options.timeout_ms }),
      call_result: null,
      reconciliation: null,
    };
  }
  const capabilityValidation = validateRequestCapabilities(request, options.envelope);
  if (!capabilityValidation.valid) {
    const authority: AuthorityResult = {
      decision: "DENY",
      reason: `Tethers request capability is outside the frozen envelope: ${capabilityValidation.errors.join("; ")}`,
    };
    return {
      authority,
      call_params: compileAppointmentCall(options.envelope, options.terms, options.destination, {
        region: options.recipient_region,
        locale: options.recipient_locale,
      }, { prompt: options.prompt_override, timeout_ms: options.timeout_ms }),
      call_result: null,
      reconciliation: null,
    };
  }
  const responseValidation = validateResponseIdentity(request, options.tethers_response);
  if (!responseValidation.valid) {
    const authority: AuthorityResult = {
      decision: "DENY",
      reason: `Tethers response identity mismatch: ${responseValidation.errors.join("; ")}`,
    };
    return {
      authority,
      call_params: compileAppointmentCall(options.envelope, options.terms, options.destination, {
        region: options.recipient_region,
        locale: options.recipient_locale,
      }, { prompt: options.prompt_override, timeout_ms: options.timeout_ms }),
      call_result: null,
      reconciliation: null,
    };
  }
  const authority = evaluateCallAuthority(
    options.envelope,
    options.tethers_response,
    options.terms
  );
  const call_params = compileAppointmentCall(
    options.envelope,
    options.terms,
    options.destination,
    {
      region: options.recipient_region,
      locale: options.recipient_locale,
    },
    { prompt: options.prompt_override, timeout_ms: options.timeout_ms },
  );
  if (authority.decision !== "ALLOW") {
    return { authority, call_params, call_result: null, reconciliation: null };
  }

  const dispatchOptions: CreateCallOptions = { authority_digest: options.envelope.digest };
  const call_result = await dispatchCall(
    options.client,
    options.registry,
    options.credentials,
    call_params,
    dispatchOptions
  );
  const reconciliation =
    call_result.status === "completed"
      ? reconcileCallResult(options.envelope, call_result)
      : null;
  return { authority, call_params, call_result, reconciliation };
}

/** Build the response shape returned by a Tethers 0.7 evaluation fixture. */
export function buildMatchedTethersResponse(
  envelope: AuthorityEnvelope,
  terms: AppointmentTerms,
  evaluation_id = "callpermit-evaluation-1"
): TetherResponse {
  const request = buildAppointmentTethersRequest(envelope, terms, evaluation_id);
  return {
    protocol_version: request.protocol_version,
    evaluation_id: request.evaluation_id,
    event_id: request.event.id,
    tether_id: request.tether.id,
    tether_version: request.tether.version,
    status: "matched",
    plan: {
      id: `${evaluation_id}/plan`,
      required_effects: ["appointment.book"],
      actions: [
        {
          action_id: `${evaluation_id}/appointment-book`,
          idempotency_key: `${evaluation_id}/appointment-book`,
          capability: "appointment.book",
          capability_version: "1.0.0",
          arguments: {
            date: terms.offered_date,
            time: terms.offered_time,
            price_minor: terms.offered_price_minor,
          },
          effects: ["appointment.book"],
        },
      ],
    },
    trail: [
      { sequence: 1, phase: "reception", kind: "event_received", outcome: "accepted", message: "Received appointment.offer" },
      { sequence: 2, phase: "evaluation", kind: "anchor_checked", outcome: "matched", message: "CallPermit appointment authority matched" },
      { sequence: 3, phase: "evaluation", kind: "action_planned", outcome: "accepted", message: "Planned appointment.book" },
    ],
  };
}
