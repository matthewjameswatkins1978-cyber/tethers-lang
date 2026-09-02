
type nat =
| O
| S of nat

module Stage0 :
 sig
  val successor : nat -> nat
 end
