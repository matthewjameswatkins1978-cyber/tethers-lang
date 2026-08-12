(** Core wire adapter: bridges the CORE-8B request adapter to the existing
    Tethers_outcome response envelope.

    This module takes ONE extended request JSON, delegates to
    [Tethers_core_request_adapter.evaluate_request], and produces a JSON
    response suitable for the existing Rust [PlannerResponseWire] classifier.

    Do NOT reimplement parsing, lowering, canonicalisation, reception,
    guard evaluation, or planning.  Delegate to CORE-8B. *)

val evaluate_request_json :
  Yojson.Safe.t -> Yojson.Safe.t
(** Evaluate a complete Core request and return the JSON response envelope.

    Success ([Ok]) produces the existing historical response envelope with
    status "matched" or "not_matched" as appropriate.

    Request errors produce a stable Core error envelope with status "error".

    The function never raises.  All outcomes are represented in the returned
    JSON. *)
