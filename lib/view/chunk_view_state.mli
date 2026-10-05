open! Base

type t =
  { chunk : Chunk.t
  ; cursor_index : int
  ; degree_dropdown_open : bool
  }
[@@deriving eq]

val initial : t
(** Four slots — a tonic note then three rests — with the cursor on the tonic.
*)

val chunk : t -> Chunk.t

val selected_slot : t -> Chunk.slot option
(** [None] when the chunk is empty. *)

val slot_label : Chunk.slot -> string
(** A degree label ("1", "b3") with a trailing apostrophe per octave above the
    tonic, or "." for a rest. *)

val slot_labels : t -> string list

val select_slot : t -> slot_index:int -> t
(** Clamps into [0, length - 1] (0 when the chunk is empty). *)

val set_selected_rest : t -> rest:bool -> t
(** [rest:true] turns the selected slot into Rest. [rest:false] turns a Rest
    into a natural degree-1 octave-0 note; a slot that already holds a note
    keeps it. *)

val set_selected_degree : t -> degree:int -> t
val set_selected_alteration : t -> alteration:Alteration.t -> t

val set_selected_octave : t -> octave:int -> t
(** Each converts a Rest slot into a Note holding the edited value (rest of the
    reference at its defaults) rather than no-opping. All are no-ops on an empty
    chunk. *)

val add_slot : t -> slot:Chunk.slot -> t
(** Appends at the end and selects the new slot. *)

val remove_selected : t -> t
(** Removes the selected slot; the cursor moves to the slot left of it, or 0 at
    the left edge. No-op on an empty chunk. *)
