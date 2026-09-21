open! Base

val x_positions : start_x:float -> gap:float -> float list -> float list
(** Left-to-right x positions for a row of controls given their widths, each
    separated by gap, starting at start_x. One position per input width. *)

val dropdown_width :
     left_text_padding:int
  -> widest_option_text_width:int
  -> arrow_padding:int
  -> text_to_arrow_gap:int
  -> float
(** A dropdown wide enough for its widest option's text (left-aligned, so only a
    small left inset is needed) plus the arrow glyph anchored arrow_padding in
    from the right edge, with a visible gap between the two so they don't
    collide. *)
