let () =
  Alcotest.run "fret_not"
    [ ("pitch_class", Pitch_class_test.tests)
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
    ]
;;
