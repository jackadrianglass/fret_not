open! Base

type t =
  | Natural
  | Sharp
  | Flat
  | Double_sharp
  | Double_flat
[@@deriving eq]

let semitones = function
  | Natural -> 0
  | Sharp -> 1
  | Flat -> -1
  | Double_sharp -> 2
  | Double_flat -> -2
;;

let of_semitones = function
  | 0 -> Some Natural
  | 1 -> Some Sharp
  | -1 -> Some Flat
  | 2 -> Some Double_sharp
  | -2 -> Some Double_flat
  | _ -> None
;;

let to_string = function
  | Natural -> ""
  | Sharp -> "#"
  | Flat -> "b"
  | Double_sharp -> "##"
  | Double_flat -> "bb"
;;

(* Not derived: declaration order is not semitone order. *)
let compare a b = Int.compare (semitones a) (semitones b)
