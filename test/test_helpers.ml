open! Base
open Fret_not

let names pitch_classes = List.map pitch_classes ~f:Pitch_class.to_string

let degree_names ~root ~mode degrees =
  List.map degrees ~f:(fun (sd : Scale_degree.t) ->
      Pitch_class.to_string (Scale_degree.pitch_class ~root ~mode sd))
;;
