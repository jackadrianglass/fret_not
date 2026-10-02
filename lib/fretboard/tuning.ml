open! Base

type t = Note.t list

let create open_string_notes = open_string_notes
let string_count t = List.length t
let open_note t ~string_index = List.nth_exn t string_index
let open_semitone t ~string_index = Note.semitone (open_note t ~string_index)
let semitone_at t ~string_index ~fret = open_semitone t ~string_index + fret

let pitch_class_at t ~string_index ~fret =
  Pitch_class.of_int (semitone_at t ~string_index ~fret)
;;

let spelled_pitch_of_pitch_class pitch_class =
  Letter.all
  |> List.filter_map ~f:(fun letter ->
      Option.bind (Spelled_pitch.alteration_for ~letter ~pitch_class)
        ~f:(fun alteration -> Some (Spelled_pitch.create ~letter ~alteration)))
  |> List.min_elt ~compare:(fun a b ->
      let alteration_semitones t =
        Alteration.semitones (Spelled_pitch.alteration t)
      in
      match
        Int.compare
          (Int.abs (alteration_semitones a))
          (Int.abs (alteration_semitones b))
      with
      | 0 -> Int.compare (alteration_semitones a) (alteration_semitones b)
      | c -> c)
  |> Option.value_exn
;;

let retune_string t ~string_index ~semitones =
  List.mapi t ~f:(fun i note ->
      if Int.equal i string_index then
        let pitch_class = Pitch_class.add (Note.pitch_class note) semitones in
        Note.create
          ~spelled_pitch:(spelled_pitch_of_pitch_class pitch_class)
          ~octave:
            ((Note.semitone note + semitones
             - Spelled_pitch.semitone (spelled_pitch_of_pitch_class pitch_class)
             )
             / 12
            - 1)
      else note)
;;

let standard =
  [ Note.natural ~letter:Letter.E ~octave:2
  ; Note.natural ~letter:Letter.A ~octave:2
  ; Note.natural ~letter:Letter.D ~octave:3
  ; Note.natural ~letter:Letter.G ~octave:3
  ; Note.natural ~letter:Letter.B ~octave:3
  ; Note.natural ~letter:Letter.E ~octave:4
  ]
;;

let drop_d = retune_string standard ~string_index:0 ~semitones:(-2)
let standard_seven_string = Note.natural ~letter:Letter.B ~octave:1 :: standard

let bass_four =
  List.take standard 4 |> List.map ~f:(fun note -> Note.shift_octave note (-1))
;;
