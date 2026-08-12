(* Core wire adapter: bridges the CORE-8B request adapter to the existing
   Tethers_outcome response envelope.

   Takes ONE extended request JSON, delegates to
   Tethers_core_request_adapter.evaluate_request, and produces a JSON
   response suitable for the Rust PlannerResponseWire classifier. *)

open Tethers_core_request_adapter

(* ================================================================== *)
(*  Error code mapping                                                 *)
(*                                                                    *)
(*  Deterministic stable codes for Core request errors.               *)
(* ================================================================== *)

let error_code_of_request_error = function
  | Invalid_request _ -> "invalid_request"
  | Missing_core_environment -> "missing_core_environment"
  | Invalid_core_environment _ -> "invalid_core_environment"
  | Missing_runtime_capability_binding _ -> "missing_runtime_capability_binding"
  | Ambiguous_runtime_capability_binding _ ->
      "ambiguous_runtime_capability_binding"
  | Invalid_scalar_type _ -> "invalid_scalar_type"
  | Adapter_error _ -> "adapter_error"

let error_message_of_request_error = function
  | Invalid_request (field, msg) -> field ^ ": " ^ msg
  | Missing_core_environment -> "core_environment is required"
  | Invalid_core_environment msg -> "invalid core_environment: " ^ msg
  | Missing_runtime_capability_binding name ->
      "no runtime capability matches runtime_name: " ^ name
  | Ambiguous_runtime_capability_binding name ->
      "multiple runtime capabilities match runtime_name: " ^ name
  | Invalid_scalar_type s -> "invalid scalar_type: " ^ s
  | Adapter_error adapter_err ->
      let detail =
        match adapter_err with
        | Tethers_core_evaluation_adapter.Parse_error (code, msg) ->
            code ^ ": " ^ msg
        | Tethers_core_evaluation_adapter.Lowering_error err ->
            (match err with
             | Tethers_core_lowerer.Unsupported_construct s ->
                 "unsupported_construct: " ^ s
             | Tethers_core_lowerer.Unknown_capability s ->
                 "unknown_capability: " ^ s
             | Tethers_core_lowerer.Duplicate_capability s ->
                 "duplicate_capability: " ^ s
             | Tethers_core_lowerer.Unknown_fact s ->
                 "unknown_fact: " ^ s
             | Tethers_core_lowerer.Duplicate_fact s ->
                 "duplicate_fact: " ^ s
             | Tethers_core_lowerer.Conflicting_capability_contract _ ->
                 "conflicting_capability_contract"
             | Tethers_core_lowerer.Missing_anchor_reference s ->
                 "missing_anchor_reference: " ^ s)
        | Tethers_core_evaluation_adapter.Canonicalization_error _ ->
            "canonicalization_error"
        | Tethers_core_evaluation_adapter.Planning_error _ ->
            "planning_error"
        | Tethers_core_evaluation_adapter.Unknown_runtime_fact_name n ->
            "unknown_runtime_fact_name: " ^ n
        | Tethers_core_evaluation_adapter.Ambiguous_runtime_fact_name n ->
            "ambiguous_runtime_fact_name: " ^ n
        | Tethers_core_evaluation_adapter.Duplicate_runtime_fact_name n ->
            "duplicate_runtime_fact_name: " ^ n
      in
      "adapter_error: " ^ detail

(* ================================================================== *)
(*  Response construction                                              *)
(* ================================================================== *)

(* Minimal error envelope: protocol_version + status + error. *)
let make_error_envelope code message =
  `Assoc
    [
      ("protocol_version", `String "0.1");
      ("status", `String "error");
      ("error", `Assoc [ ("code", `String code); ("message", `String message) ]);
    ]

(* Build the response envelope from a successful evaluated_request.
   Matches the existing Tethers_outcome.json_of_response shape. *)
let make_success_envelope (result : evaluated_request) =
  let ctx = result.context in
  match result.evaluation with
  | Tethers_core_plan.Matched canonical_plan ->
      let base =
        Tethers_outcome.json_of_response
          (Tethers_outcome.Contextual
             {
               context =
                 {
                   evaluation_id = ctx.evaluation_id;
                   event_id = ctx.event_id;
                   tether_id = ctx.tether_id;
                   tether_version = ctx.tether_version;
                 };
               payload = Tethers_outcome.Matched canonical_plan.runtime_plan;
               trail = [];
             })
      in
      (* Add program_digest as a sibling of plan in the envelope. *)
      (match base with
       | `Assoc fields ->
           `Assoc
             (fields
             @ [
                 ( "program_digest",
                   `String
                     (Tethers_core_canonical.string_of_program_digest
                        canonical_plan.program_digest) );
               ])
       | other -> other)
  | Tethers_core_plan.Not_matched ->
      Tethers_outcome.json_of_response
        (Tethers_outcome.Contextual
           {
             context =
               {
                 evaluation_id = ctx.evaluation_id;
                 event_id = ctx.event_id;
                 tether_id = ctx.tether_id;
                 tether_version = ctx.tether_version;
               };
             payload = Tethers_outcome.Not_matched;
             trail = [];
           })

(* ================================================================== *)
(*  Public API                                                         *)
(* ================================================================== *)

let evaluate_request_json request =
  match evaluate_request request with
  | Ok result -> make_success_envelope result
  | Error err ->
      make_error_envelope
        (error_code_of_request_error err)
        (error_message_of_request_error err)
