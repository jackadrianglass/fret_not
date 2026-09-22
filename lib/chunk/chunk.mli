open! Base

type t = Slot.t list

val equal : t -> t -> bool

val reframe : t -> root_degree:int -> t
(** Reinterprets every Note slot's own degree number relative to root_degree as
    the new "1" - the same rotate-by-N-degrees move Mode.degrees makes when
    picking which pitch class counts as a mode's own root, generalized to an
    arbitrary scale-degree anchor and wrapped mod 7. A degree that wraps past 7
    carries into the slot's own octave, so the reframed slot still resolves to
    the correct physical octave once projected through Fretboard. Rest slots
    pass through unchanged. *)
