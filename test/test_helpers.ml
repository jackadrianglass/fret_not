open! Base
open Fret_not

let names degrees =
  List.map degrees ~f:(fun (sd : Scale_degree.t) ->
      Pitch_class.to_string sd.pitch_class)
;;
