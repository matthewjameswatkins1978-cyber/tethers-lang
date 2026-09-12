// CallPermit — Tethers 0.7 machine interface types and authority-envelope model
// Faithful to tethers-0.1/SPEC.md and tethers-0.1/protocol/ structure.

// ---------------------------------------------------------------------------
// Tethers 0.7 request types (matches tethers-0.1/protocol/request.json)
// ---------------------------------------------------------------------------

export interface TetherSource {
  id: string;
  version: string;
  source: string;
}

export interface TetherEvent {
  id: string;
  name: string;
  data: Record<string, unknown>;
}

export interface CapabilitySchema {
  name: string;
  version: string;
  inputs: Record<string, string>;
  effects: string[];
  reversibility?: string;
}

export interface TetherRequest {
  protocol_version: string;
  language_version: string;
  evaluation_id: string;
  tether: TetherSource;
  event: TetherEvent;
  facts: Record<string, unknown>;
  capabilities: CapabilitySchema[];
}

// ---------------------------------------------------------------------------
// Tethers 0.7 response types (matches tethers-0.1/protocol/expected-response.json)
// ---------------------------------------------------------------------------

export interface TrailEntry {
  sequence: number;
  phase: string;
  kind: string;
  outcome: string;
  message: string;
}

export interface PlannedAction {
  action_id: string;
  idempotency_key: string;
  capability: string;
  capability_version: string;
  arguments: Record<string, unknown>;
  effects: string[];
}

export interface Plan {
  id: string;
  required_effects: string[];
  actions: PlannedAction[];
}

export type TetherStatus = "matched" | "not_matched" | "error";

export interface TetherResponse {
  protocol_version: string;
  evaluation_id: string;
  event_id: string;
  tether_id: string;
  tether_version: string;
  status: TetherStatus;
  plan: Plan | null;
  trail: TrailEntry[];
  error?: { code: string; message: string };
}

// ---------------------------------------------------------------------------
// Capability manifest (matches tethers-0.1/protocol/capability-manifests/*.json)
// ---------------------------------------------------------------------------

export interface ConfirmationPolicy {
  standing_permitted: boolean;
  per_call_required: boolean;
}

export interface PermissionScope {
  kind: string;
  allowed_prefixes?: string[];
}

export interface CapabilityManifest {
  manifest_format_version: string;
  capability_name: string;
  capability_version: number;
  title: string;
  description: string;
  input_schema: {
    type: "object";
    properties: Record<string, { type: string }>;
    required: string[];
    additionalProperties: false;
  };
  output_schema?: {
    type: "object";
    properties: Record<string, { type: string }>;
    required: string[];
    additionalProperties: false;
  };
  effects: string[];
  permission_scope: PermissionScope | null;
  reversibility: string;
  determinism: string;
  idempotency: {
    mechanism: string;
    argument_name: string;
    key_source: string;
  };
  confirmation_policy: ConfirmationPolicy;
  timeout_ms: number;
  provider: {
    identity: string;
    display_name: string;
    identity_source: string;
    description: string;
  };
  binding: {
    kind: string;
    server_name: string;
    tool_name: string;
    adapter: string | null;
  };
  digest: string;
}

// ---------------------------------------------------------------------------
// CallPermit authority-envelope model
// ---------------------------------------------------------------------------

/** The three possible authority outcomes. */
export type AuthorityDecision = "ALLOW" | "ASK" | "DENY";

/**
 * Immutable policy entry for a single capability.
 * Maps a Tethers capability to an authority decision.
 */
export interface CapabilityPolicy {
  /** Capability name (must match manifest capability_name). */
  capability_name: string;
  /** Capability version expected. */
  capability_version: number;
  /** Authority decision when this capability is planned. */
  decision: AuthorityDecision;
  /** Optional: allowed effects subset. If omitted, all effects are permitted
   *  for this capability (subject to the decision). */
  allowed_effects?: string[];
  /** Optional: scope constraints. */
  scope?: PermissionScope;
}

/**
 * Authority envelope — the immutable, identity-bearing policy container.
 * The digest provides a stable identity for the envelope contents.
 */
export interface AuthorityEnvelope {
  /** Schema version for forward compatibility. */
  envelope_version: string;
  /** Human-readable policy identifier. */
  policy_id: string;
  /** Immutable list of capability policies. Order is not significant. */
  capabilities: CapabilityPolicy[];
  /** Global default when a capability is not listed. */
  default_decision: AuthorityDecision;
  /** SHA-256 digest of the canonical envelope bytes (excludes digest itself). */
  digest: string;
}

/**
 * Result of an authority-envelope evaluation.
 */
export interface AuthorityResult {
  /** The final authority decision. */
  decision: AuthorityDecision;
  /** Human-readable explanation. */
  reason: string;
  /** Per-action breakdown when a plan exists. */
  action_results?: Array<{
    action_id: string;
    capability: string;
    decision: AuthorityDecision;
    reason: string;
  }>;
  /** The evaluation trail from the Tethers planner (if available). */
  trail?: TrailEntry[];
}

// ---------------------------------------------------------------------------
// CALL-E runtime adapter types
// ---------------------------------------------------------------------------

/** Credential required to authenticate with the CALL-E API. */
export interface CalleCredentials {
  /** API key for authentication. */
  api_key: string;
  /** Phone number that initiates the call (E.164 format recommended). */
  phone_number: string;
}

/** Parameters for initiating a single phone call via CALL-E. */
export interface CallParams {
  /** Destination phone number (E.164 format recommended). */
  to: string;
  /** Capability name this call fulfills. */
  capability: string;
  /** Optional capability version. */
  capability_version?: string;
  /** Optional call timeout in milliseconds. */
  timeout_ms?: number;
  /** Optional prompt or instruction for the call. */
  prompt?: string;
  /** Arbitrary call metadata. */
  metadata?: Record<string, unknown>;
}

/** Possible states for a tracked call. */
export type CallStatus = "pending" | "in_progress" | "completed" | "failed" | "timed_out";

/** Structured result returned by a completed call. */
export interface CallResult {
  /** Unique identifier for this call. */
  call_id: string;
  /** Final call status. */
  status: CallStatus;
  /** Transcript or text result of the call, if available. */
  transcript?: string;
  /** Duration in milliseconds. */
  duration_ms?: number;
  /** Timestamp when the call completed (ISO 8601). */
  completed_at?: string;
  /** Error information if the call failed. */
  error?: { code: string; message: string };
}

/** Persisted record of a single call lifecycle. */
export interface CallRecord {
  /** Deterministic identity derived from call parameters. */
  identity: string;
  /** Unique ID assigned by the CALL-E API (set after creation). */
  call_id?: string;
  /** Original call parameters. */
  params: CallParams;
  /** Current call status. */
  status: CallStatus;
  /** Phone number that initiated the call. */
  from: string;
  /** Timestamp when the record was created (ISO 8601). */
  created_at: string;
  /** Timestamp of the last state update (ISO 8601). */
  updated_at: string;
  /** Final result, if the call completed or failed. */
  result?: CallResult;
}

/** Transport-agnostic interface for CALL-E API operations. */
export interface CalleClient {
  /** Create a call and wait for it to complete. Returns structured result. */
  createAndWait(
    credentials: CalleCredentials,
    params: CallParams,
    options?: { timeout_ms?: number }
  ): Promise<CallResult>;
}

/** Persistence layer for call records. */
export interface CallRegistry {
  /** Look up a call by its deterministic identity. */
  get(identity: string): CallRecord | undefined;
  /** Insert a new call record. Throws if identity already exists. */
  insert(record: CallRecord): void;
  /** Update an existing call record. Throws if identity not found. */
  update(identity: string, patch: Partial<CallRecord>): void;
  /** List all records. */
  all(): CallRecord[];
}
