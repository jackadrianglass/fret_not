open! Base

val target_pitch_class :
  key:Key.t -> mode:Mode.t -> Degree_reference.t -> Pitch_class.t

val to_positions :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> Degree_reference.t
  -> Fretboard_position.t list

val positions_in_window :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> min_fret:int
  -> max_fret:int
  -> Fretboard_position.t list

val pentatonic_degrees : key:Key.t -> Scale_degree.t list

val pentatonic_positions_in_window :
     key:Key.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> min_fret:int
  -> max_fret:int
  -> Fretboard_position.t list

val three_notes_per_string_positions :
  key:Key.t -> mode:Mode.t -> tuning:Tuning.t -> Fretboard_position.t list list

val two_notes_per_string_positions :
  key:Key.t -> tuning:Tuning.t -> Fretboard_position.t list list

val arpeggio_degrees : key:Key.t -> Scale_degree.t list

val arpeggio_positions_in_window :
     key:Key.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> min_fret:int
  -> max_fret:int
  -> Fretboard_position.t list

val one_note_per_string_positions :
  key:Key.t -> tuning:Tuning.t -> Fretboard_position.t list list

val of_position :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> Fretboard_position.t
  -> Degree_reference.t
