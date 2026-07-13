Require Import Coq.Arith.Arith.

(* A small binary tree over natural numbers. *)
Inductive tree : Type :=
  | Leaf
  | Node (l : tree) (value : nat) (r : tree).

Fixpoint size (t : tree) : nat :=
  match t with
  | Leaf => 0
  | Node l _ r => 1 + size l + size r
  end.

Theorem size_leaf : size Leaf = 0.
Proof.
  simpl. reflexivity.
Qed.
