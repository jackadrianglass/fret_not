(lang dune 3.23)

(name fret_not)

(generate_opam_files true)

(authors "Jack Glass <glassjack@proton.me>")

(package
 (name fret_not)
 (synopsis "A fret board visualization tool")
 (description "A quick tool to help come up with exercises for navigating the fret board")
 (depends
   (ocaml (= 5.4.1))
   base
   raylib
   raygui
   (alcotest :with-test)
   ocamlformat
   ))

; See the complete stanza docs at https://dune.readthedocs.io/en/stable/reference/dune-project/index.html
