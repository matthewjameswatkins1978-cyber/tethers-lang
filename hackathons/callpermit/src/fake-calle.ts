// Explicit offline CALL-E substitute for CallPermit development and tests.
// This module never opens a socket, reads credentials, or places a call.

import type { CalleClient, CalleCredentials, CallParams, CallResult } from "./types.js";

export interface FakeCallConfig {
  fixedResult?: CallResult;
  failWith?: { code: string; message: string };
  delay_ms?: number;
  throwOnDispatch?: boolean;
}

export interface FakeCalleClient extends CalleClient {
  readonly callCount: number;
}

/** Deterministic CALL-E-shaped transport for offline acceptance only. */
export function createFakeCalleClient(config: FakeCallConfig = {}): FakeCalleClient {
  let callCount = 0;
  return {
    get callCount() {
      return callCount;
    },
    async createAndWait(
      _credentials: CalleCredentials,
      params: CallParams,
      _options?: { timeout_ms?: number }
    ): Promise<CallResult> {
      callCount += 1;
      if (config.delay_ms && config.delay_ms > 0) {
        await new Promise((resolve) => setTimeout(resolve, config.delay_ms));
      }
      if (config.throwOnDispatch) throw new Error("simulated dispatch failure");
      if (config.failWith) {
        return { call_id: "", status: "failed", outcome: "FAILED", error: config.failWith };
      }
      if (config.fixedResult) return structuredClone(config.fixedResult);
      return {
        call_id: `fake-call-${callCount}`,
        status: "completed",
        outcome: "NO_MATCH",
        task_completed: false,
        commitment_made: false,
        transcript: `offline fake CALL-E transcript for ${params.capability}`,
        duration_ms: 1200,
        completed_at: "2026-09-12T00:00:00.000Z",
      };
    },
  };
}
