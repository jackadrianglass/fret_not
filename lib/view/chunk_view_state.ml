open! Base

type t =
  { chunk : Chunk.t
  ; cursor_index : int
  ; degree_dropdown_open : bool
  }
[@@deriving eq]

let default_note = Degree_reference.natural ~degree:1 ~octave:0

let initial =
  { chunk = [ Chunk.Note default_note; Chunk.Rest; Chunk.Rest; Chunk.Rest ]
  ; cursor_index = 0
  ; degree_dropdown_open = false
  }
;;

let chunk t = t.chunk

let clamp_index t index =
  let slot_count = List.length t.chunk in
  if slot_count = 0 then 0 else Int.min (Int.max index 0) (slot_count - 1)
;;

let selected_slot t = List.nth t.chunk t.cursor_index

let slot_label = function
  | Chunk.Rest -> "."
  | Chunk.Note (dr : Degree_reference.t) ->
      let base = Scale_degree.label (Degree_reference.scale_degree dr) in
      if Degree_reference.octave dr > 0 then base ^ "'" else base
;;

let slot_labels t = List.map t.chunk ~f:slot_label

let select_slot t ~slot_index =
  { t with cursor_index = clamp_index t slot_index }
;;

let edit_selected t ~(f : Chunk.slot -> Chunk.slot) =
  if t.cursor_index >= List.length t.chunk then t
  else
    let chunk =
      List.mapi t.chunk ~f:(fun index slot ->
          if Int.equal index t.cursor_index then f slot else slot)
    in
    { t with chunk }
;;

let set_selected_rest t ~rest =
  edit_selected t ~f:(fun slot ->
      if rest then Chunk.Rest
      else
        match slot with
        | Chunk.Rest -> Chunk.Note default_note
        | Chunk.Note _ -> slot)
;;

let set_selected_degree t ~degree =
  edit_selected t ~f:(function
    | Chunk.Rest -> Chunk.Note (Degree_reference.natural ~degree ~octave:0)
    | Chunk.Note (dr : Degree_reference.t) ->
        Chunk.Note
          (Degree_reference.create
             ~scale_degree:
               (Scale_degree.create ~degree
                  ~alteration:(Degree_reference.alteration dr))
             ~octave:(Degree_reference.octave dr)))
;;

let set_selected_alteration t ~alteration =
  edit_selected t ~f:(function
    | Chunk.Rest ->
        Chunk.Note
          (Degree_reference.create
             ~scale_degree:(Scale_degree.create ~degree:1 ~alteration)
             ~octave:0)
    | Chunk.Note (dr : Degree_reference.t) ->
        Chunk.Note
          (Degree_reference.create
             ~scale_degree:
               (Scale_degree.create
                  ~degree:(Degree_reference.degree dr)
                  ~alteration)
             ~octave:(Degree_reference.octave dr)))
;;

let set_selected_octave t ~octave =
  edit_selected t ~f:(function
    | Chunk.Rest -> Chunk.Note (Degree_reference.natural ~degree:1 ~octave)
    | Chunk.Note (dr : Degree_reference.t) ->
        Chunk.Note
          (Degree_reference.create
             ~scale_degree:(Degree_reference.scale_degree dr)
             ~octave))
;;

let add_slot t ~slot =
  { t with chunk = t.chunk @ [ slot ]; cursor_index = List.length t.chunk }
;;

let remove_selected t =
  if List.is_empty t.chunk then t
  else
    let removed = t.cursor_index in
    let chunk =
      List.filteri t.chunk ~f:(fun index _ -> not (Int.equal index removed))
    in
    { t with chunk; cursor_index = Int.max 0 (removed - 1) }
;;
