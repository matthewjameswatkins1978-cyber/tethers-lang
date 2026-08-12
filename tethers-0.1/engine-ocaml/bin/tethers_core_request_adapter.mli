(** Core Evaluation Request Boundary.

    CORE-8B consumes an extended Tethers 0.1 request JSON and calls the
    accepted [Tethers_core_evaluation_adapter.evaluate], establishing the
    exact wire contract required for later production wiring.

    The module owns request parsing, runtime capability resolution, semantic
    environment assembly, and the CORE-8A adapter call.  The test body must
    not manually assemble CORE-8A environment/input.

    This is NOT the legacy [Tethers_evaluator] and must not replace it. *)

type request_context = {
  protocol_version : string;
  language_version : string;
  evaluation_id : string;
  event_id : string;
  tether_id : string;
  tether_version : string;
}
(** Correlation information preserved from the request envelope. *)

type evaluated_request = {
  context : request_context;
  evaluation : Tethers_core_plan.canonical_evaluation;
}
(** The evaluation result paired with its request context. *)

type request_error =
  | Invalid_request of string * string
  | Missing_core_environment
  | Invalid_core_environment of string
  | Missing_runtime_capability_binding of string
  | Ambiguous_runtime_capability_binding of string
  | Invalid_scalar_type of string
  | Adapter_error of Tethers_core_evaluation_adapter.adapter_error
(** Typed request error preserving layer ownership. *)

val evaluate_request :
  Yojson.Safe.t ->
  (evaluated_request, request_error) result
(** Parse an extended Tethers 0.1 request, resolve runtime capability
    bindings through [core_environment], and call the CORE-8A adapter.

    The caller supplies ONE JSON request.  The module owns request parsing,
    runtime capability parsing, semantic environment assembly, and the
    CORE-8A adapter call.  The test body must not manually assemble
    CORE-8A environment/input.

    Protocol version must be ["0.1"].  Language version must be ["0.1"].
    The [core_environment] field is required. *)
