open! Base

val x_positions : start_x:float -> gap:float -> float list -> float list
(** Left-justified: positions start at [start_x], stepping right by each width
    plus [gap]. *)

val x_positions_from_right :
  end_x:float -> gap:float -> float list -> float list
(** Right-justified: the last width ends flush at [end_x]; returned positions
    keep the widths' own left-to-right order. *)

val dropdown_width :
     left_text_padding:int
  -> widest_option_text_width:int
  -> arrow_padding:int
  -> text_to_arrow_gap:int
  -> float
