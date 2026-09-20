let () =
  Alcotest.run "fret_not"
    [ ("pitch_class", Pitch_class_test.tests)
    ; ("mode", Mode_test.tests)
    ; ("pentatonic", Pentatonic_test.tests)
    ; ("key", Key_test.tests)
    ; ("tuning", Tuning_test.tests)
    ; ("fretboard", Fretboard_test.tests)
    ; ("fretboard_layout", Fretboard_layout_test.tests)
    ]
;;
