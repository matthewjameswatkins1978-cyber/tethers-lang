(* ==================================================================
   PERFORMANCE HARNESS
   NOT A NORMAL TEST
   FULL MODE MAY BE SLOW

   B0 semantic/runtime baseline: 1ce6b10f1de3cd10fef619483df444f83899c870
   ================================================================== *)

(* Tethers Core Pipeline Microbenchmark (B0-A)

   Measures the canonical semantic/planning pipeline in-process:
     Human source -> parser -> lowerer -> validator -> canonicalisation
     -> ProgramDigest -> canonical Runtime Plan

   Invocations:
     --quick           bounded smoke (P1 + P10, tiny sample counts)
     --profile-stages  PF1 stage profiler + shape probe
     (default)         full B0-A matrix; use only for explicit full runs

   Uses high-resolution monotonic timing via Unix.gettimeofday.
   Reports batch statistics: median, p95, min, max, mean, stddev. *)

(* ================================================================== *)
(*  High-resolution timer                                              *)
(* ================================================================== *)

let now_seconds () = Unix.gettimeofday ()

(* ================================================================== *)
(*  Statistics                                                         *)
(* ================================================================== *)

let sort_floats a = Array.sort compare a; a

let median sorted =
  let n = Array.length sorted in
  if n = 0 then 0.0
  else if n mod 2 = 1 then sorted.(n / 2)
  else (sorted.(n / 2 - 1) +. sorted.(n / 2)) /. 2.0

let percentile sorted p =
  let n = Array.length sorted in
  if n = 0 then 0.0
  else
    let idx = min (n - 1) (int_of_float (p /. 100.0 *. float_of_int (n - 1))) in
    sorted.(idx)

let mean a = Array.fold_left (+.) 0.0 a /. float_of_int (Array.length a)

let stddev a =
  let m = mean a in
  let variance =
    Array.fold_left (fun acc x -> acc +. ((x -. m) *. (x -. m))) 0.0 a
    /. float_of_int (Array.length a)
  in
  sqrt variance

type stats = {
  sample_count : int;
  median_us : float;
  p95_us : float;
  min_us : float;
  max_us : float;
  mean_us : float;
  stddev_us : float;
  ops_per_sec : float;
}

let compute_stats (times_us : float array) =
  let sorted = sort_floats (Array.copy times_us) in
  let med = median sorted in
  let p95 = percentile sorted 95.0 in
  let mn = sorted.(0) in
  let mx = sorted.(Array.length sorted - 1) in
  let m = mean sorted in
  let sd = stddev sorted in
  let ops = if m > 0.0 then 1_000_000.0 /. m else 0.0 in
  {
    sample_count = Array.length sorted;
    median_us = med;
    p95_us = p95;
    min_us = mn;
    max_us = mx;
    mean_us = m;
    stddev_us = sd;
    ops_per_sec = ops;
  }

(* ================================================================== *)
(*  JSON output helpers                                                *)
(* ================================================================== *)

let json_of_stats (s : stats) =
  `Assoc
    [
      ("sample_count", `Int s.sample_count);
      ("median_us", `Float s.median_us);
      ("p95_us", `Float s.p95_us);
      ("min_us", `Float s.min_us);
      ("max_us", `Float s.max_us);
      ("mean_us", `Float s.mean_us);
      ("stddev_us", `Float s.stddev_us);
      ("ops_per_sec", `Float s.ops_per_sec);
    ]

(* ================================================================== *)
(*  Benchmark fixture: build a JSON request for n sequential actions   *)
(* ================================================================== *)

let make_ping_request ~eval_id ~evt_id ~num_actions =
  let tether_source =
    let buf = Buffer.create 1024 in
    Buffer.add_string buf
      "tether \"benchmark ping\"\n\nanchor\n    fixture.start\n\nwhen\ndo\n";
    for _ = 1 to num_actions do
      Buffer.add_string buf "    fixture.ping\n        message: anchor.message\n"
    done;
    Buffer.contents buf
  in
  let actions_json =
    `List
      [
        `Assoc
          [
            ("name", `String "fixture.ping");
            ("version", `String "1.0.0");
            ("inputs", `Assoc [ ("message", `String "string") ]);
            ("effects", `List [ `String "fixture.test" ]);
            ("reversibility", `String "compensatable");
          ];
      ]
  in
  `Assoc
    [
      ("protocol_version", `String "0.1");
      ("language_version", `String "0.1");
      ("evaluation_id", `String eval_id);
      ( "tether",
        `Assoc
          [
            ("id", `String "benchmark-ping");
            ("version", `String "1");
            ("source", `String tether_source);
          ] );
      ( "event",
        `Assoc
          [
            ("id", `String evt_id);
            ("name", `String "fixture.start");
            ("data", `Assoc [ ("message", `String "hello") ]);
          ] );
      ("facts", `Assoc []);
      ("capabilities", actions_json);
      ( "core_environment",
        `Assoc
          [
            ("program_id", `String "program.benchmark");
            ("core_version", `String "1");
            ( "capabilities",
              `List
                [
                  `Assoc
                    [
                      ("source_name", `String "fixture.ping");
                      ("capability_id", `String "cap.benchmark.ping");
                      ("contract_digest", `String "BENCH-CONTRACT-0");
                      ("runtime_name", `String "fixture.ping");
                    ];
                ] );
            ("input_facts", `List []);
          ] );
    ]

let make_not_matched_request ~eval_id ~evt_id =
  `Assoc
    [
      ("protocol_version", `String "0.1");
      ("language_version", `String "0.1");
      ("evaluation_id", `String eval_id);
      ( "tether",
        `Assoc
          [
            ("id", `String "benchmark-ping");
            ("version", `String "1");
            ( "source",
              `String
                "tether \"benchmark ping\"\n\nanchor\n    fixture.start\n\n\
                 when\n\ndo\n    fixture.ping\n        message: \
                 anchor.message\n" );
          ] );
      ( "event",
        `Assoc
          [
            ("id", `String evt_id);
            ("name", `String "fixture.wrong_event");
            ("data", `Assoc []);
          ] );
      ("facts", `Assoc []);
      ( "capabilities",
        `List
          [
            `Assoc
              [
                ("name", `String "fixture.ping");
                ("version", `String "1.0.0");
                ("inputs", `Assoc [ ("message", `String "string") ]);
                ("effects", `List [ `String "fixture.test" ]);
                ("reversibility", `String "compensatable");
              ];
          ] );
      ( "core_environment",
        `Assoc
          [
            ("program_id", `String "program.benchmark");
            ("core_version", `String "1");
            ( "capabilities",
              `List
                [
                  `Assoc
                    [
                      ("source_name", `String "fixture.ping");
                      ("capability_id", `String "cap.benchmark.ping");
                      ("contract_digest", `String "BENCH-CONTRACT-0");
                      ("runtime_name", `String "fixture.ping");
                    ];
                ] );
            ("input_facts", `List []);
          ] );
    ]

(* PC10: 10 actions with entry conditions *)
let make_pc10_request ~eval_id ~evt_id ~num_actions =
  let tether_source =
    let buf = Buffer.create 1024 in
    Buffer.add_string buf
      "tether \"benchmark conditions\"\n\nanchor\n    fixture.start\n\nwhen\n\
       \    project.type is \"software\"\n    and task.count greater_than 0\n\
       do\n";
    for _ = 1 to num_actions do
      Buffer.add_string buf "    fixture.ping\n        message: anchor.message\n"
    done;
    Buffer.contents buf
  in
  `Assoc
    [
      ("protocol_version", `String "0.1");
      ("language_version", `String "0.1");
      ("evaluation_id", `String eval_id);
      ( "tether",
        `Assoc
          [
            ("id", `String "benchmark-pc10");
            ("version", `String "1");
            ("source", `String tether_source);
          ] );
      ( "event",
        `Assoc
          [
            ("id", `String evt_id);
            ("name", `String "fixture.start");
            ("data", `Assoc [ ("message", `String "hello") ]);
          ] );
      ( "facts",
        `Assoc
          [
            ("project.type", `String "software");
            ("task.count", `Int 5);
          ] );
      ( "capabilities",
        `List
          [
            `Assoc
              [
                ("name", `String "fixture.ping");
                ("version", `String "1.0.0");
                ("inputs", `Assoc [ ("message", `String "string") ]);
                ("effects", `List [ `String "fixture.test" ]);
                ("reversibility", `String "compensatable");
              ];
          ] );
      ( "core_environment",
        `Assoc
          [
            ("program_id", `String "program.benchmark");
            ("core_version", `String "1");
            ( "capabilities",
              `List
                [
                  `Assoc
                    [
                      ("source_name", `String "fixture.ping");
                      ("capability_id", `String "cap.benchmark.ping");
                      ("contract_digest", `String "BENCH-CONTRACT-0");
                      ("runtime_name", `String "fixture.ping");
                    ];
                ] );
            ( "input_facts",
              `List
                [
                  `Assoc
                    [
                      ("source_name", `String "project.type");
                      ("fact_id", `String "fact.project_type");
                      ("host_snapshot_key", `String "project.type");
                      ("scalar_type", `String "string");
                      ("schema_description", `String "project type");
                    ];
                  `Assoc
                    [
                      ("source_name", `String "task.count");
                      ("fact_id", `String "fact.task_count");
                      ("host_snapshot_key", `String "task.count");
                      ("scalar_type", `String "integer");
                      ("schema_description", `String "task count");
                    ];
                ] );
          ] );
    ]

(* PA10: 10 actions using anchor references *)
let make_pa10_request ~eval_id ~evt_id ~num_actions =
  let tether_source =
    let buf = Buffer.create 1024 in
    Buffer.add_string buf
      "tether \"benchmark anchor refs\"\n\nanchor\n    fixture.start\n\nwhen\ndo\n";
    for _ = 1 to num_actions do
      Buffer.add_string buf
        "    fixture.ping\n        message: anchor.message\n"
    done;
    Buffer.contents buf
  in
  `Assoc
    [
      ("protocol_version", `String "0.1");
      ("language_version", `String "0.1");
      ("evaluation_id", `String eval_id);
      ( "tether",
        `Assoc
          [
            ("id", `String "benchmark-pa10");
            ("version", `String "1");
            ("source", `String tether_source);
          ] );
      ( "event",
        `Assoc
          [
            ("id", `String evt_id);
            ("name", `String "fixture.start");
            ("data", `Assoc [ ("message", `String "hello from anchor") ]);
          ] );
      ("facts", `Assoc []);
      ( "capabilities",
        `List
          [
            `Assoc
              [
                ("name", `String "fixture.ping");
                ("version", `String "1.0.0");
                ("inputs", `Assoc [ ("message", `String "string") ]);
                ("effects", `List [ `String "fixture.test" ]);
                ("reversibility", `String "compensatable");
              ];
          ] );
      ( "core_environment",
        `Assoc
          [
            ("program_id", `String "program.benchmark");
            ("core_version", `String "1");
            ( "capabilities",
              `List
                [
                  `Assoc
                    [
                      ("source_name", `String "fixture.ping");
                      ("capability_id", `String "cap.benchmark.ping");
                      ("contract_digest", `String "BENCH-CONTRACT-0");
                      ("runtime_name", `String "fixture.ping");
                    ];
                ] );
            ("input_facts", `List []);
          ] );
    ]

(* ================================================================== *)
(*  Benchmark runner                                                   *)
(* ================================================================== *)

let run_benchmark ~label ~request_json ~warmup_count ~batch_size ~num_batches =
  (* Warmup *)
  for _ = 1 to warmup_count do
    let _ = Tethers_core_wire.evaluate_request_json request_json in
    ()
  done;
  (* Measure: time batches of batch_size evaluations *)
  let times = Array.make num_batches 0.0 in
  for i = 0 to num_batches - 1 do
    let t0 = now_seconds () in
    for _ = 1 to batch_size do
      let _ = Tethers_core_wire.evaluate_request_json request_json in
      ()
    done;
    let t1 = now_seconds () in
    times.(i) <- (t1 -. t0) *. 1_000_000.0 /. float_of_int batch_size
  done;
  let stats = compute_stats times in
  let raw = Array.to_list times |> List.map (fun x -> `Float x) in
  Printf.printf "  %s: median=%.1fus p95=%.1fus mean=%.1fus ops/sec=%.0f (batch=%d x%d)\n%!"
    label stats.median_us stats.p95_us stats.mean_us stats.ops_per_sec batch_size num_batches;
  (stats, raw)

(* ================================================================== *)
(*  ProgramDigest sanity check                                         *)
(* ================================================================== *)

let verify_program_digest_stability () =
  let req = make_ping_request ~eval_id:"digest_check" ~evt_id:"evt_dc" ~num_actions:1 in
  let r1 = Tethers_core_wire.evaluate_request_json req in
  let r2 = Tethers_core_wire.evaluate_request_json req in
  let pd1 =
    match Yojson.Safe.Util.member "program_digest" r1 with
    | `String s -> s | _ -> "MISSING"
  in
  let pd2 =
    match Yojson.Safe.Util.member "program_digest" r2 with
    | `String s -> s | _ -> "MISSING"
  in
  if pd1 = pd2 && pd1 <> "MISSING" then
    Printf.printf "  ProgramDigest stability: PASS (%s)\n%!" pd1
  else
    Printf.printf "  ProgramDigest stability: FAIL (mismatch or missing)\n%!"

(* ================================================================== *)
(*  Result correctness check                                           *)
(* ================================================================== *)

let verify_correctness ~label ~request_json ~expected_status =
  let response = Tethers_core_wire.evaluate_request_json request_json in
  let status =
    match Yojson.Safe.Util.member "status" response with
    | `String s -> s | _ -> "unknown"
  in
  if status = expected_status then
    Printf.printf "  Correctness %s: PASS (status=%s)\n%!" label status
  else
    Printf.printf "  Correctness %s: FAIL (expected=%s got=%s)\n%!"
      label expected_status status

(* ================================================================== *)
(*  PF1 stage profiler (Part E / F)                                    *)
(*                                                                     *)
(*  Times each production Core stage separately by decoding one         *)
(*  request JSON and driving the same public module functions the       *)
(*  adapter uses.  Benchmark-only; no production module is changed.     *)
(* ================================================================== *)

let object_fields = function
  | `Assoc fields -> fields
  | _ -> failwith "PF1 stage decode: expected object"

let field_string fields name =
  match List.assoc_opt name fields with
  | Some (`String s) -> s
  | _ -> failwith ("PF1 stage decode: missing string field " ^ name)

let field_object fields name =
  match List.assoc_opt name fields with
  | Some (`Assoc _ as v) -> v
  | _ -> failwith ("PF1 stage decode: missing object field " ^ name)

let field_list fields name =
  match List.assoc_opt name fields with
  | Some (`List l) -> l
  | _ -> failwith ("PF1 stage decode: missing list field " ^ name)

let parse_scalar_type = function
  | "string" -> `String_type
  | "integer" -> `Integer_type
  | "boolean" -> `Boolean_type
  | s -> failwith ("PF1 stage decode: bad scalar_type " ^ s)

(* Decode one generated request JSON into the typed intermediate values
   the production pipeline consumes.  Mirrors
   Tethers_core_request_adapter.parse_request + parse_core_env so that each
   stage can be timed in isolation.  The staged pipeline is sanity-checked
   against the wire result before timing is trusted. *)
let decode_stage_inputs request_json =
  let obj = object_fields request_json in
  let evaluation_id = field_string obj "evaluation_id" in
  let tether_obj = field_object obj "tether" in
  let source = field_string (object_fields tether_obj) "source" in
  let event_obj = field_object obj "event" in
  let event_fields = object_fields event_obj in
  let event_name = field_string event_fields "name" in
  let event_data =
    match List.assoc_opt "data" event_fields with
    | Some v -> v
    | None -> `Null
  in
  let facts =
    match List.assoc_opt "facts" obj with
    | Some (`Assoc pairs) -> pairs
    | _ -> failwith "PF1 stage decode: facts must be an object"
  in
  let cap_jsons = field_list obj "capabilities" in
  let top_level_caps =
    List.map
      (fun cap_json ->
        match cap_json with
        | `Assoc _ -> Tethers_protocol.parse_capability cap_json
        | _ -> failwith "PF1 stage decode: capability must be an object")
      cap_jsons
  in
  let core_fields = object_fields (field_object obj "core_environment") in
  let program_id =
    Tethers_core.program_id_of_string (field_string core_fields "program_id")
  in
  let core_version =
    Tethers_core.core_version_of_string (field_string core_fields "core_version")
  in
  let cap_binding_jsons = field_list core_fields "capabilities" in
  let capabilities =
    List.map
      (fun binding ->
        let b = object_fields binding in
        let source_name = field_string b "source_name" in
        let capability_id =
          Tethers_core.capability_id_of_string (field_string b "capability_id")
        in
        let contract_digest =
          Tethers_core.capability_contract_digest_of_string
            (field_string b "contract_digest")
        in
        let runtime_name = field_string b "runtime_name" in
        let matches =
          List.filter
            (fun (c : Tethers_protocol.capability) -> c.name = runtime_name)
            top_level_caps
        in
        let runtime =
          match matches with
          | [ r ] -> r
          | [] -> failwith ("PF1 stage decode: no runtime capability " ^ runtime_name)
          | _ -> failwith ("PF1 stage decode: ambiguous runtime capability " ^ runtime_name)
        in
        ( {
            Tethers_core_evaluation_adapter.source_name;
            capability_id;
            contract_digest;
            runtime;
          } :
          Tethers_core_evaluation_adapter.capability_binding ))
      cap_binding_jsons
  in
  let fact_binding_jsons =
    match List.assoc_opt "input_facts" core_fields with
    | None | Some `Null -> []
    | Some (`List l) -> l
    | Some _ -> failwith "PF1 stage decode: input_facts must be an array"
  in
  let input_facts =
    List.map
      (fun fj ->
        let f = object_fields fj in
        let source_name = field_string f "source_name" in
        let fact_id = Tethers_core.fact_id_of_string (field_string f "fact_id") in
        let host_key =
          Tethers_core.host_snapshot_key_of_string (field_string f "host_snapshot_key")
        in
        let stype =
          match parse_scalar_type (field_string f "scalar_type") with
          | `String_type -> Tethers_core.String_type
          | `Integer_type -> Tethers_core.Integer_type
          | `Boolean_type -> Tethers_core.Boolean_type
        in
        let schema_description = field_string f "schema_description" in
        ( {
            source_name;
            fact =
              {
                fact_id;
                schema_description;
                provenance = Tethers_core.Evaluation_input (host_key, stype);
              };
          } :
          Tethers_core_evaluation_adapter.input_fact_binding ))
      fact_binding_jsons
  in
  let env : Tethers_core_evaluation_adapter.environment =
    { program_id; core_version; capabilities; input_facts }
  in
  let input : Tethers_core_evaluation_adapter.evaluation_input =
    { evaluation_id; source; event_name; event_data; facts }
  in
  (env, input)

(* Low-symmetry request: each Action carries a distinct literal argument
   value, making every origin signature distinguishable from round 0. *)
let make_distinct_ping_request ~eval_id ~evt_id ~num_actions =
  let tether_source =
    let buf = Buffer.create 4096 in
    Buffer.add_string buf
      "tether \"benchmark ping distinct\"\n\nanchor\n    fixture.start\n\nwhen\ndo\n";
    for i = 1 to num_actions do
      Buffer.add_string buf
        ("    fixture.ping\n        message: \"msg_" ^ string_of_int i
       ^ "\"\n        path: \"projects/bench.txt\"\n")
    done;
    Buffer.contents buf
  in
  let actions_json =
    `List
      [
        `Assoc
          [
            ("name", `String "fixture.ping");
            ("version", `String "1.0.0");
            ("inputs", `Assoc [ ("message", `String "string"); ("path", `String "string") ]);
            ("effects", `List [ `String "fixture.test" ]);
            ("reversibility", `String "compensatable");
          ];
      ]
  in
  `Assoc
    [
      ("protocol_version", `String "0.1");
      ("language_version", `String "0.1");
      ("evaluation_id", `String eval_id);
      ( "tether",
        `Assoc
          [
            ("id", `String "benchmark-ping-distinct");
            ("version", `String "1");
            ("source", `String tether_source);
          ] );
      ( "event",
        `Assoc
          [
            ("id", `String evt_id);
            ("name", `String "fixture.start");
            ("data", `Assoc [ ("message", `String "hello") ]);
          ] );
      ("facts", `Assoc []);
      ("capabilities", actions_json);
      ( "core_environment",
        `Assoc
          [
            ("program_id", `String "program.benchmark");
            ("core_version", `String "1");
            ( "capabilities",
              `List
                [
                  `Assoc
                    [
                      ("source_name", `String "fixture.ping");
                      ("capability_id", `String "cap.benchmark.ping");
                      ("contract_digest", `String "BENCH-CONTRACT-0");
                      ("runtime_name", `String "fixture.ping");
                    ];
                ] );
            ("input_facts", `List []);
          ] );
    ]

(* Verify the staged pipeline produces the same wire status as the full
   wire entry point, so isolated stage timing is not timing a wrong path. *)
let verify_stage_pipeline_equivalent request_json =
  let env, input = decode_stage_inputs request_json in
  let lowerer_env : Tethers_core_lowerer.lowering_environment =
    {
      program_id = env.program_id;
      core_version = env.core_version;
      capabilities =
        List.map
          (fun (b : Tethers_core_evaluation_adapter.capability_binding) ->
            {
              Tethers_core_lowerer.source_name = b.source_name;
              capability_id = b.capability_id;
              contract_digest = b.contract_digest;
            })
          env.capabilities;
      input_facts =
        List.map
          (fun (b : Tethers_core_evaluation_adapter.input_fact_binding) ->
            {
              Tethers_core_lowerer.source_name = b.source_name;
              fact = b.fact;
            })
          env.input_facts;
    }
  in
  let tether = Tether_parser.parse_tether input.source in
  let program = Result.get_ok (Tethers_core_lowerer.lower lowerer_env tether) in
  let canonicalized =
    Result.get_ok (Tethers_core_canonical.canonicalize program)
  in
  let eval_ctx : Tethers_core_plan.evaluation_context =
    {
      evaluation_id = input.evaluation_id;
      event = { Tethers_core_plan.name = input.event_name; data = input.event_data };
      capabilities =
        List.map
          (fun (b : Tethers_core_evaluation_adapter.capability_binding) ->
            {
              Tethers_core_plan.capability_id = b.capability_id;
              contract_digest = b.contract_digest;
              runtime = b.runtime;
            })
          env.capabilities;
      facts = [];
    }
  in
  let staged_status =
    match Tethers_core_plan.evaluate_canonicalized canonicalized eval_ctx with
    | Ok (Tethers_core_plan.Matched _) -> "matched"
    | Ok Tethers_core_plan.Not_matched -> "not_matched"
    | Error _ -> "error"
  in
  let wire = Tethers_core_wire.evaluate_request_json request_json in
  let wire_status =
    match Yojson.Safe.Util.member "status" wire with
    | `String s -> s
    | _ -> "unknown"
  in
  (staged_status, wire_status)

(* Part E + F runner.  Emits a JSON document to stdout for the harness. *)
(*
   Timing technique: each stage is measured as a batch of repeated calls
   through the exact production function, so the coarse Windows
   gettimeofday quantisation observed in per-call readings is amortised
   (the same technique B0-A uses for the whole pipeline).  The pipeline
   intermediates are built once per size; each stage's own function is
   re-run within its batch window.
*)
let time_loop ~num_batches ~batch f =
  let _ = f () in
  let samples = Array.make num_batches 0.0 in
  for b = 0 to num_batches - 1 do
    let t0 = now_seconds () in
    for _ = 1 to batch do
      let _ = f () in
      ()
    done;
    let t1 = now_seconds () in
    samples.(b) <- (t1 -. t0) *. 1_000_000.0 /. float_of_int batch
  done;
  samples

(* Build the typed pipeline intermediates for a request once, so each stage
   can be timed against pre-built inputs. *)
let prepare_size_inputs request_json =
  let env, input = decode_stage_inputs request_json in
  let lowerer_env : Tethers_core_lowerer.lowering_environment =
    {
      program_id = env.program_id;
      core_version = env.core_version;
      capabilities =
        List.map
          (fun (b : Tethers_core_evaluation_adapter.capability_binding) ->
            {
              Tethers_core_lowerer.source_name = b.source_name;
              capability_id = b.capability_id;
              contract_digest = b.contract_digest;
            })
          env.capabilities;
      input_facts =
        List.map
          (fun (b : Tethers_core_evaluation_adapter.input_fact_binding) ->
            {
              Tethers_core_lowerer.source_name = b.source_name;
              fact = b.fact;
            })
          env.input_facts;
    }
  in
  let tether = Tether_parser.parse_tether input.source in
  let program = Result.get_ok (Tethers_core_lowerer.lower lowerer_env tether) in
  let canonicalized =
    Result.get_ok (Tethers_core_canonical.canonicalize program)
  in
  let eval_ctx : Tethers_core_plan.evaluation_context =
    {
      evaluation_id = input.evaluation_id;
      event = { Tethers_core_plan.name = input.event_name; data = input.event_data };
      capabilities =
        List.map
          (fun (b : Tethers_core_evaluation_adapter.capability_binding) ->
            {
              Tethers_core_plan.capability_id = b.capability_id;
              contract_digest = b.contract_digest;
              runtime = b.runtime;
            })
          env.capabilities;
      facts = [];
    }
  in
  (input.source, tether, lowerer_env, program, canonicalized, eval_ctx)

let measure_stages ~num_batches ~batch request_json =
  let source, tether, lowerer_env, program, canonicalized, eval_ctx =
    prepare_size_inputs request_json
  in
  let parse_us =
    time_loop ~num_batches ~batch (fun () ->
        let _ = Tether_parser.parse_tether source in
        ())
  in
  let lower_us =
    time_loop ~num_batches ~batch (fun () ->
        let _ = Tethers_core_lowerer.lower lowerer_env tether in
        ())
  in
  let validate_us =
    time_loop ~num_batches ~batch (fun () ->
        let _ = Tethers_core_validator.validate program in
        ())
  in
  let canonicalize_us =
    time_loop ~num_batches ~batch (fun () ->
        let _ = Tethers_core_canonical.canonicalize program in
        ())
  in
  let rocket_v2_us =
    time_loop ~num_batches ~batch (fun () ->
        let _ = Tethers_core_canonical_v2_ir.canonicalize_ir program in
        ())
  in
  let plan_us =
    time_loop ~num_batches ~batch (fun () ->
        let _ = Tethers_core_plan.evaluate_canonicalized canonicalized eval_ctx in
        ())
  in
  let whole_us =
    time_loop ~num_batches ~batch (fun () ->
        let _ = Tethers_core_wire.evaluate_request_json request_json in
        ())
  in
  (parse_us, lower_us, validate_us, canonicalize_us, rocket_v2_us, plan_us, whole_us)

let run_stage_profile () =
  Printf.printf "PF1: Core stage profile\n%!";
  let sizes = [ 1; 3; 5; 10; 25; 50; 100; 250; 500 ] in
  (* (warmup, batches, batch_size) per size *)
  let plan_for size =
    if size <= 10 then (30, 20, 50)
    else if size <= 25 then (20, 15, 30)
    else if size <= 50 then (10, 10, 20)
    else if size <= 100 then (5, 10, 10)
    else if size <= 250 then (3, 8, 5)
    else (2, 6, 3)
  in
  let size_rows = ref [] in
  List.iter
    (fun size ->
      let warmup, num_batches, batch = plan_for size in
      let request_json =
        make_ping_request ~eval_id:("stg_" ^ string_of_int size)
          ~evt_id:("evt_stg_" ^ string_of_int size) ~num_actions:size
      in
      let staged_status, wire_status =
        verify_stage_pipeline_equivalent request_json
      in
      Printf.printf "  size=%d equivalence: staged=%s wire=%s\n%!" size
        staged_status wire_status;
      for _ = 1 to warmup do
        let _ = prepare_size_inputs request_json in
        ()
      done;
      let parse_us, lower_us, validate_us, canon_us, rocket_us, plan_us, whole_us =
        measure_stages ~num_batches ~batch request_json
      in
      let json_of name samples =
        let stats = compute_stats samples in
        (name, json_of_stats stats)
      in
      size_rows :=
        `Assoc
          [
            ("size", `Int size);
            ("batches", `Int num_batches);
            ("batch_size", `Int batch);
            ("equivalence_staged", `String staged_status);
            ("equivalence_wire", `String wire_status);
            json_of "parse" parse_us;
            json_of "lower" lower_us;
            json_of "validate" validate_us;
            json_of "canonicalize_legacy" canon_us;
            json_of "canonicalize_rocket_v2" rocket_us;
            json_of "plan" plan_us;
            json_of "whole_pipeline" whole_us;
          ]
        :: !size_rows)
    sizes;
  (* Part F: shape probe at 100 and 250 *)
  Printf.printf "PF1: shape probe (Part F)\n%!";
  let shape_probe size =
    let warmup, num_batches, batch = plan_for size in
    let high_req =
      make_ping_request ~eval_id:("shape_hi_" ^ string_of_int size)
        ~evt_id:("evt_hi_" ^ string_of_int size) ~num_actions:size
    in
    let low_req =
      make_distinct_ping_request ~eval_id:("shape_lo_" ^ string_of_int size)
        ~evt_id:("evt_lo_" ^ string_of_int size) ~num_actions:size
    in
    for _ = 1 to warmup do
      let _ = prepare_size_inputs high_req in
      let _ = prepare_size_inputs low_req in
      ()
    done;
    let _, _, _, canon_hi, rocket_hi, _, whole_hi =
      measure_stages ~num_batches ~batch high_req
    in
    let _, _, _, canon_lo, rocket_lo, _, whole_lo =
      measure_stages ~num_batches ~batch low_req
    in
    let stats_of name samples =
      let stats = compute_stats samples in
      (name, json_of_stats stats)
    in
    `Assoc
      [
        ("size", `Int size);
        stats_of "high_symmetry_canonicalize_legacy" canon_hi;
        stats_of "high_symmetry_canonicalize_rocket_v2" rocket_hi;
        stats_of "low_symmetry_canonicalize_legacy" canon_lo;
        stats_of "low_symmetry_canonicalize_rocket_v2" rocket_lo;
        stats_of "high_symmetry_whole" whole_hi;
        stats_of "low_symmetry_whole" whole_lo;
      ]
  in
  let shape_rows = [ shape_probe 100; shape_probe 250 ] in
  let json_output =
    `Assoc
      [
        ("benchmark", `String "PF1-CORE-STAGES");
        ("stages", `List (List.rev !size_rows));
        ("shape_probe", `List shape_rows);
      ]
  in
  Printf.printf "\nPF1 JSON output:\n%s\n%!"
    (Yojson.Safe.pretty_to_string json_output)

(* ================================================================== *)
(*  Main benchmark suite                                               *)
(* ================================================================== *)

type case_spec = string * Yojson.Safe.t * int * int * int
(* (label, request_json, warmup_count, batch_size, num_batches) *)

(* Full B0-A matrix. The historical baseline used these counts. *)
let full_cases : case_spec list =
  [
    ("P0 (not_matched)", make_not_matched_request ~eval_id:"b0" ~evt_id:"eb0", 500, 1000, 50);
    ("P1 (1 action)",
     make_ping_request ~eval_id:"b1" ~evt_id:"eb1" ~num_actions:1, 500, 1000, 50);
    ("P3 (3 actions)",
     make_ping_request ~eval_id:"b3" ~evt_id:"eb3" ~num_actions:3, 500, 500, 50);
    ("P10 (10 actions)",
     make_ping_request ~eval_id:"b10" ~evt_id:"eb10" ~num_actions:10, 200, 100, 50);
    ("P25 (25 actions)",
     make_ping_request ~eval_id:"b25" ~evt_id:"eb25" ~num_actions:25, 100, 20, 50);
    ("P50 (50 actions)",
     make_ping_request ~eval_id:"b50" ~evt_id:"eb50" ~num_actions:50, 50, 10, 50);
    ("PC10 (10 actions + conditions)",
     make_pc10_request ~eval_id:"bpc10" ~evt_id:"ebpc10" ~num_actions:10, 200, 100, 50);
    ("PA10 (10 actions + anchor refs)",
     make_pa10_request ~eval_id:"bpa10" ~evt_id:"ebpa10" ~num_actions:10, 200, 100, 50);
  ]

(* Bounded smoke mode: P1 + P10 with tiny sample counts. Exercises the real
   pipeline end-to-end without the expensive matrix. Quick numbers are
   correctness smoke, NOT baseline performance. *)
let quick_cases : case_spec list =
  [
    ("P1 (1 action)",
     make_ping_request ~eval_id:"q1" ~evt_id:"eq1" ~num_actions:1, 5, 5, 3);
    ("P10 (10 actions)",
     make_ping_request ~eval_id:"q10" ~evt_id:"eq10" ~num_actions:10, 5, 5, 3);
  ]

let run_cases (cases : case_spec list) =
  let results = ref [] in
  List.iter
    (fun (label, req, warmup, batch_size, num_batches) ->
      Printf.printf "Benchmarking %s (warmup=%d batch=%dx%d):\n%!" label warmup batch_size num_batches;
      let (stats, raw) =
        run_benchmark ~label ~request_json:req ~warmup_count:warmup ~batch_size ~num_batches
      in
      results := (label, stats, raw) :: !results)
    cases;
  let ordered = List.rev !results in
  Printf.printf "\n%!";
  (* Summary table *)
  Printf.printf "Summary:\n%!";
  Printf.printf "%-20s %10s %10s %10s %10s %12s\n%!"
    "Case" "Median(us)" "P95(us)" "Mean(us)" "StdDev(us)" "Ops/sec";
  Printf.printf "%-20s %10s %10s %10s %10s %12s\n%!"
    "----" "----------" "--------" "--------" "---------" "-------";
  List.iter
    (fun (label, s, _) ->
      Printf.printf "%-20s %10.1f %10.1f %10.1f %10.1f %12.0f\n%!"
        label s.median_us s.p95_us s.mean_us s.stddev_us s.ops_per_sec)
    ordered;
  Printf.printf "\n%!";
  (* JSON output *)
  let json_output =
    `Assoc
      [
        ("benchmark", `String "B0-A");
        ("description", `String "Core Pipeline Microbenchmark");
        ("cases",
          `List
            (List.map
               (fun (label, s, raw) ->
                 `Assoc
                   [
                     ("case", `String label);
                     ("stats", json_of_stats s);
                     ("raw_us", `List raw);
                   ])
               ordered));
      ]
  in
  let json_str = Yojson.Safe.pretty_to_string json_output in
  Printf.printf "JSON output:\n%s\n%!" json_str

let () =
  let argv = Array.to_list Sys.argv in
  if List.exists (fun a -> a = "--profile-stages") argv then begin
    run_stage_profile ();
    exit 0
  end;
  let quick = List.exists (fun a -> a = "--quick") argv in
  Printf.printf "B0-A: Core Pipeline Microbenchmark%s\n%!"
    (if quick then " (QUICK smoke)" else "");
  Printf.printf "====================================\n%!";
  Printf.printf "\n%!";
  (* Verify ProgramDigest stability *)
  Printf.printf "ProgramDigest sanity:\n%!";
  verify_program_digest_stability ();
  Printf.printf "\n%!";
  (* Verify correctness *)
  Printf.printf "Correctness checks:\n%!";
  verify_correctness ~label:"P1 matched"
    ~request_json:(make_ping_request ~eval_id:"c1" ~evt_id:"e1" ~num_actions:1)
    ~expected_status:"matched";
  verify_correctness ~label:"P0 not_matched"
    ~request_json:(make_not_matched_request ~eval_id:"c2" ~evt_id:"e2")
    ~expected_status:"not_matched";
  Printf.printf "\n%!";
  (* Benchmark cases *)
  run_cases (if quick then quick_cases else full_cases)
