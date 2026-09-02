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
  json_only : bool;
  json_out : string option;
  compare : string option;
  color : bool;
}

and profile = Sane | Quick | Hardcore

let profile_name = function
  | Sane -> "sane"
  | Quick -> "quick"
  | Hardcore -> "hardcore"

let profile_description = function
  | Sane -> "balanced defaults for everyday comparison"
  | Quick -> "small smoke run for fast feedback"
  | Hardcore -> "larger bounded crucible with deeper sampling"

let settings_for_profile settings profile =
  match profile with
  | Sane -> { settings with profile; warmups = 3; samples = 7; iterations = 5 }
  | Quick -> { settings with profile; warmups = 1; samples = 3; iterations = 1 }
  | Hardcore -> { settings with profile; warmups = 3; samples = 9; iterations = 3 }

let default_settings = {
  profile = Sane;
  warmups = 3;
  samples = 7;
  iterations = 5;
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
  | _ -> fail ("unknown profile " ^ value ^ " (expected sane, quick, or hardcore)")

let rec parse_args settings = function
  | [] -> settings
  | "--quick" :: rest ->
      parse_args (settings_for_profile settings Quick) rest
  | "--hardcore" :: rest ->
      parse_args (settings_for_profile settings Hardcore) rest
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
  | "--help" :: _ ->
      Printf.printf
        "Tethers Benchmarker %s\n\nUsage: tethers-bench [options]\n\n\
Measures the fixed Rocket/V2 crucible after exhaustive-reference parity checks.\n\
  no options              sane balanced defaults\n\
  --quick                 small bounded smoke run\n\
  --hardcore              larger bounded crucible\n\
  --profile NAME          sane, quick, or hardcore\n\
  --json                  write JSON only to stdout\n\
  --json-out PATH         save JSON while retaining human output\n\
  --compare PATH          compare portfolio medians with a prior JSON run\n\
  --warmups N             override warmup executions per case\n\
  --samples N             override timed samples per solver/case\n\
  --iterations N          override executions per timed sample\n\
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

type case = { name : string; program : Core.program; origins : int }

let cases = [
  { name = "empty"; program = empty_program "empty"; origins = 0 };
  { name = "path-4"; program = path_program 4; origins = 4 };
  { name = "path-5"; program = path_program 5; origins = 5 };
  { name = "independent-4"; program = independent_program 4; origins = 4 };
  { name = "independent-5"; program = independent_program 5; origins = 5 };
]

let quick_cases = [
  { name = "empty"; program = empty_program "empty"; origins = 0 };
  { name = "path-4"; program = path_program 4; origins = 4 };
  { name = "independent-4"; program = independent_program 4; origins = 4 };
]

let hardcore_cases = [
  { name = "empty"; program = empty_program "empty"; origins = 0 };
  { name = "path-4"; program = path_program 4; origins = 4 };
  { name = "path-5"; program = path_program 5; origins = 5 };
  { name = "path-6"; program = path_program 6; origins = 6 };
  { name = "independent-4"; program = independent_program 4; origins = 4 };
  { name = "independent-5"; program = independent_program 5; origins = 5 };
  { name = "independent-6"; program = independent_program 6; origins = 6 };
]

let selected_cases = function
  | Sane -> cases
  | Quick -> quick_cases
  | Hardcore -> hardcore_cases

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
        let now = Yojson.Safe.Util.member "portfolio" current |> json_int "median_us" in
        let before = Yojson.Safe.Util.member "portfolio" old |> json_int "median_us" in
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

let run_case settings current_case =
  let portfolio = result_payload (Portfolio.canonicalise current_case.program) in
  let reference = reference_payload current_case.name (Reference.slow_oracle current_case.program) in
  if portfolio.payload <> reference then
    fail ("reference parity mismatch in case " ^ current_case.name);
  let portfolio_gc_before = Gc.quick_stat () in
  let portfolio_samples = benchmark_solver ~warmups:settings.warmups
      ~samples:settings.samples ~iterations:settings.iterations
      (fun () -> ignore (Portfolio.canonicalise current_case.program)) in
  let portfolio_gc_after = Gc.quick_stat () in
  let reference_gc_before = Gc.quick_stat () in
  let reference_samples = benchmark_solver ~warmups:settings.warmups
      ~samples:settings.samples ~iterations:settings.iterations
      (fun () -> ignore (Reference.slow_oracle current_case.program)) in
  let reference_gc_after = Gc.quick_stat () in
  `Assoc [
    ("case", `String current_case.name);
    ("origins", `Int current_case.origins);
    ("parity", `Bool true);
    ("digest", `String portfolio.digest);
    ("portfolio_backend", `String (Portfolio.backend_name portfolio.stats.backend));
    ("portfolio", stats_json portfolio_samples settings.iterations);
    ("reference", stats_json reference_samples settings.iterations);
    ("resource_counters", `Assoc [
      ("portfolio_gc", gc_counters portfolio_gc_before portfolio_gc_after);
      ("reference_gc", gc_counters reference_gc_before reference_gc_after);
    ]);
  ]

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
  let fallback_count = List.fold_left (fun count item ->
    let backend = Yojson.Safe.Util.member "portfolio_backend" item |> Yojson.Safe.Util.to_string in
    if is_fallback_route backend then count + 1 else count) 0 cases_json in
  let fast_count = List.length cases_json - fallback_count in
  let portfolio_total_us = List.fold_left (fun total item ->
    total +. json_int "total_us" (Yojson.Safe.Util.member "portfolio" item)) 0. cases_json in
  let portfolio_operations = List.fold_left (fun total item ->
    total +. json_int "operations" (Yojson.Safe.Util.member "portfolio" item)) 0. cases_json in
  let overall_ops = if portfolio_total_us = 0. then 0.
    else portfolio_operations /. (portfolio_total_us /. 1_000_000.) in
  let run_duration = format_us (json_int "duration_us" json) in
  Printf.printf "%s\n" (paint color "1" ("Tethers Benchmarker " ^ tool_version));
  Printf.printf "%s  %s\n" (paint color "36" ("Rocket V3 performance crucible · " ^ String.uppercase_ascii profile)) description;
  Printf.printf "%s\n" (paint color "32" (Printf.sprintf "Exactness: PASS · %d cases · fixed fixtures" (List.length cases_json)));
  Printf.printf "Settings: %d warmups · %d samples · %d iterations · run %s\n\n"
    settings.warmups settings.samples settings.iterations run_duration;
  Printf.printf "%s\n" (paint color "1" "CASE               ROUTE                    MEDIAN       P95     REF MED   SPEEDUP     OPS/S");
  Printf.printf "%s\n" (paint color "90" "----               -----                    ------       ---     -------   -------     -----");
  List.iter (fun item ->
    let name = Yojson.Safe.Util.member "case" item |> Yojson.Safe.Util.to_string in
    let backend = Yojson.Safe.Util.member "portfolio_backend" item |> Yojson.Safe.Util.to_string in
    let portfolio = Yojson.Safe.Util.member "portfolio" item in
    let reference = Yojson.Safe.Util.member "reference" item in
    let median_value = json_int "median_us" portfolio in
    let p95_value = json_int "p95_us" portfolio in
    let reference_value = json_int "median_us" reference in
    let speedup = if median_value = 0. then 0. else reference_value /. median_value in
    let route = route_label backend in
    Printf.printf "%-18s %-24s %10s %10s %10s %8.2fx %10.0f\n" name route
      (format_us median_value) (format_us p95_value) (format_us reference_value)
      speedup (json_float "operations_per_second" portfolio)) cases_json;
  Printf.printf "\n%s\n" (paint color "1" "SUMMARY");
  Printf.printf "%d cases · %d fast routes · %d exact-search/fallback routes · portfolio %.0f ops/s · elapsed %s\n"
    (List.length cases_json) fast_count fallback_count overall_ops run_duration;
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
  let case_json = List.map (run_case settings) selected_cases in
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
      ("quick", `Bool (settings.profile = Quick));
      ("hardcore", `Bool (settings.profile = Hardcore));
    ]);
    ("workload", `Assoc [
      ("name", `String "rocket-v3-portfolio-crucible");
      ("seed", `String "fixed-fixtures-v1");
      ("profile", `String (profile_name settings.profile));
      ("case_count", `Int (List.length selected_cases));
      ("case_names", `List (List.map (fun item -> `String item.name) selected_cases));
      ("bounded", `Bool true);
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
