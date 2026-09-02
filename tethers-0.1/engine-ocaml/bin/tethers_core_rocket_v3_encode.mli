(** Exact Enc_V2 emission from a discrete, stable Rocket V3 leaf. *)

type leaf = {
  labels : Tethers_core_canonical_v2_format.label_assignment;
  payload : string;
  preimage : bytes;
  digest : string;
}

type error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Model_mismatch
  | Partition_not_stable
  | Partition_not_discrete
  | Missing_vertex of string
  | No_legal_label_assignment of string

val encode :
  Tethers_core.program ->
  Tethers_core_rocket_v3_model.t ->
  Tethers_core_rocket_v3_partition.t ->
  (leaf, error) result
(** The ordering certificate is the exact minimum over the complete legal V2
    label domain.  Every candidate is emitted by the existing frozen
    [encode_program] and compared with unsigned-byte lexicographic order.
    Stable partition cells are used only to establish the required discrete
    leaf precondition; they are not numeric-label authority. *)
