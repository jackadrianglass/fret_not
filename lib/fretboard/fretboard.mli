open! Base

val to_positions :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> Degree_reference.t
  -> Fretboard_position.t list
(** anchor_position defines what "octave 0" means for this call: the nearest
    occurrence of the target pitch class to that position, ties broken upward.
    The same Degree_reference resolves to different Fretboard Positions
    depending on the anchor — that's what lets an abstract pattern be projected
    anywhere on the neck without editing its own data. Returns one position per
    string that can reach the target (each string reaches a given absolute pitch
    at exactly one fret, or not at all), sorted by fret-distance from
    anchor_position. *)

val positions_in_window :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> min_fret:int
  -> max_fret:int
  -> Fretboard_position.t list
(** Every position of every degree in the mode that falls within
    [min_fret, max_fret], anchored the same way as to_positions. The octave
    range searched per degree is computed from the window and tuning, not
    guessed - unlike calling to_positions once per (degree, octave) and
    filtering by hand, this can't silently miss positions on a wide neck or an
    unusual tuning. Sorted by (string_index, fret). *)

val pentatonic_degrees : key:Key.t -> Scale_degree.t list
(** This key's own pentatonic scale degrees (major pentatonic for a Major key,
    minor pentatonic for a Minor one) - see Pentatonic. Degree numbers are the
    original 1-7 from the full diatonic scale, just filtered down to 5, not
    renumbered. *)

val pentatonic_positions_in_window :
     key:Key.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> min_fret:int
  -> max_fret:int
  -> Fretboard_position.t list
(** Same contract as positions_in_window, restricted to this key's own
    pentatonic scale (major pentatonic for a Major key, minor pentatonic for a
    Minor one - see Pentatonic) instead of all 7 diatonic degrees. No ~mode
    parameter: pentatonic-major/minor aren't independently selectable the way
    the 7 diatonic modes are. *)

val three_notes_per_string_positions :
  key:Key.t -> mode:Mode.t -> tuning:Tuning.t -> Fretboard_position.t list list
(** The 7 "3-notes-per-string" mode positions for this key, one flat list of
    Fretboard_position.t per position (string_index ascending, 3 ascending
    diatonic frets per string). Position 1 starts at the nearest occurrence of
    [mode]'s own root to the low string's open position; each later position
    starts one scale-degree past the previous position's own low-string start
    (not past its last note), so adjacent positions share 2 of their 3
    low-string notes and position N lands on the Nth mode in Key.modes order
    (e.g. position 2 is Dorian relative to a major key). Climbing the neck one
    scale-degree at a time would otherwise walk positions further and further up
    with no way back down, so each position is dropped by whole octaves (pitch
    class is unaffected by octave, so this doesn't change which mode it is) to
    sit at its own lowest playable spot - unlike to_positions/
    positions_in_window, not unbounded. *)

val two_notes_per_string_positions :
  key:Key.t -> tuning:Tuning.t -> Fretboard_position.t list list
(** The 5 pentatonic "2-notes-per-string" positions for this key's own
    pentatonic scale (major pentatonic for a Major key, minor pentatonic for a
    Minor one), one flat list of Fretboard_position.t per position - otherwise
    the same shape and chaining/octave-drop contract as
    three_notes_per_string_positions, sized for 2 notes/string and 5 positions
    (one octave of the 5-note pentatonic scale) instead of 3 and 7. No ~mode
    parameter, for the same reason as pentatonic_positions_in_window. *)

val arpeggio_degrees : key:Key.t -> Scale_degree.t list
(** This key's own triad arpeggio degrees (major arpeggio for a Major key, minor
    arpeggio for a Minor one) - see Arpeggio. Degree numbers are the original
    1/3/5 from the full diatonic scale, not renumbered. *)

val arpeggio_positions_in_window :
     key:Key.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> min_fret:int
  -> max_fret:int
  -> Fretboard_position.t list
(** Same contract as positions_in_window, restricted to this key's own triad
    arpeggio (see arpeggio_degrees) instead of all 7 diatonic degrees. No ~mode
    parameter, for the same reason as pentatonic_positions_in_window. *)

val one_note_per_string_positions :
  key:Key.t -> tuning:Tuning.t -> Fretboard_position.t list list
(** The 3 arpeggio positions (Root, 1st Inversion, 2nd Inversion) for this key's
    own triad, one flat list of Fretboard_position.t per position: 1 note per
    string, plus one extra closing note on the high string that continues the
    same ascending walk one more step, so the position starts and ends on the
    same tone. Otherwise the same chaining/octave-drop contract as
    three_notes_per_string_positions and two_notes_per_string_positions, sized
    for 1 note/string and 3 positions (one octave of the 3-note arpeggio)
    instead of 3/7 or 2/5. No ~mode parameter, for the same reason as
    arpeggio_positions_in_window. *)

val of_position :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> Fretboard_position.t
  -> Degree_reference.t
(** Inverse of to_positions, anchored the same way — round-tripping (degree ->
    position -> degree) via the same anchor_position returns the original
    degree, for the diatonic case (alteration = 0). Raises if the position's
    pitch class isn't one of the mode's 7 diatonic degrees; chromatic positions
    aren't representable as a Degree_reference yet. *)
