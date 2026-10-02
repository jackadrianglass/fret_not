open! Base

(** Notes-per-string shapes over an arbitrary degree list. The degree list
    drives everything: the diatonic degrees give the seven 3-notes-per-string
    positions, Pentatonic's degrees give the five 2-notes-per-string positions,
    Arpeggio's the three 1-note-per-string inversions. *)

val scale_notes :
     degrees:Scale_degree.t list
  -> start:int
  -> count:int
  -> Degree_reference.t list
(** The cyclic degree sequence starting [start] notes into the cycle, with the
    octave carrying once per cycle. Empty degrees give no notes. *)

val positions :
     instrument:Instrument.t
  -> key:Key.t
  -> mode:Mode.t
  -> degrees:Scale_degree.t list
  -> notes_per_string:int
  -> anchor:Fretboard_position.t
  -> Fretboard_position.t list list
(** One shape per degree: shape k starts [k] scale notes above the anchor's key
    tonic occurrence, laid out [notes_per_string] notes per string ascending.
    Each shape is dropped to its lowest playable octave; shapes that still leave
    unplayable frets are omitted. *)
