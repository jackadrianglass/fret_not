let () =
  Alcotest.run "fret_not"
    [ ("pitch_class", Pitch_class_test.tests)
    ; ("alteration", Alteration_test.tests)
    ; ("scale_degree", Scale_degree_test.tests)
    ; ("mode", Mode_test.tests)
    ; ("pentatonic", Pentatonic_test.tests)
    ; ("arpeggio", Arpeggio_test.tests)
    ; ("key", Key_test.tests)
    ; ("tuning", Tuning_test.tests)
    ; ("fretboard", Fretboard_test.tests)
    ; ("fretboard_layout", Fretboard_layout_test.tests)
    ; ("tab_layout", Tab_layout_test.tests)
    ; ("row_layout", Row_layout_test.tests)
    ; ("fretboard_view_state", Fretboard_view_state_test.tests)
    ; ("slot", Slot_test.tests)
    ; ("chunk", Chunk_test.tests)
    ; ("chord_progression", Chord_progression_test.tests)
    ; ("chunk_solver", Chunk_solver_test.tests)
    ]
;;
