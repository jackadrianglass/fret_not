open! Base

type t =
  { fret_span : int
  ; string_span : int
  }
[@@deriving ord]

(** Finger-stretch geometry between positions. The fret span is the physical
    stretch along the neck; the string span is the spread across strings. The
    sounding span between two positions also depends on the tuning's
    cross-string intervals, not just the fret delta. The derived compare orders
    by fret span first — it dominates the physical difficulty of a stretch —
    then by string span. *)

val between : Fretboard_position.t -> Fretboard_position.t -> t

val of_positions : Fretboard_position.t list -> t option
(** The bounding box of a whole shape; None for no positions. *)

val semitone_span :
  instrument:Instrument.t -> Fretboard_position.t -> Fretboard_position.t -> int
