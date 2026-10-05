open! Base
open Fret_not

let note ~degree ~alteration ~octave =
  Chunk.Note
    (Degree_reference.create
       ~scale_degree:(Scale_degree.create ~degree ~alteration)
       ~octave)
;;

let initial_labels_are_degree_then_rests () =
  Alcotest.(check (list string))
    "labels" [ "1"; "."; "."; "." ]
    (Chunk_view_state.slot_labels Chunk_view_state.initial);
  Alcotest.(check int)
    "cursor on the tonic" 0 Chunk_view_state.initial.cursor_index
;;

let select_slot_clamps_into_the_chunk () =
  let state = Chunk_view_state.initial in
  Alcotest.(check int)
    "in range" 2 (Chunk_view_state.select_slot state ~slot_index:2).cursor_index;
  Alcotest.(check int)
    "past the end clamps to the last slot" 3
    (Chunk_view_state.select_slot state ~slot_index:99).cursor_index;
  Alcotest.(check int)
    "negative clamps to the first slot" 0
    (Chunk_view_state.select_slot state ~slot_index:(-5)).cursor_index
;;

let rest_toggle_round_trips_through_a_default_note () =
  let as_rest =
    Chunk_view_state.set_selected_rest Chunk_view_state.initial ~rest:true
  in
  Alcotest.(check bool)
    "tonic becomes a rest" true
    (Chunk.equal
       [ Chunk.Rest; Chunk.Rest; Chunk.Rest; Chunk.Rest ]
       (Chunk_view_state.chunk as_rest));
  let as_note = Chunk_view_state.set_selected_rest as_rest ~rest:false in
  Alcotest.(check bool)
    "rest becomes the default tonic note" true
    (Chunk.equal
       [ note ~degree:1 ~alteration:Alteration.Natural ~octave:0
       ; Chunk.Rest
       ; Chunk.Rest
       ; Chunk.Rest
       ]
       (Chunk_view_state.chunk as_note))
;;

let set_selected_rest_on_a_note_keeps_it () =
  let state = { Chunk_view_state.initial with cursor_index = 1 } in
  let state = Chunk_view_state.set_selected_degree state ~degree:5 in
  let state = Chunk_view_state.set_selected_rest state ~rest:false in
  Alcotest.(check bool)
    "the degree-5 note survives the no-op toggle" true
    (Chunk.equal
       [ Chunk.Note (Degree_reference.natural ~degree:1 ~octave:0)
       ; note ~degree:5 ~alteration:Alteration.Natural ~octave:0
       ; Chunk.Rest
       ; Chunk.Rest
       ]
       (Chunk_view_state.chunk state))
;;

let editing_a_rest_converts_it_into_that_note () =
  let state = { Chunk_view_state.initial with cursor_index = 1 } in
  Alcotest.(check bool)
    "degree edit converts the rest" true
    (Chunk.equal
       [ Chunk.Note (Degree_reference.natural ~degree:1 ~octave:0)
       ; note ~degree:3 ~alteration:Alteration.Natural ~octave:0
       ; Chunk.Rest
       ; Chunk.Rest
       ]
       (Chunk_view_state.chunk
          (Chunk_view_state.set_selected_degree state ~degree:3)));
  Alcotest.(check bool)
    "alteration edit converts the rest" true
    (Chunk.equal
       [ Chunk.Note (Degree_reference.natural ~degree:1 ~octave:0)
       ; note ~degree:1 ~alteration:Alteration.Flat ~octave:0
       ; Chunk.Rest
       ; Chunk.Rest
       ]
       (Chunk_view_state.chunk
          (Chunk_view_state.set_selected_alteration state
             ~alteration:Alteration.Flat)));
  Alcotest.(check bool)
    "octave edit converts the rest" true
    (Chunk.equal
       [ Chunk.Note (Degree_reference.natural ~degree:1 ~octave:0)
       ; note ~degree:1 ~alteration:Alteration.Natural ~octave:1
       ; Chunk.Rest
       ; Chunk.Rest
       ]
       (Chunk_view_state.chunk
          (Chunk_view_state.set_selected_octave state ~octave:1)))
;;

let editing_a_note_keeps_the_rest_of_the_reference () =
  let state = { Chunk_view_state.initial with cursor_index = 0 } in
  let state = Chunk_view_state.set_selected_octave state ~octave:1 in
  let state =
    Chunk_view_state.set_selected_alteration state ~alteration:Alteration.Sharp
  in
  let state = Chunk_view_state.set_selected_degree state ~degree:4 in
  Alcotest.(check bool)
    "only the edited parts change" true
    (Chunk.equal
       [ note ~degree:4 ~alteration:Alteration.Sharp ~octave:1
       ; Chunk.Rest
       ; Chunk.Rest
       ; Chunk.Rest
       ]
       (Chunk_view_state.chunk state));
  Alcotest.(check (list string))
    "label carries the octave apostrophe" [ "#4'"; "."; "."; "." ]
    (Chunk_view_state.slot_labels state)
;;

let add_slot_appends_and_selects_it () =
  let state =
    Chunk_view_state.add_slot Chunk_view_state.initial
      ~slot:(note ~degree:2 ~alteration:Alteration.Natural ~octave:0)
  in
  Alcotest.(check bool)
    "the slot is appended" true
    (Chunk.equal
       (Chunk_view_state.chunk Chunk_view_state.initial
       @ [ note ~degree:2 ~alteration:Alteration.Natural ~octave:0 ])
       (Chunk_view_state.chunk state));
  Alcotest.(check int) "the cursor selects the new slot" 4 state.cursor_index
;;

let remove_selected_moves_the_cursor_left () =
  let state = { Chunk_view_state.initial with cursor_index = 2 } in
  let state = Chunk_view_state.remove_selected state in
  Alcotest.(check bool)
    "the selected slot is gone" true
    (Chunk.equal
       [ Chunk.Note (Degree_reference.natural ~degree:1 ~octave:0)
       ; Chunk.Rest
       ; Chunk.Rest
       ]
       (Chunk_view_state.chunk state));
  Alcotest.(check int) "cursor moves left" 1 state.cursor_index
;;

let removing_at_the_left_edge_keeps_the_cursor_at_zero () =
  let state = Chunk_view_state.remove_selected Chunk_view_state.initial in
  Alcotest.(check int) "cursor stays at 0" 0 state.cursor_index;
  let emptied = { state with chunk = [ Chunk.Rest ] } in
  let emptied = Chunk_view_state.remove_selected emptied in
  Alcotest.(check bool)
    "the last slot can be removed" true
    (Chunk.equal [] (Chunk_view_state.chunk emptied));
  Alcotest.(check bool)
    "removing from an empty chunk is a no-op" true
    (Chunk_view_state.equal emptied (Chunk_view_state.remove_selected emptied));
  Alcotest.(check bool)
    "editing an empty chunk is a no-op" true
    (Chunk_view_state.equal emptied
       (Chunk_view_state.set_selected_degree emptied ~degree:3))
;;

let tests =
  [ Alcotest.test_case "initial labels are degree then rests" `Quick
      initial_labels_are_degree_then_rests
  ; Alcotest.test_case "select_slot clamps into the chunk" `Quick
      select_slot_clamps_into_the_chunk
  ; Alcotest.test_case "rest toggle round-trips through a default note" `Quick
      rest_toggle_round_trips_through_a_default_note
  ; Alcotest.test_case "set_selected_rest on a note keeps it" `Quick
      set_selected_rest_on_a_note_keeps_it
  ; Alcotest.test_case "editing a rest converts it into that note" `Quick
      editing_a_rest_converts_it_into_that_note
  ; Alcotest.test_case "editing a note keeps the rest of the reference" `Quick
      editing_a_note_keeps_the_rest_of_the_reference
  ; Alcotest.test_case "add_slot appends and selects it" `Quick
      add_slot_appends_and_selects_it
  ; Alcotest.test_case "remove_selected moves the cursor left" `Quick
      remove_selected_moves_the_cursor_left
  ; Alcotest.test_case "removing at the left edge keeps the cursor at zero"
      `Quick removing_at_the_left_edge_keeps_the_cursor_at_zero
  ]
;;
