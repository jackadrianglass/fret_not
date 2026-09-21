open! Base

let x_positions ~start_x ~gap widths =
  List.folding_map widths ~init:start_x ~f:(fun x width ->
      (x +. width +. gap, x))
;;

let dropdown_width ~left_text_padding ~widest_option_text_width ~arrow_padding
    ~text_to_arrow_gap =
  Float.of_int
    (left_text_padding + widest_option_text_width + arrow_padding
   + text_to_arrow_gap)
;;
