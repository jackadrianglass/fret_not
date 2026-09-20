open! Base

type t = int

let of_int n = n % 12
let to_int t = t
let add t semitones = (t + semitones) % 12

let names =
  [| "C"; "C#"; "D"; "D#"; "E"; "F"; "F#"; "G"; "G#"; "A"; "A#"; "B" |]
;;

let to_string t = names.(t)
let equal = Int.equal
let compare = Int.compare
let c = of_int 0
let c_sharp = of_int 1
let d = of_int 2
let d_sharp = of_int 3
let e = of_int 4
let f = of_int 5
let f_sharp = of_int 6
let g = of_int 7
let g_sharp = of_int 8
let a = of_int 9
let a_sharp = of_int 10
let b = of_int 11
