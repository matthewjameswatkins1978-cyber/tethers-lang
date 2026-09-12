import { describe, it } from "node:test";
import assert from "node:assert/strict";

import {
  buildAppointmentEnvelope,
  evaluateAppointmentTerms,
  evaluateCallAuthority,
} from "../src/authority.js";
import { createCallRegistry, dispatchCall } from "../src/calle.js";
import { buildMatchedTethersResponse, runAppointmentCall } from "../src/callpermit.js";
import { createFakeCalleClient } from "../src/fake-calle.js";
import { reconcileCallResult } from "../src/reconcile.js";
import type { AppointmentTerms, CalleCredentials, CallResult } from "../src/types.js";

const envelope = buildAppointmentEnvelope();
const credentials: CalleCredentials = {
  api_key: "offline-placeholder",
  phone_number: "+15555550100",
};

function terms(overrides: Partial<AppointmentTerms> = {}): AppointmentTerms {
  return {
    offered_date: "2026-09-15",
    offered_time: "15:30",
    offered_price_minor: 4500,
    currency: "GBP",
    service: "standard service",
    extra_requested: null,
    ...overrides,
  };
}

function responseResult(t: AppointmentTerms, overrides: Partial<CallResult> = {}): CallResult {
  return {
    call_id: "fake-call-result-1",
    status: "completed",
    outcome: "COMMITTED",
    commitment_made: true,
    task_completed: true,
    offered_date: t.offered_date,
    offered_time: t.offered_time,
    offered_price_minor: t.offered_price_minor,
    currency: t.currency,
    service: t.service,
    extra_requested: t.extra_requested,
    recipient_words_supporting_result: ["The recipient confirmed the terms."],
    ...overrides,
  };
}

describe("CallPermit authority fixtures", () => {
  it("ALLOW: Tuesday at 15:30 for £45 standard service", () => {
    const t = terms();
    const result = evaluateAppointmentTerms(envelope, t);
    assert.equal(result.decision, "ALLOW");
    assert.equal(evaluateCallAuthority(envelope, buildMatchedTethersResponse(envelope, t), t).decision, "ALLOW");
  });

  it("ASK: price above £60", () => {
    const t = terms({ offered_price_minor: 9500 });
    assert.equal(evaluateAppointmentTerms(envelope, t).decision, "ASK");
  });

  it("ASK: unrequested extra", () => {
    const t = terms({ extra_requested: "premium add-on" });
    assert.equal(evaluateAppointmentTerms(envelope, t).decision, "ASK");
  });

  it("DENY: forbidden subscription", () => {
    const t = terms({ commitment_kind: "subscription", subscription: true });
    assert.equal(evaluateAppointmentTerms(envelope, t).decision, "DENY");
  });

  it("DENY: subscription named as an extra", () => {
    const t = terms({ extra_requested: "subscription" });
    assert.equal(evaluateAppointmentTerms(envelope, t).decision, "DENY");
  });

  it("DENY: malformed banana/ferret input", () => {
    const result = evaluateAppointmentTerms(envelope, { banana: "ferret" });
    assert.equal(result.decision, "DENY");
  });
});

describe("CallPermit offline runtime", () => {
  it("does not dispatch when frozen authority returns ASK", async () => {
    const t = terms({ offered_price_minor: 9500 });
    const client = createFakeCalleClient();
    const result = await runAppointmentCall({
      envelope,
      terms: t,
      tethers_response: buildMatchedTethersResponse(envelope, t),
      credentials,
      registry: createCallRegistry(),
      destination: "+15555550101",
      client,
    });
    assert.equal(result.authority.decision, "ASK");
    assert.equal(result.call_result, null);
    assert.equal(client.callCount, 0);
  });

  it("reconciles an allowed fake commitment", async () => {
    const t = terms();
    const client = createFakeCalleClient({ fixedResult: responseResult(t) });
    const result = await runAppointmentCall({
      envelope,
      terms: t,
      tethers_response: buildMatchedTethersResponse(envelope, t),
      credentials,
      registry: createCallRegistry(),
      destination: "+15555550101",
      client,
    });
    assert.equal(result.call_result?.call_id, "fake-call-result-1");
    assert.equal(result.reconciliation?.status, "COMMITTED_WITHIN_AUTHORITY");
  });

  it("flags a £95 commitment as policy violation, never success", () => {
    const result = reconcileCallResult(
      envelope,
      responseResult(terms(), { offered_price_minor: 9500 })
    );
    assert.equal(result.status, "INVALID_OR_UNCERTAIN_RESULT");
    assert.match(result.reason, /POLICY VIOLATION/);
    assert.notEqual(result.status, "COMMITTED_WITHIN_AUTHORITY");
  });

  it("returns a decision card for a deferred extra", () => {
    const t = terms({ extra_requested: "premium add-on" });
    const result = reconcileCallResult(envelope, responseResult(t, {
      outcome: "DEFERRED",
      commitment_made: false,
      defer_reason: "The recipient offered a premium add-on.",
    }));
    assert.equal(result.status, "DEFERRED_NEEDS_MATTHEW");
    assert.equal(result.decision_card?.follow_up_call_created, false);
    assert.ok(result.decision_card?.new_authority_needed);
  });

  it("marks contradictory or incomplete evidence uncertain", () => {
    const result = reconcileCallResult(envelope, responseResult(terms(), {
      offered_price_minor: null,
      commitment_made: true,
    }));
    assert.equal(result.status, "INVALID_OR_UNCERTAIN_RESULT");
  });

  it("keeps the authority digest on the persisted atomic call", async () => {
    const t = terms();
    const registry = createCallRegistry();
    await runAppointmentCall({
      envelope,
      terms: t,
      tethers_response: buildMatchedTethersResponse(envelope, t),
      credentials,
      registry,
      destination: "+15555550101",
      client: createFakeCalleClient({ fixedResult: responseResult(t) }),
    });
    const record = registry.all()[0];
    assert.equal(record.authority_digest, envelope.digest);
    const changed = buildAppointmentEnvelope("callpermit.new-authority");
    assert.notEqual(changed.digest, record.authority_digest);
    assert.equal(record.authority_digest, envelope.digest);
  });

  it("uses a new identity for a genuinely new frozen envelope", async () => {
    const t = terms();
    const registry = createCallRegistry();
    const client = createFakeCalleClient({ fixedResult: responseResult(t) });
    await dispatchCall(client, registry, credentials, {
      to: "+15555550101", capability: "appointment.book",
    }, { authority_digest: envelope.digest });
    const nextEnvelope = buildAppointmentEnvelope("callpermit.follow-up");
    await dispatchCall(client, registry, credentials, {
      to: "+15555550101", capability: "appointment.book",
    }, { authority_digest: nextEnvelope.digest });
    assert.equal(client.callCount, 2);
  });

  it("rejects malformed Tethers responses without throwing", () => {
    const result = evaluateCallAuthority(envelope, { status: "unknown" } as never, terms());
    assert.equal(result.decision, "DENY");
  });

  it("rejects a Tethers response for a different evaluation", async () => {
    const t = terms();
    const response = buildMatchedTethersResponse(envelope, t, "other-evaluation");
    const client = createFakeCalleClient();
    const result = await runAppointmentCall({
      envelope,
      terms: t,
      tethers_response: response,
      credentials,
      registry: createCallRegistry(),
      destination: "+15555550101",
      client,
      evaluation_id: "expected-evaluation",
    });
    assert.equal(result.authority.decision, "DENY");
    assert.equal(client.callCount, 0);
  });
});
