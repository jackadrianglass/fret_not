open! Base

let keep ~allowed =
  List.filter Scale_degree.diatonic ~f:(fun (d : Scale_degree.t) ->
      List.mem allowed d.degree ~equal:Int.equal)
;;

let pentatonic_major = keep ~allowed:[ 1; 2; 3; 5; 6 ]
let pentatonic_minor = keep ~allowed:[ 1; 3; 4; 5; 7 ]
let arpeggio_major = keep ~allowed:[ 1; 3; 5 ]

(* Same degrees as the major triad: scale-relative degrees already spell the
   minor triad in a minor key. *)
let arpeggio_minor = arpeggio_major
