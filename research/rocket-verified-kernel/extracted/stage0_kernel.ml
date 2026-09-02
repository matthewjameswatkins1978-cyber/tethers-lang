
type nat =
| O
| S of nat

module Stage0 =
 struct
  (** val successor : nat -> nat **)

  let successor n =
    S n
 end
