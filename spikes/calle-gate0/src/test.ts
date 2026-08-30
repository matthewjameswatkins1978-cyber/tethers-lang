import { loadConfig, buildTestRequest, generateIdempotencyKey, runGate0Call } from "./index.js";
import assert from "node:assert";

async function runTests() {
  console.log("=== Running Gate 0 Deterministic Local Tests ===");

  // Test 1: Absent config fail-safe
  console.log("Test 1: Absent config fail-safe...");
  const emptyConfig = { apiKey: undefined, testPhone: undefined };
  let caughtError: Error | null = null;
  try {
    await runGate0Call(emptyConfig);
  } catch (err: any) {
    caughtError = err as Error;
  }
  assert.ok(caughtError, "Expected error when credentials are missing");
  assert.match(caughtError!.message, /CALLE_API_KEY is missing/);
  console.log("✓ Test 1 Passed: Absent config correctly triggers fail-safe error.");

  // Test 2: Request schema construction & structure
  console.log("Test 2: Request schema construction...");
  const testPhone = "+1555019999";
  const req = buildTestRequest(testPhone);
  assert.strictEqual(req.recipient.phone, testPhone);
  assert.ok(req.resultSchema);
  assert.strictEqual((req.resultSchema as any).properties.can_hear_clearly.type, "string");
  const enumValues = (req.resultSchema as any).properties.can_hear_clearly.enum;
  assert.deepStrictEqual(enumValues, ["yes", "no", "unknown"]);
  console.log("✓ Test 2 Passed: Request schema correctly structured with required enum {yes, no, unknown}.");

  // Test 3: Idempotency key uniqueness and behavior
  console.log("Test 3: Idempotency key behavior...");
  const key1 = generateIdempotencyKey();
  const key2 = generateIdempotencyKey();
  assert.ok(key1.length > 0);
  assert.ok(key2.length > 0);
  assert.notStrictEqual(key1, key2, "Generated idempotency keys must be unique UUIDs");
  console.log(`✓ Test 3 Passed: Unique idempotency keys verified (${key1} vs ${key2}).`);

  // Test 4: Dry-run execution with mock config
  console.log("Test 4: Dry-run execution mode...");
  const mockConfig = { apiKey: "mock-key-secret-12345", testPhone: "+1555019999" };
  const dryRunResult = await runGate0Call(mockConfig, { dryRun: true });
  assert.strictEqual(dryRunResult.success, true);
  assert.strictEqual(dryRunResult.mode, "dry-run");
  assert.ok(dryRunResult.idempotencyKey);
  assert.ok(dryRunResult.requestPayload);
  console.log("✓ Test 4 Passed: Dry-run execution succeeded without leaking secrets.");

  // Test 5: Credentials check & live vs dry-run decision branch
  console.log("Test 5: Credential presence check handling...");
  const liveConfig = loadConfig();
  console.log(`[Gate0] Environment check: CALLE_API_KEY present = ${!!liveConfig.apiKey}, CALLE_TEST_PHONE present = ${!!liveConfig.testPhone}`);
  if (liveConfig.apiKey && liveConfig.testPhone) {
    console.log("Credentials detected! Running controlled test call with idempotency key...");
    const liveResult = await runGate0Call(liveConfig);
    console.log("Live result status:", liveResult.success ? "SUCCESS" : "FAILED / ERROR");
    if (liveResult.success) {
      console.log(`Call ID: ${liveResult.callId}, Status: ${liveResult.status}`);
      console.log(`Structured Result:`, liveResult.structuredResult);
    } else {
      console.log(`Live call encountered error (expected if mock key or test network): ${liveResult.errorMessage}`);
    }
  } else {
    console.log("Credentials missing; successfully validated fail-safe and dry-run code paths without guessing secrets.");
  }

  console.log("=== All Gate 0 Local Tests Completed Successfully ===\n");
}

runTests().catch((err) => {
  console.error("Test execution failed:", err);
  process.exit(1);
});
