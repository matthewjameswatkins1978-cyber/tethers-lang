// CALL-E 0.7.0 SDK adapter.
//
// This is the only module that knows the official CALL-E wire shape.  The
// rest of CallPermit depends on the small transport-agnostic CalleClient
// interface, which keeps tests offline and keeps provider evidence separate
// from the frozen authority decision.

import { CalleClient as OfficialCalleClient } from "@call-e/calle";
import type {
  Call as OfficialCall,
  CreateCallInput,
  JsonObject,
} from "@call-e/calle";
import { computeCallIdentity } from "./calle.js";
import type {
  CalleClient,
  CalleCredentials,
  CallParams,
  CallResult,
} from "./types.js";

/** Structured result contract requested from CALL-E for one CallPermit call. */
export const CALLPERMIT_RESULT_SCHEMA: JsonObject = {
  type: "object",
  additionalProperties: false,
  properties: {
    outcome: { type: "string", enum: ["COMMITTED", "DEFERRED", "NO_MATCH", "FAILED"] },
    offered_date: { type: ["string", "null"] },
    offered_time: { type: ["string", "null"] },
    offered_price_minor: { type: ["integer", "null"] },
    currency: { type: ["string", "null"] },
    service: { type: ["string", "null"] },
    extra_requested: { type: ["string", "null"] },
    commitment_kind: { type: ["string", "null"] },
    subscription: { type: ["boolean", "null"] },
    deposit_minor: { type: ["integer", "null"] },
    commitment_made: { type: ["boolean", "null"] },
    defer_reason: { type: ["string", "null"] },
    recipient_words_supporting_result: { type: "array", items: { type: "string" } },
  },
  required: ["outcome"],
};

/** Minimal injectable surface used to test mapping without a network call. */
export interface CalleSdkClientLike {
    calls: {
      createAndWait(
        input: CreateCallInput,
      options?: { idempotencyKey?: string; timeoutMs?: number; intervalMs?: number },
      ): Promise<OfficialCall>;
  };
}

export interface CalleSdkClientOptions {
  /** Injected only for deterministic tests or a caller-owned configured client. */
  client?: CalleSdkClientLike;
  api_key?: string;
  base_url?: string;
  /** Explicit account compatibility mode; omission keeps structured extraction enabled. */
  result_schema_mode?: "recipient" | "none";
  /** Optional proof-specific schema; defaults to the normal CallPermit schema. */
  result_schema?: JsonObject;
}

/** A real CALL-E SDK transport implementing CallPermit's narrow interface. */
export class CalleSdkClient implements CalleClient {
  private readonly client: CalleSdkClientLike;
  private readonly resultSchemaMode: "recipient" | "none";
  private readonly resultSchema: JsonObject;

  constructor(options: CalleSdkClientOptions = {}) {
    this.resultSchemaMode = options.result_schema_mode ?? "recipient";
    this.resultSchema = options.result_schema ?? CALLPERMIT_RESULT_SCHEMA;
    if (options.client) {
      this.client = options.client;
      return;
    }
    if (!options.api_key || options.api_key.trim().length === 0) {
      throw new Error("CALL-E api_key is required to construct the live SDK client");
    }
    this.client = new OfficialCalleClient({
      apiKey: options.api_key,
      ...(options.base_url ? { baseUrl: options.base_url } : {}),
    });
  }

  async createAndWait(
    credentials: CalleCredentials,
    params: CallParams,
    options: { timeout_ms?: number } = {},
  ): Promise<CallResult> {
    const authorityDigest = typeof params.metadata?.authority_digest === "string"
      ? params.metadata.authority_digest
      : undefined;
    const identity = computeCallIdentity(credentials.phone_number, params, authorityDigest);
    const input: CreateCallInput = {
      task: params.prompt ?? `Call ${params.to} and complete the bounded ${params.capability} task.`,
      recipients: [{
        phones: [params.to],
        ...(params.region ? { region: params.region } : {}),
        ...(params.locale ? { locale: params.locale } : {}),
      }],
      metadata: params.metadata ?? {},
    };
    if (this.resultSchemaMode !== "none") {
      // The live 0.7.0 SDK exposes per-recipient extraction.  Some deployed
      // accounts reject both schema fields before call creation; callers must
      // opt into `none` explicitly rather than silently weakening evidence.
      input.recipientResultSchema = this.resultSchema;
    }
    const raw = await this.client.calls.createAndWait(input, {
      idempotencyKey: `callpermit-${identity.slice("call:".length)}`,
      ...(options.timeout_ms !== undefined ? { timeoutMs: options.timeout_ms } : {}),
    });
    return mapOfficialCall(raw);
  }
}

function mapStatus(status: OfficialCall["status"]): CallResult["status"] {
  switch (status) {
    case "queued":
      return "pending";
    case "in_progress":
      return "in_progress";
    case "completed":
      return "completed";
    case "failed":
      return "failed";
    case "canceled":
      return "failed";
    default:
      return "failed";
  }
}

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function nullableString(value: unknown): string | null | undefined {
  return value === null ? null : stringValue(value);
}

function nullableNumber(value: unknown): number | null | undefined {
  return value === null ? null : typeof value === "number" && Number.isSafeInteger(value) ? value : undefined;
}

function nullableBoolean(value: unknown): boolean | undefined {
  return value === null ? undefined : typeof value === "boolean" ? value : undefined;
}

function stringArray(value: unknown): string[] | undefined {
  return Array.isArray(value) && value.every((item) => typeof item === "string")
    ? value
    : undefined;
}

function transcriptOf(call: OfficialCall): string | undefined {
  const turns = call.recipients.flatMap((recipient) =>
    recipient.attempts.flatMap((attempt) => attempt.transcriptTurns),
  );
  if (turns.length === 0) return call.summary ?? undefined;
  return turns.map((turn) => `${turn.speaker}: ${turn.text}`).join("\n");
}

/** Convert the official provider object into the stable CallPermit result. */
export function mapOfficialCall(call: OfficialCall): CallResult {
  const structured = call.recipients.find((recipient) => recipient.structuredResult)?.structuredResult
    ?? call.structuredResult
    ?? {};
  const result: CallResult = {
    call_id: call.id,
    status: mapStatus(call.status),
    transcript: transcriptOf(call),
    completed_at: call.completedAt ?? undefined,
    task_completed: call.taskCompleted ?? undefined,
    outcome: stringValue(structured.outcome) as CallResult["outcome"],
    offered_date: nullableString(structured.offered_date),
    offered_time: nullableString(structured.offered_time),
    offered_price_minor: nullableNumber(structured.offered_price_minor),
    currency: nullableString(structured.currency),
    service: nullableString(structured.service),
    extra_requested: nullableString(structured.extra_requested),
    commitment_kind: stringValue(structured.commitment_kind),
    subscription: nullableBoolean(structured.subscription),
    deposit_minor: nullableNumber(structured.deposit_minor),
    commitment_made: nullableBoolean(structured.commitment_made),
    defer_reason: nullableString(structured.defer_reason),
    recipient_words_supporting_result: stringArray(structured.recipient_words_supporting_result),
    structured_result: structured as Record<string, unknown>,
    evidence: call.evidence,
    completion_confidence: call.completionConfidence as Record<string, unknown> | null,
  };
  if (call.status === "canceled") {
    result.error = { code: "CALL_CANCELLED", message: "CALL-E canceled the call" };
  } else if (call.failureCode || call.failureMessage) {
    result.error = {
      code: call.failureCode ?? "CALL_FAILED",
      message: call.failureMessage ?? "CALL-E reported a failed call",
    };
  }
  return result;
}
