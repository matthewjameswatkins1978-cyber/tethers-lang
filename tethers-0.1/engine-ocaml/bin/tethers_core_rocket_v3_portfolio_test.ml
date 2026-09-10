module Core = Tethers_core
module Format = Tethers_core_canonical_v2_format
module V2 = Tethers_core_canonical_v2
module Reference = Tethers_core_canonical_v2_reference
module Path = Tethers_core_rocket_v3_success_path
module Portfolio = Tethers_core_rocket_v3_portfolio

let tests_run = ref 0
let tests_passed = ref 0

let check name condition =
  incr tests_run;
  if condition then incr tests_passed
  else begin
    Printf.eprintf "FAIL: %s\n%!" name;
    exit 1
  end

let oid value = Core.origin_id_of_string value
let pid value = Core.program_id_of_string value
let cid value = Core.capability_id_of_string value
let digest value = Core.capability_contract_digest_of_string value
let version value = Core.core_version_of_string value

let empty_program () = {
  Core.program_id = pid "portfolio-program";
  core_version = version "0.1.0";
  input_facts = [];
  entry_guards = [];
  entry_origin = None;
  success_continuations = [];
  origin_sites = [];
  branches = [];
  roles = [];
  item_templates = [];
  capability_contracts = [];
}

let action origin_id = Core.Action_origin {
  action_origin_id = origin_id;
  capability_id = cid "portfolio-capability";
  contract_digest = digest "portfolio-digest";
  inputs = [];
  declared_facts = [];
  execution_constraints = [];
}

let path_program_with_prefix prefix size =
  let origins = List.init size (fun i -> oid (prefix ^ "-origin-" ^ string_of_int i)) in
  let sites = List.map action origins in
  let continuations = List.mapi (fun i from_origin ->
    { Core.from_origin;
      target = if i + 1 = size then Core.Program_complete
        else Core.Origin_target (List.nth origins (i + 1)) }) origins
  in
  { (empty_program ()) with
    program_id = pid (prefix ^ "-path-" ^ string_of_int size);
    entry_origin = Some (List.hd origins);
    success_continuations = continuations;
    origin_sites = sites;
    capability_contracts = [{
      Core.capability_id = cid "portfolio-capability";
      contract_digest = digest "portfolio-digest";
      schema_description = "portfolio test contract";
    }];
  }

let path_program size = path_program_with_prefix "portfolio" size

let payload_of_v2 program =
  match V2.canonicalize program with
  | Ok result -> V2.canonical_payload result
  | Error _ -> failwith "V2 canonicalizer rejected a valid test fixture"

let payload_of_reference program =
  match Reference.slow_oracle program with
  | Ok result -> result.payload
  | Error _ -> failwith "reference rejected a valid test fixture"

let test_empty_matches_reference () =
  let program = empty_program () in
  match Portfolio.canonicalise program with
  | Error _ -> check "empty portfolio succeeds" false
  | Ok result ->
      check "empty uses exact V2 fallback"
        (Portfolio.backend_name result.stats.backend = "r3_2_refined_exact_leaf" ||
         Portfolio.backend_name result.stats.backend = "frozen_v2_exact_search");
      check "empty payload matches reference"
        (result.payload = payload_of_reference program);
      check "empty digest derives from payload"
        (result.digest = Format.digest_string_v2
           (Format.sha256_hex result.preimage))

let test_path_matches_reference () =
  let program = path_program 5 in
  match Portfolio.canonicalise program with
  | Error _ -> check "path portfolio succeeds" false
  | Ok result ->
      check "path selects B2" (result.stats.backend = Portfolio.B2_success_path);
      check "path payload matches reference"
        (result.payload = payload_of_reference program);
      check "path payload matches V2 exact search"
        (result.payload = payload_of_v2 program)

let test_reference_mode_is_explicit () =
  let program = path_program 3 in
  match Portfolio.canonicalise ~reference:true program with
  | Error _ -> check "explicit reference succeeds" false
  | Ok result ->
      check "explicit reference selects reference backend"
        (result.stats.backend = Portfolio.Exhaustive_reference);
      check "explicit reference records candidates"
        (Option.is_some result.stats.reference_candidates);
      check "explicit reference matches portfolio"
        (result.payload = payload_of_reference program)

let test_budget_falls_back_without_changing_identity () =
  let program = empty_program () in
  match Portfolio.canonicalise ~max_candidates:0 program with
  | Error _ -> check "zero budget has exact fallback" false
  | Ok result ->
      check "zero budget selects reference fallback"
        (result.stats.backend = Portfolio.Exhaustive_reference);
      check "zero budget preserves payload"
        (result.payload = payload_of_reference program)

let test_bounded_generated_and_metamorphic_corpus () =
  let rec loop size =
    if size > 5 then ()
    else begin
      let program = path_program size in
      let renamed = path_program_with_prefix ("renamed-" ^ string_of_int size) size in
      let reordered = { program with
        origin_sites = List.rev program.origin_sites;
        success_continuations = List.rev program.success_continuations;
      } in
      List.iter (fun (label, candidate) ->
        match Portfolio.canonicalise candidate with
        | Error _ -> check (label ^ " portfolio succeeds") false
        | Ok result ->
            check (label ^ " matches reference")
              (result.payload = payload_of_reference candidate);
            check (label ^ " digest is payload-derived")
              (result.digest = Format.digest_string_v2
                 (Format.sha256_hex result.preimage)))
        [ ("generated-path-" ^ string_of_int size, program);
          ("renamed-path-" ^ string_of_int size, renamed);
          ("reordered-path-" ^ string_of_int size, reordered) ];
      loop (size + 1)
    end
  in
  loop 1

let () =
  test_empty_matches_reference ();
  test_path_matches_reference ();
  test_reference_mode_is_explicit ();
  test_budget_falls_back_without_changing_identity ();
  test_bounded_generated_and_metamorphic_corpus ();
  Printf.printf "Rocket V3 portfolio: %d/%d checks passed\n%!"
    !tests_passed !tests_run
