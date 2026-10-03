open! Base

type t =
  | C
  | D
  | E
  | F
  | G
  | A
  | B
[@@deriving eq, ord]

let all = [ C; D; E; F; G; A; B ]

let index = function
  | C -> 0
  | D -> 1
  | E -> 2
  | F -> 3
  | G -> 4
  | A -> 5
  | B -> 6
;;

let of_index i = List.nth_exn all (((i % 7) + 7) % 7)
let offset t n = of_index (index t + n)

let to_pitch_class = function
  | C -> Pitch_class.c
  | D -> Pitch_class.d
  | E -> Pitch_class.e
  | F -> Pitch_class.f
  | G -> Pitch_class.g
  | A -> Pitch_class.a
  | B -> Pitch_class.b
;;

let to_string = function
  | C -> "C"
  | D -> "D"
  | E -> "E"
  | F -> "F"
  | G -> "G"
  | A -> "A"
  | B -> "B"
;;
