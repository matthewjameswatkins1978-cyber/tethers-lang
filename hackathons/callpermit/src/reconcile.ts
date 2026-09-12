// Deterministic post-call reconciliation.
// Provider claims are evidence. The frozen AuthorityEnvelope remains decisive.

import { evaluateAppointmentTerms } from "./authority.js";
import type {
  AppointmentTerms,
  AuthorityDecision,
  AuthorityEnvelope,
  CallResult,
} from "./types.js";

export type ReconciliationStatus =
  | "COMMITTED_WITHIN_AUTHORITY"
  | "DEFERRED_NEEDS_MATTHEW"
  | "NO_COMMITMENT"
  | "INVALID_OR_UNCERTAIN_RESULT";

export interface DecisionCard {
  title: string;
  offered_terms: Partial<AppointmentTerms>;
  reason: string;
  evidence: string[];
  new_authority_needed: string;
  follow_up_call_created: false;
}

export interface ReconciliationResult {
  status: ReconciliationStatus;
  authority_decision: AuthorityDecision;
  reason: string;
  call_id: string;
  decision_card?: DecisionCard;
}

function offeredTerms(result: CallResult): AppointmentTerms | null {
  if (
    typeof result.offered_date !== "string" ||
    typeof result.offered_time !== "string" ||
    typeof result.offered_price_minor !== "number" ||
    typeof result.currency !== "string" ||
    typeof result.service !== "string" ||
    !(result.extra_requested === null || typeof result.extra_requested === "string")
  ) {
    return null;
  }
  return {
    offered_date: result.offered_date,
    offered_time: result.offered_time,
    offered_price_minor: result.offered_price_minor,
    currency: result.currency,
    service: result.service,
    extra_requested: result.extra_requested,
    commitment_kind: result.commitment_kind,
    subscription: result.subscription,
    deposit_minor: result.deposit_minor,
  };
}

function evidence(result: CallResult): string[] {
  const lines = result.recipient_words_supporting_result ?? [];
  return lines.length > 0 ? [...lines] : result.transcript ? [result.transcript] : [];
}

function card(
  result: CallResult,
  terms: Partial<AppointmentTerms>,
  reason: string,
  newAuthority: string
): DecisionCard {
  return {
    title: "CallPermit decision: Matthew must authorise",
    offered_terms: terms,
    reason,
    evidence: evidence(result),
    new_authority_needed: newAuthority,
    follow_up_call_created: false,
  };
}

/** Reconcile one terminal fake/real-shaped CALL-E result against one envelope. */
export function reconcileCallResult(
  envelope: AuthorityEnvelope,
  result: CallResult
): ReconciliationResult {
  if (!result || typeof result !== "object" || typeof result.call_id !== "string") {
    return {
      status: "INVALID_OR_UNCERTAIN_RESULT",
      authority_decision: "DENY",
      reason: "malformed CALL-E result; call identity is missing",
      call_id: "",
    };
  }
  if (result.status !== "completed") {
    return {
      status: "INVALID_OR_UNCERTAIN_RESULT",
      authority_decision: "DENY",
      reason: `CALL-E did not produce a terminal completed result: ${result.status}`,
      call_id: result.call_id,
    };
  }
  if (result.commitment_made === true && result.outcome !== "COMMITTED") {
    return {
      status: "INVALID_OR_UNCERTAIN_RESULT",
      authority_decision: "DENY",
      reason: "provider result contradicts itself: commitment_made=true but outcome is not COMMITTED",
      call_id: result.call_id,
    };
  }
  if (result.commitment_made === false && result.outcome === "COMMITTED") {
    return {
      status: "INVALID_OR_UNCERTAIN_RESULT",
      authority_decision: "DENY",
      reason: "provider result contradicts itself: outcome is COMMITTED but commitment_made=false",
      call_id: result.call_id,
    };
  }

  const terms = offeredTerms(result);
  if (!terms) {
    if (result.commitment_made === true || result.outcome === "COMMITTED") {
      return {
        status: "INVALID_OR_UNCERTAIN_RESULT",
        authority_decision: "DENY",
        reason: "CALL-E claims a commitment but did not return complete offered terms",
        call_id: result.call_id,
      };
    }
    return {
      status: result.outcome === "NO_MATCH" ? "NO_COMMITMENT" : "INVALID_OR_UNCERTAIN_RESULT",
      authority_decision: "ASK",
      reason: result.defer_reason ?? "CALL-E returned no complete terms to reconcile",
      call_id: result.call_id,
    };
  }

  const authority = evaluateAppointmentTerms(envelope, terms);
  if (result.commitment_made === true) {
    if (authority.decision !== "ALLOW") {
      const violation = `POLICY VIOLATION / INVALID RESULT: CALL-E claims commitment at ${terms.offered_price_minor} ${terms.currency}, but frozen authority returned ${authority.decision}`;
      return {
        status: "INVALID_OR_UNCERTAIN_RESULT",
        authority_decision: authority.decision,
        reason: violation,
        call_id: result.call_id,
        decision_card: card(result, terms, violation, authority.reason),
      };
    }
    return {
      status: "COMMITTED_WITHIN_AUTHORITY",
      authority_decision: "ALLOW",
      reason: "CALL-E commitment reconciles within the same frozen authority envelope",
      call_id: result.call_id,
    };
  }

  if (authority.decision === "ASK") {
    return {
      status: "DEFERRED_NEEDS_MATTHEW",
      authority_decision: "ASK",
      reason: result.defer_reason ?? authority.reason,
      call_id: result.call_id,
      decision_card: card(
        result,
        terms,
        result.defer_reason ?? authority.reason,
        `new envelope permitting the proposed ${terms.service} at ${terms.offered_price_minor} ${terms.currency}`
      ),
    };
  }
  if (authority.decision === "DENY") {
    return {
      status: "NO_COMMITMENT",
      authority_decision: "DENY",
      reason: `no commitment accepted: ${authority.reason}`,
      call_id: result.call_id,
    };
  }
  return {
    status: "NO_COMMITMENT",
    authority_decision: "ALLOW",
    reason: "CALL-E reported no commitment; offered terms were inside authority",
    call_id: result.call_id,
  };
}
