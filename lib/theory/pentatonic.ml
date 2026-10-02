open! Base

let keep ~allowed =
  List.filter Scale_degree.diatonic ~f:(fun (d : Scale_degree.t) ->
      List.mem allowed d.degree ~equal:Int.equal)
;;

let major = keep ~allowed:[ 1; 2; 3; 5; 6 ]
let minor = keep ~allowed:[ 1; 3; 4; 5; 7 ]
