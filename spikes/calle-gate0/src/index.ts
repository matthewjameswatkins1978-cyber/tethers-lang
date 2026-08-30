import { CalleClient, CalleAuthenticationError, CalleTimeoutError } from "@call-e/calle";
import crypto from "node:crypto";

export interface Gate0Config {
  apiKey?: string;
  testPhone?: string;
}

export function loadConfig(env: NodeJS.ProcessEnv = process.env): Gate0Config {
  return {
    apiKey: env.CALLE_API_KEY,
    testPhone: env.CALLE_TEST_PHONE,
  };
}

export function buildTestRequest(phone: string) {
  return {
    task: "Verify audio clarity and connection quality for Gate 0 diagnostic check.",
    recipient: {
      phone,
    },
    resultSchema: {
      type: "object",
      properties: {
        can_hear_clearly: {
          type: "string",
          enum: ["yes", "no", "unknown"],
          description: "Whether the audio was heard clearly during the call."
        }
      },
      required: ["can_hear_clearly"]
    }
  };
}

export function generateIdempotencyKey(): string {
  return crypto.randomUUID();
}

export async function runGate0Call(config: Gate0Config, options: { dryRun?: boolean } = {}) {
  if (!config.apiKey) {
    throw new Error("CALLE_API_KEY is missing or empty. Fail-safe triggered.");
  }
  if (!config.testPhone) {
    throw new Error("CALLE_TEST_PHONE is missing or empty. Fail-safe triggered.");
  }

  const client = new CalleClient({ apiKey: config.apiKey });
  const idempotencyKey = generateIdempotencyKey();
  const requestPayload = buildTestRequest(config.testPhone);

  if (options.dryRun) {
    return {
      success: true,
      mode: "dry-run",
      idempotencyKey,
      requestPayload,
    };
  }

  console.log(`[Gate0] Initiating call to ${config.testPhone.replace(/\d(?=\d{4})/g, "*")} with idempotency key ${idempotencyKey}`);
  
  try {
    // Create call with idempotency key and wait for result with safe timeout / retry prevention rules
    const call = await client.calls.createAndWait(requestPayload, {
      idempotencyKey,
      timeoutMs: 300000, // 5 minutes max wait
      intervalMs: 5000,   // poll every 5 seconds
    });

    return {
      success: true,
      mode: "live",
      callId: call.id,
      status: call.status,
      structuredResult: call.structuredResult,
      summary: call.summary,
      evidence: call.evidence,
    };
  } catch (error: any) {
    return {
      success: false,
      mode: "live",
      errorName: error.name,
      errorMessage: error.message,
    };
  }
}
