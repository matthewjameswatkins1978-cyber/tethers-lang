// CALL-E runtime adapter — focused test suite
// Covers: no-network operation, duplicate-dispatch prevention,
// credential/phone guards, persisted identity, and restart safety.
// Uses Node.js built-in test runner (node:test + node:assert).

import { describe, it } from "node:test";
import assert from "node:assert/strict";

import {
  computeCallIdentity,
  createCallRegistry,
  deserialiseRegistry,
  dispatchCall,
  guardCredentials,
  guardPhoneNumber,
  serialiseRegistry,
} from "../src/calle.js";
import { createFakeCalleClient } from "../src/fake-calle.js";
import { CalleSdkClient, mapOfficialCall } from "../src/calle-sdk.js";
import {
  CALLE_OFFICIAL_TEST_HOTLINE,
  LIVE_PROOF_PROMPT,
  LIVE_PROOF_RESULT_SCHEMA,
  requireOfficialHotlineProfile,
} from "../src/live-proof.js";
import type { CalleCredentials, CallParams, CallResult, CallRecord } from "../src/types.js";

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

const CREDS: CalleCredentials = {
  api_key: "test-api-key-abc123",
  phone_number: "+14155551234",
};

const PARAMS: CallParams = {
  to: "+14155559999",
  capability: "call.verify_identity",
  prompt: "Please verify the caller's identity.",
};

// ---------------------------------------------------------------------------
// Deterministic call identity
// ---------------------------------------------------------------------------

describe("computeCallIdentity", () => {
  it("produces a stable identity for the same inputs", () => {
    const id1 = computeCallIdentity(CREDS.phone_number, PARAMS);
    const id2 = computeCallIdentity(CREDS.phone_number, PARAMS);
    assert.equal(id1, id2, "same inputs must yield same identity");
    assert.ok(id1.startsWith("call:"), "identity must be prefixed with 'call:'");
  });

  it("different destinations yield different identities", () => {
    const p2: CallParams = { ...PARAMS, to: "+12125550000" };
    const id1 = computeCallIdentity(CREDS.phone_number, PARAMS);
    const id2 = computeCallIdentity(CREDS.phone_number, p2);
    assert.notEqual(id1, id2, "different 'to' must yield different identity");
  });

  it("different capabilities yield different identities", () => {
    const p2: CallParams = { ...PARAMS, capability: "call.schedule" };
    const id1 = computeCallIdentity(CREDS.phone_number, PARAMS);
    const id2 = computeCallIdentity(CREDS.phone_number, p2);
    assert.notEqual(id1, id2, "different capability must yield different identity");
  });

  it("different source phones yield different identities", () => {
    const id1 = computeCallIdentity("+14155551234", PARAMS);
    const id2 = computeCallIdentity("+14155559999", PARAMS);
    assert.notEqual(id1, id2, "different from must yield different identity");
  });

  it("is case-insensitive on phone numbers", () => {
    const id1 = computeCallIdentity("+14155551234", PARAMS);
    const id2 = computeCallIdentity("+14155551234", PARAMS);
    assert.equal(id1, id2, "identity must be stable");
  });
});

// ---------------------------------------------------------------------------
// Credential guards
// ---------------------------------------------------------------------------

describe("guardCredentials", () => {
  it("passes for valid credentials", () => {
    const r = guardCredentials(CREDS);
    assert.ok(r.ok);
    assert.equal(r.errors.length, 0);
  });

  it("rejects null", () => {
    const r = guardCredentials(null);
    assert.ok(!r.ok);
    assert.ok(r.errors.length > 0);
  });

  it("rejects empty api_key", () => {
    const r = guardCredentials({ api_key: "", phone_number: "+14155551234" });
    assert.ok(!r.ok);
    assert.ok(r.errors.some((e) => e.includes("api_key")));
  });

  it("rejects empty phone_number", () => {
    const r = guardCredentials({ api_key: "key", phone_number: "" });
    assert.ok(!r.ok);
    assert.ok(r.errors.some((e) => e.includes("phone_number")));
  });
});

describe("guardPhoneNumber", () => {
  it("passes for valid E.164 number", () => {
    const r = guardPhoneNumber("+14155551234");
    assert.ok(r.ok);
  });

  it("rejects number without leading plus", () => {
    const r = guardPhoneNumber("14155551234");
    assert.ok(!r.ok);
    assert.ok(r.errors.some((e) => e.includes("+")));
  });

  it("rejects too-short number", () => {
    const r = guardPhoneNumber("+12345");
    assert.ok(!r.ok);
    assert.ok(r.errors.some((e) => e.includes("7 digits")));
  });

  it("rejects null", () => {
    const r = guardPhoneNumber(null);
    assert.ok(!r.ok);
  });
});

// ---------------------------------------------------------------------------
// Call registry
// ---------------------------------------------------------------------------

describe("CallRegistry", () => {
  it("insert and get round-trips", () => {
    const reg = createCallRegistry();
    const rec: CallRecord = {
      identity: "call:abc",
      params: PARAMS,
      status: "pending",
      from: CREDS.phone_number,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    };
    reg.insert(rec);
    const got = reg.get("call:abc");
    assert.ok(got);
    assert.equal(got.status, "pending");
  });

  it("throws on duplicate insert", () => {
    const reg = createCallRegistry();
    const rec: CallRecord = {
      identity: "call:dup",
      params: PARAMS,
      status: "pending",
      from: CREDS.phone_number,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    };
    reg.insert(rec);
    assert.throws(() => reg.insert(rec), /already exists/);
  });

  it("throws on update of missing identity", () => {
    const reg = createCallRegistry();
    assert.throws(() => reg.update("call:missing", { status: "completed" }), /not found/);
  });

  it("update changes status", () => {
    const reg = createCallRegistry();
    const rec: CallRecord = {
      identity: "call:update-test",
      params: PARAMS,
      status: "pending",
      from: CREDS.phone_number,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    };
    reg.insert(rec);
    reg.update("call:update-test", { status: "in_progress" });
    assert.equal(reg.get("call:update-test")!.status, "in_progress");
  });

  it("all() returns all records", () => {
    const reg = createCallRegistry();
    reg.insert({
      identity: "call:r1",
      params: PARAMS,
      status: "pending",
      from: CREDS.phone_number,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    });
    reg.insert({
      identity: "call:r2",
      params: PARAMS,
      status: "completed",
      from: CREDS.phone_number,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    });
    assert.equal(reg.all().length, 2);
  });
});

// ---------------------------------------------------------------------------
// Serialise / deserialise registry persistence
// ---------------------------------------------------------------------------

describe("registry persistence (serialise/deserialise)", () => {
  it("round-trips through JSON", () => {
    const reg = createCallRegistry();
    reg.insert({
      identity: "call:persist1",
      params: PARAMS,
      status: "completed",
      from: CREDS.phone_number,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
      result: { call_id: "api-123", status: "completed" },
    });

    const serialised = serialiseRegistry(reg);
    const json = JSON.stringify(serialised);
    const parsed = JSON.parse(json) as CallRecord[];
    const restored = deserialiseRegistry(parsed);

    assert.equal(restored.all().length, 1);
    const got = restored.get("call:persist1");
    assert.ok(got);
    assert.equal(got.status, "completed");
    assert.equal(got.result?.call_id, "api-123");
  });
});

// ---------------------------------------------------------------------------
// dispatchCall — no-network tests
// ---------------------------------------------------------------------------

describe("dispatchCall (no-network)", () => {
  it("returns failed for missing credentials", async () => {
    const client = createFakeCalleClient();
    const reg = createCallRegistry();
    const result = await dispatchCall(client, reg, null as any, PARAMS);
    assert.equal(result.status, "failed");
    assert.equal(result.error?.code, "INVALID_CREDENTIALS");
    assert.equal(client.callCount, 0, "no call should be dispatched");
  });

  it("returns failed for invalid destination phone", async () => {
    const client = createFakeCalleClient();
    const reg = createCallRegistry();
    const badParams: CallParams = { ...PARAMS, to: "not-a-phone" };
    const result = await dispatchCall(client, reg, CREDS, badParams);
    assert.equal(result.status, "failed");
    assert.equal(result.error?.code, "INVALID_PHONE");
    assert.equal(client.callCount, 0, "no call should be dispatched");
  });

  it("dispatches a new call and persists result", async () => {
    const client = createFakeCalleClient();
    const reg = createCallRegistry();
    const result = await dispatchCall(client, reg, CREDS, PARAMS);

    assert.equal(result.status, "completed");
    assert.equal(client.callCount, 1);

    // Registry should have the record
    const identity = computeCallIdentity(CREDS.phone_number, PARAMS);
    const record = reg.get(identity);
    assert.ok(record);
    assert.equal(record.status, "completed");
    assert.ok(record.result);
  });

  it("returns completed result from cache on duplicate dispatch", async () => {
    const client = createFakeCalleClient();
    const reg = createCallRegistry();

    // First call
    const r1 = await dispatchCall(client, reg, CREDS, PARAMS);
    assert.equal(r1.status, "completed");
    assert.equal(client.callCount, 1);

    // Second call with identical params — should NOT dispatch again
    const r2 = await dispatchCall(client, reg, CREDS, PARAMS);
    assert.equal(r2.status, "completed");
    assert.equal(client.callCount, 1, "duplicate must not trigger a second API call");
    assert.equal(r2.call_id, r1.call_id, "should return same result");
  });

  it("allows retry after a failed call", async () => {
    // First attempt: fails
    const failingClient = createFakeCalleClient({
      failWith: { code: "NETWORK_ERROR", message: "connection refused" },
    });
    const reg = createCallRegistry();
    const r1 = await dispatchCall(failingClient, reg, CREDS, PARAMS);
    assert.equal(r1.status, "failed");
    assert.equal(failingClient.callCount, 1);

    // Second attempt: succeeds (fresh client)
    const successClient = createFakeCalleClient();
    const r2 = await dispatchCall(successClient, reg, CREDS, PARAMS);
    assert.equal(r2.status, "completed");
    assert.equal(successClient.callCount, 1);
  });

  it("does not allow duplicate dispatch while call is in_progress", async () => {
    // Client that hangs (never resolves)
    let resolveFn: ((v: CallResult) => void) | null = null;
    const hangingClient = createFakeCalleClient();
    // Override createAndWait to hang
    hangingClient.createAndWait = () =>
      new Promise<CallResult>((resolve) => {
        resolveFn = resolve;
      }) as any;

    const reg = createCallRegistry();

    // Start first call (will hang)
    const p1 = dispatchCall(hangingClient, reg, CREDS, PARAMS);

    // Try second call immediately — should detect in_progress
    const r2 = await dispatchCall(hangingClient, reg, CREDS, PARAMS);
    assert.equal(r2.status, "in_progress");
    assert.ok(r2.error?.code === "CALL_IN_PROGRESS");

    // Resolve the hanging call
    resolveFn!({
      call_id: "api-hang-1",
      status: "completed",
      transcript: "done",
      duration_ms: 5000,
    });

    const r1 = await p1;
    assert.equal(r1.status, "completed");
  });

  it("handles client throw as failed dispatch", async () => {
    const throwingClient = createFakeCalleClient({ throwOnDispatch: true });
    const reg = createCallRegistry();
    const result = await dispatchCall(throwingClient, reg, CREDS, PARAMS);

    assert.equal(result.status, "failed");
    assert.equal(result.error?.code, "DISPATCH_ERROR");
    assert.ok(result.error?.message.includes("simulated dispatch failure"));
  });

  it("respects timeout_ms override", async () => {
    const client = createFakeCalleClient({ delay_ms: 100 });
    const reg = createCallRegistry();
    const result = await dispatchCall(client, reg, CREDS, PARAMS, { timeout_ms: 500 });
    assert.equal(result.status, "completed");
  });
});

// ---------------------------------------------------------------------------
// Restart safety
// ---------------------------------------------------------------------------

describe("restart safety", () => {
  it("restarting with same params returns cached result, no duplicate call", async () => {
    const client = createFakeCalleClient();
    const reg = createCallRegistry();

    // Simulate first run: call completes
    await dispatchCall(client, reg, CREDS, PARAMS);
    assert.equal(client.callCount, 1);

    // Simulate restart: new registry loaded from persistence
    const records = serialiseRegistry(reg);
    const restoredReg = deserialiseRegistry(records);

    // Second run with same params: should NOT call API again
    const result = await dispatchCall(client, restoredReg, CREDS, PARAMS);
    assert.equal(result.status, "completed");
    assert.equal(client.callCount, 1, "restart must not re-dispatch");
    assert.ok(result.call_id, "should have a call_id");
  });

  it("restarting after a provider ID exists never dispatches a duplicate", async () => {
    // A timeout after provider allocation is uncertain, not a safe retry.
    const client1 = createFakeCalleClient({
      fixedResult: {
        call_id: "api-existing-1",
        status: "timed_out",
        error: { code: "TIMEOUT", message: "call timed out" },
      },
    });
    const reg = createCallRegistry();
    await dispatchCall(client1, reg, CREDS, PARAMS);

    // Simulate restart
    const records = serialiseRegistry(reg);
    const restoredReg = deserialiseRegistry(records);

    // Second run: a new client must not be called for the same identity.
    const client2 = createFakeCalleClient();
    const result = await dispatchCall(client2, restoredReg, CREDS, PARAMS);
    assert.equal(result.status, "timed_out");
    assert.equal(result.call_id, "api-existing-1");
    assert.equal(client2.callCount, 0, "existing provider ID must prevent duplicate dispatch");
  });

  it("does not duplicate calls for different capabilities", async () => {
    const client = createFakeCalleClient();
    const reg = createCallRegistry();

    await dispatchCall(client, reg, CREDS, { ...PARAMS, capability: "call.verify" });
    await dispatchCall(client, reg, CREDS, { ...PARAMS, capability: "call.schedule" });

    assert.equal(client.callCount, 2, "different capabilities should each dispatch");
  });
});

describe("official CALL-E SDK adapter", () => {
  it("maps structured recipient evidence and sends the current SDK request shape", async () => {
    let receivedInput: Record<string, unknown> | undefined;
    let receivedOptions: Record<string, unknown> | undefined;
    const client = new CalleSdkClient({
      client: {
        calls: {
          async createAndWait(input, options) {
            receivedInput = input;
            receivedOptions = options;
            return {
              id: "calle-call-42",
              object: "call_task",
              status: "completed",
              task: input.task,
              recipients: [{
                id: "recipient-1",
                phones: [PARAMS.to],
                locale: null,
                region: null,
                status: "completed",
                summary: "Appointment confirmed.",
                structuredResult: {
                  outcome: "COMMITTED",
                  offered_date: "2026-09-15",
                  offered_time: "15:30",
                  offered_price_minor: 4500,
                  currency: "GBP",
                  service: "standard service",
                  commitment_made: true,
                  recipient_words_supporting_result: ["Confirmed for Tuesday."],
                },
                attempts: [{
                  id: "attempt-1",
                  phone: PARAMS.to,
                  status: "completed",
                  startedAt: null,
                  completedAt: null,
                  summary: null,
                  transcriptTurns: [{ speaker: "user", text: "Confirmed for Tuesday.", offset_seconds: null }],
                  providerCallId: "provider-42",
                  failureCode: null,
                  failureMessage: null,
                }],
              }],
              structuredResult: null,
              summary: "Appointment confirmed.",
              taskCompleted: true,
              completionConfidence: { score: 0.99, label: "high" },
              evidence: ["recipient confirmed the appointment"],
              metadata: {},
              failureCode: null,
              failureMessage: null,
              createdAt: "2026-09-15T14:00:00Z",
              completedAt: "2026-09-15T14:02:00Z",
            };
          },
        },
      },
    });

    const result = await client.createAndWait(
      { api_key: "test-key", phone_number: CREDS.phone_number },
      { ...PARAMS, metadata: { authority_digest: "sha256:test" } },
      { timeout_ms: 12_000 },
    );

    assert.equal(result.call_id, "calle-call-42");
    assert.equal(result.status, "completed");
    assert.equal(result.outcome, "COMMITTED");
    assert.equal(result.offered_price_minor, 4500);
    assert.match(result.transcript ?? "", /Confirmed for Tuesday/);
    assert.equal(receivedInput?.recipients?.[0]?.phones?.[0], PARAMS.to);
    assert.deepEqual(receivedInput?.metadata, { authority_digest: "sha256:test" });
    assert.equal(receivedOptions?.timeoutMs, 12_000);
    assert.match(String(receivedOptions?.idempotencyKey), /^callpermit-[0-9a-f]{16}$/);
  });

  it("maps cancellation and failure without treating task completion as commitment", () => {
    const result = mapOfficialCall({
      id: "calle-canceled-1",
      object: "call_task",
      status: "canceled",
      task: "test",
      recipients: [],
      structuredResult: { outcome: "COMMITTED", commitment_made: true },
      summary: null,
      taskCompleted: true,
      completionConfidence: null,
      evidence: [],
      metadata: {},
      failureCode: null,
      failureMessage: null,
      createdAt: "2026-09-15T14:00:00Z",
      completedAt: null,
    });
    assert.equal(result.status, "failed");
    assert.equal(result.error?.code, "CALL_CANCELLED");
    assert.equal(result.task_completed, true);
    assert.equal(result.outcome, "COMMITTED");
  });
});

describe("official hotline live-proof profile", () => {
  it("resolves only the explicitly selected public test profile", () => {
    assert.deepEqual(requireOfficialHotlineProfile("official-hotline"), CALLE_OFFICIAL_TEST_HOTLINE);
    assert.equal(CALLE_OFFICIAL_TEST_HOTLINE.phone, "+12763229632");
    assert.equal(CALLE_OFFICIAL_TEST_HOTLINE.region, "US");
    assert.equal(CALLE_OFFICIAL_TEST_HOTLINE.locale, "en-US");
    assert.throws(() => requireOfficialHotlineProfile(undefined), /refuse arbitrary live destinations/);
  });

  it("keeps the harmless task and proof schema explicit", () => {
    assert.match(LIVE_PROOF_PROMPT, /integration-test hotline/);
    assert.match(LIVE_PROOF_PROMPT, /Do not make appointments/);
    assert.deepEqual(LIVE_PROOF_RESULT_SCHEMA.required, ["connected", "test_response_observed"]);
  });
});
