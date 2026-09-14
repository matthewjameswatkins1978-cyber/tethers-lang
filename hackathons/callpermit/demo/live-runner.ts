import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { buildAppointmentEnvelope } from "../src/authority.js";
import { computeCallIdentity, dispatchCall, deserialiseRegistry, serialiseRegistry, createCallRegistry } from "../src/calle.js";
import { buildMatchedTethersResponse, compileAppointmentCall, runAppointmentCall } from "../src/callpermit.js";
import { CalleSdkClient } from "../src/calle-sdk.js";
import {
  CALLE_OFFICIAL_TEST_HOTLINE,
  LIVE_PROOF_PROMPT,
  LIVE_PROOF_RESULT_SCHEMA,
  requireOfficialHotlineProfile,
} from "../src/live-proof.js";
import type { AppointmentTerms, CalleCredentials, CallRegistry } from "../src/types.js";

const profile = requireOfficialHotlineProfile(process.env.CALLPERMIT_LIVE_PROFILE?.trim());
const configuredDestination = process.env.CALLE_TEST_PHONE?.trim();
if (configuredDestination && configuredDestination.replace(/[^0-9+]/g, "") !== profile.phone) {
  throw new Error("CALLE_TEST_PHONE does not match the explicit official-hotline profile; refuse to dial");
}
const destination = profile.phone;
const apiKey = process.env.CALLE_API_KEY?.trim();
const sourceIdentity = process.env.CALLE_SOURCE_PHONE?.trim() || destination;
const evidenceRoot = resolve(process.env.CALLPERMIT_EVIDENCE_DIR || "work/live-call");
const registryPath = resolve(evidenceRoot, "registry.json");
const rawEvidencePath = resolve(evidenceRoot, "raw-evidence.json");
const publicEvidencePath = resolve(evidenceRoot, "public-evidence.json");
const resultSchemaMode = process.env.CALLE_RESULT_SCHEMA_MODE === "recipient" ? "recipient" : "none";

if (!apiKey) throw new Error("CALLE_API_KEY is required; refuse to run without explicit credentials");
if (!sourceIdentity) throw new Error("CALLE_SOURCE_PHONE or CALLE_TEST_PHONE is required for deterministic identity");

const envelope = buildAppointmentEnvelope("callpermit.live.demo.v2");
const terms: AppointmentTerms = {
  offered_date: "2026-09-15",
  offered_time: "15:30",
  offered_price_minor: 4500,
  currency: "GBP",
  service: "standard service",
  extra_requested: null,
};
const askTerms: AppointmentTerms = { ...terms, offered_price_minor: 9500 };
const denyTerms: AppointmentTerms = {
  ...terms,
  commitment_kind: "subscription",
  subscription: true,
};

async function loadRegistry(): Promise<CallRegistry> {
  try {
    const raw = await readFile(registryPath, "utf8");
    return deserialiseRegistry(JSON.parse(raw));
  } catch (error) {
    const code = error && typeof error === "object" && "code" in error ? error.code : undefined;
    if (code === "ENOENT") return createCallRegistry();
    throw error;
  }
}

function redactPhone(value: string | undefined): string | undefined {
  return value ? "[REDACTED_PHONE]" : value;
}

function redactTranscript(value: string | undefined): string | undefined {
  if (!value) return value;
  return value.replace(/\+?\d[\d\s().-]{6,}\d/g, "[REDACTED_PHONE]");
}

function publicRun(run: Awaited<ReturnType<typeof runAppointmentCall>> | null) {
  if (!run) return null;
  return {
    authority: run.authority,
    call_params: {
      ...run.call_params,
      to: redactPhone(run.call_params.to),
    },
    call: run.call_result
      ? {
          ...run.call_result,
          transcript: redactTranscript(run.call_result.transcript),
          call_id: run.call_result.call_id ? "[REAL_CALL_ID_REDACTED]" : run.call_result.call_id,
        }
      : null,
    reconciliation: run.reconciliation
      ? { ...run.reconciliation, call_id: "[REAL_CALL_ID_REDACTED]" }
      : null,
  };
}

await mkdir(dirname(registryPath), { recursive: true });
const credentials: CalleCredentials = { api_key: apiKey, phone_number: sourceIdentity };
const client = new CalleSdkClient({
  api_key: apiKey,
  result_schema_mode: resultSchemaMode,
  result_schema: LIVE_PROOF_RESULT_SCHEMA,
});

const ask = await runAppointmentCall({
  envelope,
  terms: askTerms,
  tethers_response: buildMatchedTethersResponse(envelope, askTerms, "callpermit-live-ask"),
  credentials,
  registry: createCallRegistry(),
  destination,
  recipient_region: profile.region,
  recipient_locale: profile.locale,
  evaluation_id: "callpermit-live-ask",
  client,
});
const deny = await runAppointmentCall({
  envelope,
  terms: denyTerms,
  tethers_response: buildMatchedTethersResponse(envelope, denyTerms, "callpermit-live-deny"),
  credentials,
  registry: createCallRegistry(),
  destination,
  recipient_region: profile.region,
  recipient_locale: profile.locale,
  evaluation_id: "callpermit-live-deny",
  client,
});

const previewParams = compileAppointmentCall(envelope, terms, destination, {
  region: profile.region,
  locale: profile.locale,
}, { prompt: LIVE_PROOF_PROMPT, timeout_ms: 180_000 });
const previewIdentity = computeCallIdentity(sourceIdentity, previewParams, envelope.digest);
console.log(JSON.stringify({
  pre_dispatch_preview: {
    decision: "ALLOW",
    target: profile.target_type,
    destination: redactPhone(destination),
    region: profile.region,
    locale: profile.locale,
    authority_digest: envelope.digest,
    idempotency_fingerprint: previewIdentity.slice(-8),
    task: "harmless CALL-E integration test; no appointment, purchase, subscription, payment, deposit, or commitment",
    notice: "A real CALL-E call is about to occur exactly once; replay is skipped unless a provider call ID is returned.",
  },
}, null, 2));

const firstRegistry = await loadRegistry();
const first = await runAppointmentCall({
  envelope,
  terms,
  tethers_response: buildMatchedTethersResponse(envelope, terms, "callpermit-live-allow"),
  credentials,
  registry: firstRegistry,
  destination,
  recipient_region: profile.region,
  recipient_locale: profile.locale,
  prompt_override: LIVE_PROOF_PROMPT,
  timeout_ms: 180_000,
  evaluation_id: "callpermit-live-allow",
  client,
});
await writeFile(registryPath, JSON.stringify(serialiseRegistry(firstRegistry), null, 2), "utf8");

const restartedRegistry = await loadRegistry();
const repeat = first.call_result?.call_id
  ? await runAppointmentCall({
      envelope,
      terms,
      tethers_response: buildMatchedTethersResponse(envelope, terms, "callpermit-live-allow"),
      credentials,
      registry: restartedRegistry,
      destination,
      recipient_region: profile.region,
      recipient_locale: profile.locale,
      prompt_override: LIVE_PROOF_PROMPT,
      timeout_ms: 180_000,
      evaluation_id: "callpermit-live-allow",
      client,
    })
  : null;
await writeFile(registryPath, JSON.stringify(serialiseRegistry(restartedRegistry), null, 2), "utf8");

const rawEvidence = {
  mode: "live-sdk",
  profile: profile.profile,
  target_type: profile.target_type,
  region: profile.region,
  locale: profile.locale,
  permission_proof: {
    ask: { decision: ask.authority.decision, dispatch_count: ask.call_result ? 1 : 0 },
    deny: { decision: deny.authority.decision, dispatch_count: deny.call_result ? 1 : 0 },
    allow: { decision: first.authority.decision, dispatch_count: first.call_result ? 1 : 0 },
  },
  result_schema_mode: resultSchemaMode,
  destination,
  source_identity: sourceIdentity,
  authority_digest: envelope.digest,
  ask,
  deny,
  first,
  repeat,
  idempotency: first.call_result?.call_id
    ? {
        same_call_id: Boolean(first.call_result.call_id === repeat?.call_result?.call_id),
        repeat_status: repeat?.call_result?.status,
        repeat_dispatch_count: 0,
      }
    : {
        same_call_id: false,
        repeat_status: null,
        repeat_dispatch_count: 0,
        safe_replay_skipped: true,
        reason: "first attempt returned no provider call ID; no replay was attempted",
      },
};
await writeFile(rawEvidencePath, JSON.stringify(rawEvidence, null, 2), "utf8");
await writeFile(
  publicEvidencePath,
  JSON.stringify(
    {
      ...rawEvidence,
      destination: redactPhone(destination),
      source_identity: redactPhone(sourceIdentity),
      ask: publicRun(ask),
      deny: publicRun(deny),
      first: publicRun(first),
      repeat: publicRun(repeat),
      idempotency: rawEvidence.idempotency,
    },
    null,
    2,
  ),
  "utf8",
);

console.log(JSON.stringify({
  mode: "live-sdk",
  evidence: { raw: rawEvidencePath, public: publicEvidencePath },
  ask: { decision: ask.authority.decision, call_dispatched: Boolean(ask.call_result) },
  deny: { decision: deny.authority.decision, call_dispatched: Boolean(deny.call_result) },
  first: {
    decision: first.authority.decision,
    status: first.call_result?.status,
    call_id_present: Boolean(first.call_result?.call_id),
    reconciliation: first.reconciliation?.status,
  },
  repeat: {
    status: repeat?.call_result?.status,
    same_call_id: rawEvidence.idempotency.same_call_id,
  },
}, null, 2));
