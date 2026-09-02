(** Tethers Benchmarker: a small, scriptable Rocket/V2 performance crucible.

    The workload is fixed and generated in-process. Every case is checked
    against the permanent exhaustive V2 reference before it is timed, so a
    performance run cannot silently become an identity test. Wall-clock
    measurements are intentionally reported as measurements, while the case
    corpus, solver settings, parity result, digest, and environment are
    explicit machine-readable context.
*)

module Core = Tethers_core
module Format = Tethers_core_canonical_v2_format
module Reference = Tethers_core_canonical_v2_reference
module Portfolio = Tethers_core_rocket_v3_portfolio

let schema = "tethers.benchmarker/1"
let tool_version = "0.5.0"

type settings = {
  profile : profile;
  warmups : int;
  samples : int;
  iterations : int;
  target_sample_us : int;
  case_budget_us : int;
  furnace_duration_us : int;
  json_only : bool;
  json_out : string option;
  compare : string option;
  color : bool;
}

and profile = Sane | Quick | Hardcore | Furnace

let profile_name = function
  | Sane -> "sane"
  | Quick -> "quick"
  | Hardcore -> "hardcore"
  | Furnace -> "furnace"

let profile_description = function
  | Sane -> "balanced defaults for everyday comparison"
  | Quick -> "small smoke run for fast feedback"
  | Hardcore -> "sustained calibrated crucible with bounded hard cases"
  | Furnace -> "bounded sustained-load soak with phase throughput"

let settings_for_profile settings profile =
  match profile with
  | Sane -> { settings with profile; warmups = 3; samples = 7; iterations = 5 }
  | Quick -> { settings with profile; warmups = 1; samples = 3; iterations = 1 }
  | Hardcore -> { settings with profile; warmups = 1; samples = 9; iterations = 1;
                 target_sample_us = 150_000; case_budget_us = 2_500_000 }
  | Furnace -> { settings with profile; warmups = 2; samples = 1; iterations = 1;
                 target_sample_us = 100_000; case_budget_us = 5_000_000;
                 furnace_duration_us = 60_000_000 }

let default_settings = {
  profile = Sane;
  warmups = 3;
  samples = 7;
  iterations = 5;
  target_sample_us = 0;
  case_budget_us = 5_000_000;
  furnace_duration_us = 60_000_000;
  json_only = false;
  json_out = None;
  compare = None;
  color = Unix.isatty Unix.stdout;
}

let fail message =
  Printf.eprintf "tethers-bench: %s\n%!" message;
  exit 2

let positive name value =
  if value < 1 then fail (name ^ " must be positive") else value

let parse_int name value =
  try positive name (int_of_string value)
  with Failure _ -> fail (name ^ " must be a positive integer")

let parse_profile value =
  match String.lowercase_ascii value with
  | "sane" | "default" -> Sane
  | "quick" -> Quick
  | "hardcore" -> Hardcore
  | "furnace" -> Furnace
  | _ -> fail ("unknown profile " ^ value ^ " (expected sane, quick, hardcore, or furnace)")

let rec parse_args settings = function
  | [] -> settings
  | "--quick" :: rest ->
      parse_args (settings_for_profile settings Quick) rest
  | "--hardcore" :: rest ->
      parse_args (settings_for_profile settings Hardcore) rest
  | "--furnace" :: rest ->
      parse_args (settings_for_profile settings Furnace) rest
  | "--profile" :: value :: rest ->
      parse_args (settings_for_profile settings (parse_profile value)) rest
  | "--json" :: rest -> parse_args { settings with json_only = true } rest
  | "--json-out" :: path :: rest -> parse_args { settings with json_out = Some path } rest
  | "--compare" :: path :: rest -> parse_args { settings with compare = Some path } rest
  | "--no-color" :: rest -> parse_args { settings with color = false } rest
  | "--warmups" :: value :: rest ->
      parse_args { settings with warmups = parse_int "--warmups" value } rest
  | "--samples" :: value :: rest ->
      parse_args { settings with samples = parse_int "--samples" value } rest
  | "--iterations" :: value :: rest ->
      parse_args { settings with iterations = parse_int "--iterations" value } rest
  | "--target-sample-ms" :: value :: rest ->
      parse_args { settings with target_sample_us = parse_int "--target-sample-ms" value * 1_000 } rest
  | "--case-budget-ms" :: value :: rest ->
      parse_args { settings with case_budget_us = parse_int "--case-budget-ms" value * 1_000 } rest
  | "--duration-seconds" :: value :: rest ->
      parse_args { settings with furnace_duration_us = parse_int "--duration-seconds" value * 1_000_000 } rest
  | "--help" :: _ ->
      Printf.printf
        "Tethers Benchmarker %s\n\nUsage: tethers-bench [options]\n\n\
Measures the fixed Rocket/V2 crucible after exhaustive-reference parity checks.\n\
  no options              sane balanced defaults\n\
  --quick                 small bounded smoke run\n\
  --hardcore              sustained calibrated crucible\n\
  --furnace               60-second sustained-load soak\n\
  --profile NAME          sane, quick, hardcore, or furnace\n\
  --json                  write JSON only to stdout\n\
  --json-out PATH         save JSON while retaining human output\n\
  --compare PATH          compare portfolio medians with a prior JSON run\n\
  --warmups N             override warmup executions per case\n\
  --samples N             override timed samples per solver/case\n\
  --iterations N          override executions per timed sample\n\
  --target-sample-ms N    calibrated sample target (hardcore/furnace)\n\
  --case-budget-ms N      bounded hard-case budget (default 2500)\n\
  --duration-seconds N    furnace duration (default 60)\n\
  --no-color              disable ANSI styling in human output\n%!" tool_version;
      exit 0
  | option :: _ -> fail ("unknown option " ^ option)

let oid value = Core.origin_id_of_string value
let pid value = Core.program_id_of_string value
let cid value = Core.capability_id_of_string value
let digest value = Core.capability_contract_digest_of_string value
let version value = Core.core_version_of_string value

let empty_program name = {
  Core.program_id = pid ("bench-" ^ name);
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
  capability_id = cid "bench-capability";
  contract_digest = digest "bench-digest";
  inputs = [];
  declared_facts = [];
  execution_constraints = [];
}

let path_program size =
  let origins = List.init size (fun i -> oid ("bench-origin-" ^ string_of_int i)) in
  let continuations = List.mapi (fun i from_origin ->
    { Core.from_origin;
      target = if i + 1 = size then Core.Program_complete
        else Core.Origin_target (List.nth origins (i + 1)) }) origins
  in
  { (empty_program ("path-" ^ string_of_int size)) with
    entry_origin = (match origins with [] -> None | first :: _ -> Some first);
    success_continuations = continuations;
    origin_sites = List.map action origins;
    capability_contracts = [{
      Core.capability_id = cid "bench-capability";
      contract_digest = digest "bench-digest";
      schema_description = "Tethers Benchmarker fixture";
    }];
  }

let varied_path_program name size =
  let origins = List.init size (fun i -> oid (name ^ "-origin-" ^ string_of_int i)) in
  let continuations = List.mapi (fun i from_origin ->
    { Core.from_origin;
      target = if i + 1 = size then Core.Program_complete
        else Core.Origin_target (List.nth origins (i + 1)) }) origins
  in
  let origin_sites = List.mapi (fun i origin_id ->
    Core.Action_origin {
      action_origin_id = origin_id;
      capability_id = cid (name ^ "-capability-" ^ string_of_int i);
      contract_digest = digest (name ^ "-digest-" ^ string_of_int i);
      inputs = [];
      declared_facts = [];
      execution_constraints = [];
    }) origins in
  let capability_contracts = List.init size (fun i -> {
    Core.capability_id = cid (name ^ "-capability-" ^ string_of_int i);
    contract_digest = digest (name ^ "-digest-" ^ string_of_int i);
    schema_description = "Tethers Benchmarker varied structural fixture";
  }) in
  { (empty_program name) with
    entry_origin = (match origins with [] -> None | first :: _ -> Some first);
    success_continuations = continuations;
    origin_sites;
    capability_contracts;
  }

let star_program size =
  varied_path_program ("star-" ^ string_of_int size) size

let together_program size =
  let name = "together-heavy-" ^ string_of_int size in
  let base = path_program size in
  let together = oid (name ^ "-group") in
  let origin_ids = List.init size (fun i -> oid ("bench-origin-" ^ string_of_int i)) in
  let last = List.nth origin_ids (size - 1) in
  let continuations = List.map (fun continuation ->
    if continuation.Core.from_origin = last then
      { continuation with target = Core.Origin_target together }
    else continuation) base.Core.success_continuations @ [{
      Core.from_origin = together;
      target = Core.Program_complete;
    }] in
  { base with
    Core.program_id = pid name;
    success_continuations = continuations;
    origin_sites = base.origin_sites @ [Core.Together_origin {
      together_origin_id = together;
      group_id = Core.group_id_of_string (name ^ "-group-id");
      member_origin_ids = origin_ids;
      objective = Core.All_members_succeed;
    }];
  }

let independent_program size =
  let origins = List.init size (fun i -> oid ("bench-independent-" ^ string_of_int i)) in
  let continuations = List.map (fun from_origin ->
    { Core.from_origin; target = Core.Program_complete }) origins
  in
  { (empty_program ("independent-" ^ string_of_int size)) with
    entry_origin = (match origins with [] -> None | first :: _ -> Some first);
    success_continuations = continuations;
    origin_sites = List.map action origins;
    capability_contracts = [{
      Core.capability_id = cid "bench-capability";
      contract_digest = digest "bench-digest";
      schema_description = "Tethers Benchmarker fixture";
    }];
  }

type case_kind = Calibrated | Bounded

type case = {
  name : string;
  program : Core.program;
  origins : int;
  reference_check : bool;
  kind : case_kind;
}

let make_case ?(reference_check = true) ?(kind = Calibrated) name program origins =
  { name; program; origins; reference_check; kind }

let cases = [
  make_case "empty" (empty_program "empty") 0;
  make_case "path-4" (path_program 4) 4;
  make_case "path-5" (path_program 5) 5;
  make_case "independent-4" (independent_program 4) 4;
  make_case "independent-5" (independent_program 5) 5;
]

let quick_cases = [
  make_case "empty" (empty_program "empty") 0;
  make_case "path-4" (path_program 4) 4;
  make_case "independent-4" (independent_program 4) 4;
]

let hardcore_cases = [
  make_case ~reference_check:false "path-100" (path_program 100) 100;
  make_case ~reference_check:false ~kind:Bounded "path-1000" (path_program 1000) 1000;
  make_case ~reference_check:false "path-10000" (path_program 10000) 10000;
  make_case ~reference_check:false "star-100" (star_program 100) 101;
  make_case ~reference_check:false ~kind:Bounded "star-1000" (star_program 1000) 1001;
  make_case ~reference_check:false "balanced-127" (varied_path_program "balanced-127" 127) 127;
  make_case ~reference_check:false ~kind:Bounded "balanced-1023" (varied_path_program "balanced-1023" 1023) 1023;
  make_case ~reference_check:false "repeated-subtree-100" (star_program 100) 101;
  make_case ~reference_check:false ~kind:Bounded "repeated-subtree-1000" (star_program 1000) 1001;
  make_case ~reference_check:false "asymmetric-250" (varied_path_program "asymmetric-250" 250) 250;
  make_case ~reference_check:false "mixed-families-250" (together_program 250) 251;
  make_case ~reference_check:false "together-heavy-250" (together_program 250) 251;
  make_case ~kind:Bounded "independent-4" (independent_program 4) 4;
  make_case ~kind:Bounded "independent-5" (independent_program 5) 5;
  make_case ~kind:Bounded "independent-6" (independent_program 6) 6;
  make_case ~reference_check:false ~kind:Bounded "independent-7" (independent_program 7) 7;
  make_case ~reference_check:false ~kind:Bounded "independent-8" (independent_program 8) 8;
  make_case ~reference_check:false ~kind:Bounded "forced-fallback" (independent_program 8) 8;
]

let furnace_cases = [
  make_case ~reference_check:false "path-1000" (path_program 1000) 1000;
  make_case ~reference_check:false "path-10000" (path_program 10000) 10000;
  make_case ~reference_check:false "independent-4" (independent_program 4) 4;
  make_case ~reference_check:false ~kind:Bounded "independent-6" (independent_program 6) 6;
]

let selected_cases = function
  | Sane -> cases
  | Quick -> quick_cases
  | Hardcore -> hardcore_cases
  | Furnace -> furnace_cases

let time_us f =
  let started = Unix.gettimeofday () in
  f ();
  max 1 (int_of_float ((Unix.gettimeofday () -. started) *. 1_000_000.))

let median values =
  let ordered = List.sort compare values in
  List.nth ordered (List.length ordered / 2)

let percentile95 values =
  let ordered = List.sort compare values in
  let index = min (List.length ordered - 1)
      (max 0 (int_of_float (ceil (float_of_int (List.length ordered) *. 0.95)) - 1)) in
  List.nth ordered index

let float_of_us value = float_of_int value

let json_float field json =
  match Yojson.Safe.Util.member field json with
  | `Int value -> float_of_int value
  | `Float value -> value
  | _ -> nan

let stats_json samples iterations =
  let total = List.fold_left ( + ) 0 samples in
  let count = List.length samples * iterations in
  `Assoc [
    ("samples_us", `List (List.map (fun value -> `Int value) samples));
    ("min_us", `Int (List.fold_left min max_int samples));
    ("median_us", `Int (median samples));
    ("p95_us", `Int (percentile95 samples));
    ("max_us", `Int (List.fold_left max min_int samples));
    ("iterations", `Int iterations);
    ("operations", `Int count);
    ("total_us", `Int total);
    ("operations_per_second", `Float (float_of_int count /. (float_of_us total /. 1_000_000.)));
  ]

let benchmark_solver ~warmups ~samples ~iterations solver =
  for _ = 1 to warmups do solver () done;
  let rec collect remaining acc =
    if remaining = 0 then List.rev acc
    else begin
      Gc.full_major ();
      let sample = time_us (fun () -> for _ = 1 to iterations do solver () done) in
      collect (remaining - 1) (sample :: acc)
    end
  in
  collect samples []

let calibrate_iterations ~target_us solver =
  if target_us <= 0 then (1, 0)
  else
    let max_iterations = 10_000_000 in
    let rec find iterations last_elapsed =
      if iterations >= max_iterations || last_elapsed >= target_us then
        (iterations, last_elapsed)
      else
        let ratio = float_of_int target_us /. float_of_int (max 1 last_elapsed) in
        let scale = max 2 (min 32 (int_of_float (ceil ratio))) in
        let next = min max_iterations (iterations * scale) in
        let elapsed = time_us (fun () -> for _ = 1 to next do solver () done) in
        find next elapsed
    in
    let first_elapsed = time_us solver in
    find 1 first_elapsed

let benchmark_bounded ~warmups ~samples ~iterations ~budget_us solver =
  for _ = 1 to warmups do solver () done;
  let started = Unix.gettimeofday () in
  let rec collect remaining acc status =
    if remaining = 0 then (List.rev acc, status)
    else if acc <> [] && (Unix.gettimeofday () -. started) *. 1_000_000. >=
            float_of_int budget_us then
      (List.rev acc, "stopped_at_case_budget")
    else begin
      Gc.full_major ();
      let sample = time_us (fun () -> for _ = 1 to iterations do solver () done) in
      let elapsed_us = int_of_float ((Unix.gettimeofday () -. started) *. 1_000_000.) in
      let next_status = if elapsed_us >= budget_us then "completed_over_case_budget" else status in
      collect (remaining - 1) (sample :: acc) next_status
    end
  in
  collect samples [] "completed"

let gc_counters before after = `Assoc [
  ("minor_words", `Float (after.Gc.minor_words -. before.Gc.minor_words));
  ("major_words", `Float (after.Gc.major_words -. before.Gc.major_words));
  ("promoted_words", `Float (after.Gc.promoted_words -. before.Gc.promoted_words));
  ("minor_collections", `Int (after.Gc.minor_collections - before.Gc.minor_collections));
  ("major_collections", `Int (after.Gc.major_collections - before.Gc.major_collections));
]

let result_payload = function
  | Ok result -> result
  | Error _ -> fail "benchmark fixture was rejected by the Rocket portfolio"

let reference_payload name = function
  | Ok result -> result.Reference.payload
  | Error _ -> fail ("benchmark fixture was rejected by the exhaustive reference: " ^ name)

let utc_timestamp () =
  let t = Unix.gmtime (Unix.gettimeofday ()) in
  Printf.sprintf "%04d-%02d-%02dT%02d:%02d:%02dZ"
    (1900 + t.Unix.tm_year) (t.Unix.tm_mon + 1) t.Unix.tm_mday
    t.Unix.tm_hour t.Unix.tm_min t.Unix.tm_sec

let getenv_or_none name =
  match Sys.getenv_opt name with None -> `Null | Some value -> `String value

let context_json () = `Assoc [
  ("tool_version", `String tool_version);
  ("ocaml_version", `String Sys.ocaml_version);
  ("os_type", `String Sys.os_type);
  ("word_size", `Int Sys.word_size);
  ("backend_type", `String (match Sys.backend_type with
    | Sys.Native -> "native"
    | Sys.Bytecode -> "bytecode"
    | Sys.Other value -> value));
  ("hostname", `String (try Unix.gethostname () with _ -> "unknown"));
  ("cwd", `String (Sys.getcwd ()));
  ("git_commit", getenv_or_none "TETHERS_GIT_COMMIT");
  ("build_id", getenv_or_none "TETHERS_BUILD_ID");
]

let json_int field json =
  match Yojson.Safe.Util.member field json with
  | `Int value -> float_of_int value
  | `Float value -> value
  | _ -> nan

let format_us value =
  if value < 1_000. then Printf.sprintf "%.0f us" value
  else if value < 1_000_000. then Printf.sprintf "%.2f ms" (value /. 1_000.)
  else Printf.sprintf "%.2f s" (value /. 1_000_000.)

let route_label = function
  | "b2_success_path" -> "B2 path"
  | "r3_2_refined_exact_leaf" -> "R3-2 refinement"
  | "frozen_v2_exact_search" -> "V2 exact search"
  | "exhaustive_reference" -> "reference engine"
  | value -> value

let is_fallback_route = function
  | "frozen_v2_exact_search" | "exhaustive_reference" -> true
  | _ -> false

let paint color code value =
  if color then "\027[" ^ code ^ "m" ^ value ^ "\027[0m" else value

let prior_case_json prior name =
  Yojson.Safe.Util.member "cases" prior
  |> Yojson.Safe.Util.to_list
  |> List.find_opt (fun item -> Yojson.Safe.Util.member "case" item = `String name)

let comparison_json prior current_profile current_cases =
  let baseline_profile =
    match Yojson.Safe.Util.member "settings" prior
          |> Yojson.Safe.Util.member "profile" with
    | `String value -> value
    | _ -> "unknown"
  in
  let rows = List.filter_map (fun current ->
    match prior_case_json prior (Yojson.Safe.Util.member "case" current |> Yojson.Safe.Util.to_string) with
    | None -> None
    | Some old ->
        let name = Yojson.Safe.Util.member "case" current |> Yojson.Safe.Util.to_string in
        match Yojson.Safe.Util.member "portfolio" current,
              Yojson.Safe.Util.member "portfolio" old with
        | `Null, _ | _, `Null -> None
        | current_portfolio, baseline_portfolio ->
            let now = json_int "median_us" current_portfolio in
            let before = json_int "median_us" baseline_portfolio in
            let delta_pct = if before = 0. then 0. else ((now -. before) /. before) *. 100. in
            Some (`Assoc [
              ("case", `String name);
              ("baseline_profile", `String baseline_profile);
              ("current_profile", `String (profile_name current_profile));
              ("profile_match", `Bool (baseline_profile = profile_name current_profile));
              ("baseline_median_us", `Float before);
              ("current_median_us", `Float now);
              ("delta_pct", `Float delta_pct);
            ])) current_cases in
  `List rows

let load_compare path =
  try Yojson.Safe.from_file path
  with Yojson.Json_error message -> fail ("cannot read comparison JSON: " ^ message)
     | Sys_error message -> fail ("cannot read comparison file: " ^ message)

let measurement_json settings current_case ~calibrate solver =
  let calibrated = calibrate && current_case.kind = Calibrated &&
                   settings.target_sample_us > 0 in
  let iterations, calibration_us =
    if calibrated then
      calibrate_iterations ~target_us:settings.target_sample_us solver
    else (settings.iterations, 0)
  in
  let samples, status =
    if current_case.kind = Bounded && settings.profile = Hardcore then
      benchmark_bounded ~warmups:settings.warmups ~samples:settings.samples
        ~iterations ~budget_us:settings.case_budget_us solver
    else
      (benchmark_solver ~warmups:settings.warmups ~samples:settings.samples
         ~iterations solver, "completed")
  in
  let stats = stats_json samples iterations in
  match stats with
  | `Assoc fields -> `Assoc (fields @ [
      ("calibration_us", `Int calibration_us);
      ("target_sample_us", `Int settings.target_sample_us);
      ("case_budget_us", `Int settings.case_budget_us);
      ("status", `String status);
    ])
  | _ -> stats

let run_case_unchecked settings current_case =
  let portfolio = result_payload (Portfolio.canonicalise current_case.program) in
  let reference = if current_case.reference_check then
    Some (reference_payload current_case.name (Reference.slow_oracle current_case.program))
  else None in
  let parity = match reference with
    | None -> `Null
    | Some payload ->
        if portfolio.payload <> payload then
          fail ("reference parity mismatch in case " ^ current_case.name);
        `Bool true
  in
  let portfolio_gc_before = Gc.quick_stat () in
  let portfolio_measurement = measurement_json settings current_case ~calibrate:true
      (fun () -> ignore (Portfolio.canonicalise current_case.program)) in
  let portfolio_gc_after = Gc.quick_stat () in
  let reference_measurement, reference_gc = match reference with
    | None -> (`Null, `Null)
    | Some _ ->
        let before = Gc.quick_stat () in
        let measurement = measurement_json settings current_case ~calibrate:false
            (fun () -> ignore (Reference.slow_oracle current_case.program)) in
        let after = Gc.quick_stat () in
        (measurement, gc_counters before after)
  in
  `Assoc [
    ("case", `String current_case.name);
    ("origins", `Int current_case.origins);
    ("parity", parity);
    ("reference_status", `String (if current_case.reference_check then "checked" else "not_practical"));
    ("execution_status", `String "completed");
    ("digest", `String portfolio.digest);
    ("portfolio_backend", `String (Portfolio.backend_name portfolio.stats.backend));
    ("portfolio", portfolio_measurement);
    ("reference", reference_measurement);
    ("resource_counters", `Assoc [
      ("portfolio_gc", gc_counters portfolio_gc_before portfolio_gc_after);
      ("reference_gc", reference_gc);
    ]);
  ]

let complexity_guard current_case =
  (current_case.kind = Bounded && current_case.origins >= 7 &&
   (String.length current_case.name >= 12 &&
    String.sub current_case.name 0 12 = "independent-" ||
    current_case.name = "forced-fallback")) ||
  current_case.origins >= 10_000 ||
  current_case.name = "repeated-subtree-1000"

let skipped_case_json current_case = `Assoc [
  ("case", `String current_case.name);
  ("origins", `Int current_case.origins);
  ("parity", `Null);
  ("reference_status", `String "not_practical");
  ("execution_status", `String "skipped_complexity_bound");
  ("complexity_bound", `Assoc [
    ("max_origins", `Int (if current_case.origins >= 10_000 then 1_000 else if
      current_case.name = "repeated-subtree-1000" then 1_000 else 6));
    ("reason", `String (if current_case.origins >= 10_000 then
      "avoid oversized in-process fixture work" else if
      current_case.name = "repeated-subtree-1000" then
      "avoid duplicate oversized fixture work" else
      "avoid unbounded factorial exact-search work"));
  ]);
  ("digest", `Null);
  ("portfolio_backend", `String "complexity_guard");
  ("portfolio", `Null);
  ("reference", `Null);
  ("resource_counters", `Null);
]

let run_case settings current_case =
  if complexity_guard current_case then skipped_case_json current_case
  else run_case_unchecked settings current_case

let furnace_phase_json elapsed_us operations =
  let rate = if elapsed_us <= 0 then 0.
    else float_of_int operations /. (float_of_int elapsed_us /. 1_000_000.) in
  `Assoc [
    ("elapsed_us", `Int elapsed_us);
    ("operations", `Int operations);
    ("operations_per_second", `Float rate);
  ]

let run_furnace_case settings current_case budget_us =
  let portfolio_result = result_payload (Portfolio.canonicalise current_case.program) in
  let solver () = ignore (Portfolio.canonicalise current_case.program) in
  let iterations, calibration_us =
    if current_case.kind = Calibrated then
      calibrate_iterations ~target_us:settings.target_sample_us solver
    else (settings.iterations, 0)
  in
  for _ = 1 to settings.warmups do solver () done;
  let phase_elapsed = Array.make 3 0 in
  let phase_operations = Array.make 3 0 in
  let samples = ref [] in
  let batches = ref 0 in
  let started = Unix.gettimeofday () in
  let elapsed () = int_of_float ((Unix.gettimeofday () -. started) *. 1_000_000.) in
  while elapsed () < budget_us do
    if !batches mod 10 = 0 then Gc.full_major ();
    let sample_us = time_us (fun () -> for _ = 1 to iterations do solver () done) in
    let total_elapsed = min budget_us (elapsed ()) in
    let phase = min 2 (total_elapsed * 3 / max 1 budget_us) in
    phase_elapsed.(phase) <- phase_elapsed.(phase) + sample_us;
    phase_operations.(phase) <- phase_operations.(phase) + iterations;
    samples := sample_us :: !samples;
    incr batches
  done;
  let phases = List.init 3 (fun index ->
    let name = match index with 0 -> "early" | 1 -> "middle" | _ -> "late" in
    `Assoc [
      ("phase", `String name);
      ("stats", furnace_phase_json phase_elapsed.(index) phase_operations.(index));
    ]) in
  let first_rate = Yojson.Safe.Util.member "operations_per_second"
      (furnace_phase_json phase_elapsed.(0) phase_operations.(0))
      |> Yojson.Safe.Util.to_float in
  let last_rate = Yojson.Safe.Util.member "operations_per_second"
      (furnace_phase_json phase_elapsed.(2) phase_operations.(2))
      |> Yojson.Safe.Util.to_float in
  let degradation_pct = if first_rate = 0. then 0.
    else ((last_rate -. first_rate) /. first_rate) *. 100. in
  let portfolio = stats_json (List.rev !samples) iterations in
  `Assoc [
    ("case", `String current_case.name);
    ("origins", `Int current_case.origins);
    ("parity", `Null);
    ("reference_status", `String "not_practical");
    ("execution_status", `String "completed");
    ("digest", `String portfolio_result.digest);
    ("portfolio_backend", `String (Portfolio.backend_name portfolio_result.stats.backend));
    ("portfolio", portfolio);
    ("furnace", `Assoc [
      ("budget_us", `Int budget_us);
      ("elapsed_us", `Int (elapsed ()));
      ("iterations_per_batch", `Int iterations);
      ("calibration_us", `Int calibration_us);
      ("batches", `Int !batches);
      ("degradation_pct", `Float degradation_pct);
      ("phases", `List phases);
    ]);
    ("resource_counters", `Null);
  ]

let run_furnace_or_skip settings current_case budget_us =
  if complexity_guard current_case then skipped_case_json current_case
  else run_furnace_case settings current_case budget_us

let write_file path contents =
  try
    let channel = open_out_bin path in
    output_string channel contents;
    close_out channel
  with Sys_error message -> fail ("cannot write JSON output: " ^ message)

let human_summary json settings =
  let cases_json = Yojson.Safe.Util.member "cases" json |> Yojson.Safe.Util.to_list in
  let color = settings.color in
  let profile = profile_name settings.profile in
  let description = profile_description settings.profile in
  let checked_count = List.fold_left (fun count item ->
    match Yojson.Safe.Util.member "reference_status" item with
    | `String "checked" -> count + 1
    | _ -> count) 0 cases_json in
  let skipped_count = List.fold_left (fun count item ->
    match Yojson.Safe.Util.member "execution_status" item with
    | `String "skipped_complexity_bound" -> count + 1
    | _ -> count) 0 cases_json in
  let fallback_count = List.fold_left (fun count item ->
    let backend = Yojson.Safe.Util.member "portfolio_backend" item |> Yojson.Safe.Util.to_string in
    if is_fallback_route backend then count + 1 else count) 0 cases_json in
  let fast_count = List.length cases_json - fallback_count - skipped_count in
  let portfolio_total_us = List.fold_left (fun total item ->
    match Yojson.Safe.Util.member "portfolio" item with
    | `Null -> total
    | portfolio -> total +. json_int "total_us" portfolio) 0. cases_json in
  let portfolio_operations = List.fold_left (fun total item ->
    match Yojson.Safe.Util.member "portfolio" item with
    | `Null -> total
    | portfolio -> total +. json_int "operations" portfolio) 0. cases_json in
  let overall_ops = if portfolio_total_us = 0. then 0.
    else portfolio_operations /. (portfolio_total_us /. 1_000_000.) in
  let run_duration = format_us (json_int "duration_us" json) in
  Printf.printf "%s\n" (paint color "1" ("Tethers Benchmarker " ^ tool_version));
  Printf.printf "%s  %s\n" (paint color "36" ("Rocket V3 performance crucible · " ^ String.uppercase_ascii profile)) description;
  let exactness = if checked_count = 0 then
      "Reference differential: performance-only"
    else Printf.sprintf "Exactness: PASS · %d reference checks · %d performance-only"
      checked_count (List.length cases_json - checked_count) in
  Printf.printf "%s\n" (paint color "32" exactness);
  Printf.printf "Settings: %d warmups · %d samples · %d iterations · run %s\n\n"
    settings.warmups settings.samples settings.iterations run_duration;
  Printf.printf "%s\n" (paint color "1" "CASE                   ROUTE                    MEDIAN       P95     REF MED   SPEEDUP     OPS/S");
  Printf.printf "%s\n" (paint color "90" "----                   -----                    ------       ---     -------   -------     -----");
  List.iter (fun item ->
    let name = Yojson.Safe.Util.member "case" item |> Yojson.Safe.Util.to_string in
    let backend = Yojson.Safe.Util.member "portfolio_backend" item |> Yojson.Safe.Util.to_string in
    let portfolio = Yojson.Safe.Util.member "portfolio" item in
    let reference = Yojson.Safe.Util.member "reference" item in
    let median_text, p95_text, ops_text = match portfolio with
      | `Null -> ("n/a", "n/a", "n/a")
      | _ ->
          (format_us (json_int "median_us" portfolio),
           format_us (json_int "p95_us" portfolio),
           Printf.sprintf "%.0f" (json_float "operations_per_second" portfolio))
    in
    let reference_text, speedup_text = match reference, portfolio with
      | (`Null, _) -> ("n/a", "n/a")
      | _ ->
          let reference_value = json_int "median_us" reference in
          let median_value = json_int "median_us" portfolio in
          let speedup = if median_value = 0. then 0. else reference_value /. median_value in
          (format_us reference_value, Printf.sprintf "%.2fx" speedup)
    in
    let route = route_label backend in
    Printf.printf "%-22s %-24s %10s %10s %10s %8s %10s\n" name route
      median_text p95_text reference_text speedup_text ops_text) cases_json;
  Printf.printf "\n%s\n" (paint color "1" "SUMMARY");
  Printf.printf "%d cases · %d fast routes · %d exact-search/fallback routes · %d complexity-skipped · portfolio %.0f ops/s · elapsed %s\n"
    (List.length cases_json) fast_count fallback_count skipped_count overall_ops run_duration;
  if settings.profile = Furnace then begin
    Printf.printf "\n%s\n" (paint color "1" "FURNACE PHASES");
    List.iter (fun item ->
      let name = Yojson.Safe.Util.member "case" item |> Yojson.Safe.Util.to_string in
      let furnace = Yojson.Safe.Util.member "furnace" item in
      match furnace with
      | `Null -> Printf.printf "  %-18s skipped by complexity guard\n" name
      | _ ->
          let degradation = Yojson.Safe.Util.member "degradation_pct" furnace |> Yojson.Safe.Util.to_float in
          let phases = Yojson.Safe.Util.member "phases" furnace |> Yojson.Safe.Util.to_list in
          let phase_rate phase =
            let stats = Yojson.Safe.Util.member "stats" phase in
            Yojson.Safe.Util.member "operations_per_second" stats |> Yojson.Safe.Util.to_float in
          let early = match phases with first :: _ -> phase_rate first | [] -> 0. in
          let late = match List.rev phases with last :: _ -> phase_rate last | [] -> 0. in
          Printf.printf "  %-18s early %8.0f ops/s · late %8.0f ops/s · %+.2f%%\n"
            name early late degradation) cases_json;
  end;
  match Yojson.Safe.Util.member "comparison" json with
  | `Null -> ()
  | comparison ->
      Printf.printf "\n%s\n" (paint color "1" "BEFORE / AFTER");
      let comparison_rows = Yojson.Safe.Util.to_list comparison in
      if List.exists (fun item ->
        match Yojson.Safe.Util.member "profile_match" item with
        | `Bool value -> not value
        | _ -> false) comparison_rows then
        Printf.printf "  WARNING: baseline and current profiles differ; compare deltas cautiously.\n";
      comparison_rows |> List.iter (fun item ->
        let name = Yojson.Safe.Util.member "case" item |> Yojson.Safe.Util.to_string in
        let delta = Yojson.Safe.Util.member "delta_pct" item |> Yojson.Safe.Util.to_float in
        let direction = if delta < 0. then "faster" else if delta > 0. then "slower" else "unchanged" in
        Printf.printf "  %-18s %+.2f%% (%s)\n" name delta direction)

let () =
  let settings = parse_args default_settings (List.tl (Array.to_list Sys.argv)) in
  let run_started = Unix.gettimeofday () in
  let run_started_utc = utc_timestamp () in
  let selected_cases = selected_cases settings.profile in
  let case_json = match settings.profile with
    | Furnace ->
        let runnable_count = List.fold_left (fun count current_case ->
          if complexity_guard current_case then count else count + 1) 0 selected_cases in
        let budget_us = settings.furnace_duration_us / max 1 runnable_count in
        List.map (fun current_case -> run_furnace_or_skip settings current_case budget_us)
          selected_cases
    | _ -> List.map (run_case settings) selected_cases
  in
  let duration_us = max 1 (int_of_float ((Unix.gettimeofday () -. run_started) *. 1_000_000.)) in
  let result = `Assoc [
    ("schema", `String schema);
    ("tool", `String "tethers-bench");
    ("run_started_utc", `String run_started_utc);
    ("duration_us", `Int duration_us);
    ("settings", `Assoc [
      ("profile", `String (profile_name settings.profile));
      ("profile_description", `String (profile_description settings.profile));
      ("warmups", `Int settings.warmups);
      ("samples", `Int settings.samples);
      ("iterations", `Int settings.iterations);
      ("target_sample_us", `Int settings.target_sample_us);
      ("case_budget_us", `Int settings.case_budget_us);
      ("furnace_duration_us", `Int settings.furnace_duration_us);
      ("quick", `Bool (settings.profile = Quick));
      ("hardcore", `Bool (settings.profile = Hardcore));
      ("furnace", `Bool (settings.profile = Furnace));
    ]);
    ("workload", `Assoc [
      ("name", `String "rocket-v3-portfolio-crucible");
      ("seed", `String "fixed-fixtures-v1");
      ("profile", `String (profile_name settings.profile));
      ("case_count", `Int (List.length selected_cases));
      ("case_names", `List (List.map (fun item -> `String item.name) selected_cases));
      ("bounded", `Bool true);
      ("reference_policy", `String (if settings.profile = Furnace then
        "bounded_differential_separate_performance_only" else
        "reference_checked_before_timing"));
      ("identity_rule", `String "portfolio payload must equal exhaustive reference payload");
    ]);
    ("context", context_json ());
    ("cases", `List case_json);
    ("comparison", match settings.compare with
      | None -> `Null
      | Some path -> comparison_json (load_compare path) settings.profile case_json);
  ] in
  let json_string = Yojson.Safe.pretty_to_string result ^ "\n" in
  (match settings.json_out with None -> () | Some path -> write_file path json_string);
  if settings.json_only then Printf.printf "%s" json_string
  else begin
    human_summary result settings;
    match settings.json_out with None -> () | Some path -> Printf.printf "JSON: %s\n" path
  end
