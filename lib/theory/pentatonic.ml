open! Base

let keep degrees ~allowed =
  List.filter degrees ~f:(fun (d : Scale_degree.t) ->
      List.mem allowed d.degree ~equal:Int.equal)
;;

let major ~root = keep (Mode.degrees Ionian ~root) ~allowed:[ 1; 2; 3; 5; 6 ]
let minor ~root = keep (Mode.degrees Aeolian ~root) ~allowed:[ 1; 3; 4; 5; 7 ]
