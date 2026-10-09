open! Base

let x_positions ~start_x ~gap widths =
  List.folding_map widths ~init:start_x ~f:(fun x width ->
      (x +. width +. gap, x))
;;

let x_positions_from_right ~end_x ~gap widths =
  (* Right-justified: processed right to left, so the last width ends flush
     at [end_x] and the returned positions stay in the original left-to-right
     order. *)
  List.fold_left (List.rev widths) ~init:(end_x, [])
    ~f:(fun (right, acc) width ->
      (right -. width -. gap, (right -. width) :: acc))
  |> snd
;;

let dropdown_width ~left_text_padding ~widest_option_text_width ~arrow_padding
    ~text_to_arrow_gap =
  Float.of_int
    (left_text_padding + widest_option_text_width + arrow_padding
   + text_to_arrow_gap)
;;
