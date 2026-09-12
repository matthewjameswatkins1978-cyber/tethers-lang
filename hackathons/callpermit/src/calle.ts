// CALL-E runtime adapter
// Provides deterministic call identity, persisted call registry,
// credential/phone guards, and duplicate-dispatch prevention.
// No live calls are ever made by this module; all external API interaction
// is abstracted through the CalleClient interface.

import { createHash } from "node:crypto";
import type {
  CalleClient,
  CalleCredentials,
  CallParams,
  CallRecord,
  CallRegistry,
  CallResult,
  CallStatus,
} from "./types.js";

// ---------------------------------------------------------------------------
// Deterministic call identity
// ---------------------------------------------------------------------------

/**
 * Compute a deterministic identity string for a call from its parameters.
 * Same (from, to, capability) always yields the same identity.
 * This prevents duplicate dispatch after timeout or restart.
 */
export function computeCallIdentity(
  from: string,
  params: CallParams,
  authorityDigest?: string
): string {
  const payload = JSON.stringify({
    from: from.toLowerCase().trim(),
    to: params.to.toLowerCase().trim(),
    capability: params.capability.trim(),
    authority_digest: authorityDigest ?? null,
  });
  const hash = createHash("sha256").update(payload, "utf8").digest("hex");
  return `call:${hash.slice(0, 16)}`;
}

// ---------------------------------------------------------------------------
// Credential guards
// ---------------------------------------------------------------------------

/** Result of a credential guard check. */
export interface GuardResult {
  ok: boolean;
  errors: string[];
}

/**
 * Validate that a credentials object is present and well-formed.
 * Fails closed: missing or empty fields are errors.
 */
export function guardCredentials(creds: CalleCredentials | null | undefined): GuardResult {
  const errors: string[] = [];
  if (!creds || typeof creds !== "object") {
    return { ok: false, errors: ["credentials are required"] };
  }
  if (!creds.api_key || typeof creds.api_key !== "string" || creds.api_key.trim().length === 0) {
    errors.push("api_key is required and must be non-empty");
  }
  if (!creds.phone_number || typeof creds.phone_number !== "string" || creds.phone_number.trim().length === 0) {
    errors.push("phone_number is required and must be non-empty");
  }
  return { ok: errors.length === 0, errors };
}

/**
 * Validate that a phone number looks plausible.
 * Requires a leading '+' and at least 7 digits (E.164 minimum).
 */
export function guardPhoneNumber(phone: string | null | undefined): GuardResult {
  const errors: string[] = [];
  if (!phone || typeof phone !== "string") {
    return { ok: false, errors: ["phone number is required"] };
  }
  const trimmed = phone.trim();
  if (!trimmed.startsWith("+")) {
    errors.push("phone number must start with '+' (E.164 format)");
  }
  const digits = trimmed.replace(/[^0-9]/g, "");
  if (digits.length < 7) {
    errors.push("phone number must contain at least 7 digits");
  }
  return { ok: errors.length === 0, errors };
}

// ---------------------------------------------------------------------------
// In-memory CallRegistry (JSON-serializable, persistable)
// ---------------------------------------------------------------------------

/** Create an in-memory call registry. */
export function createCallRegistry(): CallRegistry {
  const records = new Map<string, CallRecord>();

  return {
    get(identity: string): CallRecord | undefined {
      return records.get(identity);
    },

    insert(record: CallRecord): void {
      if (records.has(record.identity)) {
        throw new Error(`call identity already exists: ${record.identity}`);
      }
      records.set(record.identity, { ...record });
    },

    update(identity: string, patch: Partial<CallRecord>): void {
      const existing = records.get(identity);
      if (!existing) {
        throw new Error(`call identity not found: ${identity}`);
      }
      records.set(identity, { ...existing, ...patch, updated_at: new Date().toISOString() });
    },

    all(): CallRecord[] {
      return Array.from(records.values());
    },
  };
}

// ---------------------------------------------------------------------------
// Serialise / deserialise registry for persistence
// ---------------------------------------------------------------------------

/** Serialise a registry to a JSON-safe array. */
export function serialiseRegistry(registry: CallRegistry): CallRecord[] {
  return registry.all();
}

/** Deserialise a JSON array back into a fresh registry. */
export function deserialiseRegistry(records: CallRecord[]): CallRegistry {
  const registry = createCallRegistry();
  for (const r of records) {
    registry.insert(r);
  }
  return registry;
}

// ---------------------------------------------------------------------------
// CalleClient with duplicate-dispatch prevention
// ---------------------------------------------------------------------------

/** Options for createCall. */
export interface CreateCallOptions {
  /** Timeout override in milliseconds. */
  timeout_ms?: number;
  /** Digest of the immutable authority used to compile this call. */
  authority_digest?: string;
}

/**
 * Attempt to create and wait for a call.
 *
 * Duplicate dispatch prevention:
 * - Computes a deterministic identity from (from, to, capability).
 * - If the registry already has a COMPLETED or IN_PROGRESS record with
 *   this identity, returns the existing result instead of making a new call.
 * - If a FAILED record has no provider call ID, allows retry.
 * - A TIMED_OUT/FAILED record with a provider call ID is never dispatched again.
 * - Otherwise creates a new record, dispatches, and persists the result.
 *
 * Never places a live call: the actual API interaction is delegated to the
 * supplied CalleClient implementation.
 */
export async function dispatchCall(
  client: CalleClient,
  registry: CallRegistry,
  credentials: CalleCredentials,
  params: CallParams,
  options?: CreateCallOptions
): Promise<CallResult> {
  // Guard: credentials
  const credGuard = guardCredentials(credentials);
  if (!credGuard.ok) {
    return {
      call_id: "",
      status: "failed",
      error: { code: "INVALID_CREDENTIALS", message: credGuard.errors.join("; ") },
    };
  }

  // Guard: destination phone
  const phoneGuard = guardPhoneNumber(params.to);
  if (!phoneGuard.ok) {
    return {
      call_id: "",
      status: "failed",
      error: { code: "INVALID_PHONE", message: phoneGuard.errors.join("; ") },
    };
  }

  // Compute identity. A new frozen envelope is a new atomic call, even when
  // the destination and conversational task are otherwise identical.
  const identity = computeCallIdentity(
    credentials.phone_number,
    params,
    options?.authority_digest
  );

  // Check registry for existing call
  const existing = registry.get(identity);
  if (existing) {
    // Completed or in-progress: return existing result (no duplicate dispatch)
    if (existing.status === "completed" && existing.result) {
      return existing.result;
    }
    if (existing.status === "in_progress") {
      // Cannot start a second call for the same identity while one is running.
      // Return the existing pending result or a waiting error.
      if (existing.result) return existing.result;
      return {
        call_id: existing.call_id ?? "",
        status: "in_progress",
        error: { code: "CALL_IN_PROGRESS", message: "call already in progress for this identity" },
      };
    }
    // If CALL-E assigned an ID, the external effect may have happened even if
    // local observation failed. Never create a second call; resume observation
    // through a future provider-specific get/poll operation instead.
    if (existing.call_id) {
      return existing.result ?? {
        call_id: existing.call_id,
        status: existing.status,
        error: {
          code: "OBSERVATION_REQUIRED",
          message: "existing CALL-E call identity is present; do not dispatch again",
        },
      };
    }
    // A failed attempt with no provider call ID is safe to retry.
  }

  // Create record
  const now = new Date().toISOString();
  const record: CallRecord = {
    identity,
    params,
    status: "pending",
    from: credentials.phone_number,
    authority_digest: options?.authority_digest,
    created_at: now,
    updated_at: now,
  };

  // Insert or replace (for retry after failure)
  if (existing) {
    registry.update(identity, { ...record, call_id: existing.call_id });
  } else {
    registry.insert(record);
  }

  // Mark in_progress
  registry.update(identity, { status: "in_progress" });

  // Dispatch via client
  const timeout = options?.timeout_ms ?? params.timeout_ms ?? 30_000;
  let result: CallResult;
  try {
    result = await client.createAndWait(credentials, params, { timeout_ms: timeout });
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    result = {
      call_id: "",
      status: "failed",
      error: { code: "DISPATCH_ERROR", message: msg },
    };
  }

  // Persist result
  registry.update(identity, {
    status: result.status,
    call_id: result.call_id || undefined,
    result,
  });

  return result;
}
