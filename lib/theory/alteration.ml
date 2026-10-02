open! Base

type t =
  | Natural
  | Sharp
  | Flat
  | Double_sharp
  | Double_flat

let semitones = function
  | Natural -> 0
  | Sharp -> 1
  | Flat -> -1
  | Double_sharp -> 2
  | Double_flat -> -2
;;

let to_string = function
  | Natural -> ""
  | Sharp -> "#"
  | Flat -> "b"
  | Double_sharp -> "##"
  | Double_flat -> "bb"
;;

let equal a b =
  match (a, b) with
  | Natural, Natural
  | Sharp, Sharp
  | Flat, Flat
  | Double_sharp, Double_sharp
  | Double_flat, Double_flat ->
      true
  | _ -> false
;;

let compare a b = Int.compare (semitones a) (semitones b)
