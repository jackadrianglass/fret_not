open! Base

(** Degree-subset presets: which degrees of the key a scale keeps. The same
    lists drive highlighting, shape generation, and the scale dropdown. *)

val pentatonic_major : Scale_degree.t list
(** Degrees 1 2 3 5 6. *)

val pentatonic_minor : Scale_degree.t list
(** Degrees 1 3 4 5 7. *)

val arpeggio_major : Scale_degree.t list
(** The triad: degrees 1 3 5. *)

val arpeggio_minor : Scale_degree.t list
(** The same 1 3 5 — scale-relative degrees already spell the minor triad in a
    minor key. *)
