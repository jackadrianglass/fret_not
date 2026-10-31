use absolute::alteration::Alteration;
use absolute::letter::Letter;
use absolute::note::Note;
use absolute::spelled_pitch::SpelledPitch;
use fretboard::chunk_solver;
use fretboard::fingering::Fingering;
use fretboard::fretboard::{degree_at, degree_note, degrees_in_window, spelled_degree_at};
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::tuning;
use relative::chunk::{self, Slot, Word};
use relative::degree_reference::DegreeReference;
use relative::key::{Key, Quality};
use relative::mode::Mode;
use relative::scale_degree::ScaleDegree;

pub const PROGRESSION: [i32; 3] = [1, 4, 5];
pub const PROGRESSION_LABELS: [&str; 3] = ["I", "IV", "V"];
pub const MAX_SLOTS: usize = 12;

// The solver's exploration window: fingerings must stay within the fret cap
// of the anchor. The anchor stays fixed; the cap is a setting.
pub const SOLVER_ANCHOR: FretboardPosition = FretboardPosition {
    string_index: 0,
    fret: 5,
};

// Text sizes across the app are design sizes at this base; the font-size
// setting scales them off it.
pub const BASE_FONT_SIZE: f32 = 16.0;

pub fn instrument() -> Instrument {
    InstrumentPreset::Standard.instrument()
}

// The known tunings, each with the fret count its instrument ships with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentPreset {
    Standard,
    DropD,
    SevenString,
    BassFour,
}

impl InstrumentPreset {
    pub const ALL: [InstrumentPreset; 4] = [
        InstrumentPreset::Standard,
        InstrumentPreset::DropD,
        InstrumentPreset::SevenString,
        InstrumentPreset::BassFour,
    ];

    pub fn instrument(&self) -> Instrument {
        match self {
            InstrumentPreset::Standard => Instrument::new_uniform(tuning::standard(), 22),
            InstrumentPreset::DropD => Instrument::new_uniform(tuning::drop_d(), 22),
            InstrumentPreset::SevenString => {
                Instrument::new_uniform(tuning::standard_seven_string(), 24)
            }
            InstrumentPreset::BassFour => Instrument::new_uniform(tuning::bass_four(), 20),
        }
    }

    // The preset an instrument matches by tuning. Custom tunings (a later
    // follow-up) match nothing and fall back to Standard.
    pub fn of(instrument: &Instrument) -> InstrumentPreset {
        InstrumentPreset::ALL
            .into_iter()
            .find(|preset| preset.instrument().tuning == instrument.tuning)
            .unwrap_or(InstrumentPreset::Standard)
    }
}

impl std::fmt::Display for InstrumentPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InstrumentPreset::Standard => "Standard 6-string",
            InstrumentPreset::DropD => "Drop D",
            InstrumentPreset::SevenString => "7-string",
            InstrumentPreset::BassFour => "Bass 4-string",
        })
    }
}

pub const TONICS: [SpelledPitch; 12] = [
    SpelledPitch::natural(Letter::C),
    SpelledPitch::new(Letter::D, Alteration::Flat),
    SpelledPitch::natural(Letter::D),
    SpelledPitch::new(Letter::E, Alteration::Flat),
    SpelledPitch::natural(Letter::E),
    SpelledPitch::natural(Letter::F),
    SpelledPitch::new(Letter::F, Alteration::Sharp),
    SpelledPitch::natural(Letter::G),
    SpelledPitch::new(Letter::A, Alteration::Flat),
    SpelledPitch::natural(Letter::A),
    SpelledPitch::new(Letter::B, Alteration::Flat),
    SpelledPitch::natural(Letter::B),
];

pub fn key_of(tonic: SpelledPitch, quality: Quality) -> Key {
    Key::new(tonic, quality)
}

pub fn mode_of(quality: Quality) -> Mode {
    Mode::from(quality)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelMode {
    Degrees,
    Notes,
}

impl std::fmt::Display for LabelMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LabelMode::Degrees => "Degrees",
            LabelMode::Notes => "Notes",
        })
    }
}

pub struct Dot {
    pub position: FretboardPosition,
    pub label: String,
    // The key-anchored register the dot sits in — the count of 7-degree
    // cycles above the tonic's register 0.
    pub octave: i32,
    pub is_root: bool,
}

pub fn degree_label(dr: DegreeReference) -> String {
    dr.scale_degree.label().to_string()
}

// The slot's degree label against the chunk's anchor note: a trailing
// apostrophe per octave above it, a trailing comma per octave below — chunk
// octaves count from the first note of the chunk.
pub fn slot_label(dr: DegreeReference) -> String {
    let mut label = dr.scale_degree.label();
    for _ in 0..dr.octave {
        label.push('\'');
    }
    for _ in 0..(-dr.octave) {
        label.push(',');
    }
    label
}

// The slot's absolute note against the chunk's base register — [base] is
// the key-anchored octave of the chunk's own degree-1, the register the
// editor's clicks resolve against.
pub fn degree_note_in_register(key: Key, mode: Mode, base: i32, dr: DegreeReference) -> Note {
    dr.note(Note::new(key.tonic, base), mode)
}

pub const ROMANS: [&str; 7] = ["I", "II", "III", "IV", "V", "VI", "VII"];

pub fn roman(root: i32) -> &'static str {
    ROMANS[(root - 1).rem_euclid(7) as usize]
}

// Chunk labels from the lineage: roots are lettered in order, each variant
// numbered under its root ("A", "A1", "A2", ...). A variant's parent is
// always a root, processed earlier, so its letter is already known.
pub fn chunk_names(lineage: &[Option<usize>]) -> Vec<String> {
    let mut names = Vec::with_capacity(lineage.len());
    let mut root_letter = vec![b'A'; lineage.len()];
    let mut variant_count = vec![0usize; lineage.len()];
    let mut next_root = 0usize;
    for (i, &parent) in lineage.iter().enumerate() {
        match parent {
            None => {
                root_letter[i] = b'A' + (next_root % 26) as u8;
                next_root += 1;
                names.push(char::from(root_letter[i]).to_string());
            }
            Some(root) => {
                variant_count[root] += 1;
                names.push(format!(
                    "{}{}",
                    char::from(root_letter[root]),
                    variant_count[root]
                ));
            }
        }
    }
    names
}

fn label_of(label_mode: LabelMode, key: Key, mode: Mode, dr: DegreeReference) -> String {
    match label_mode {
        LabelMode::Degrees => degree_label(dr),
        LabelMode::Notes => degree_note(key, mode, dr).to_string(),
    }
}

// Every playable position, split into in-key dots (labeled) and accidental
// dots — off-key positions resolved to their spelled degree, labeled for
// when they're played. Order is fixed — string, then fret.
pub fn board(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    label_mode: LabelMode,
) -> (Vec<Dot>, Vec<Dot>) {
    let in_key: Vec<Dot> = degrees_in_window(instrument, key, mode, 0, instrument.max_fret())
        .into_iter()
        .map(|(position, degree)| Dot {
            position,
            label: label_of(label_mode, key, mode, degree),
            octave: degree.octave,
            is_root: degree.scale_degree.degree == 1,
        })
        .collect();

    let mut accidentals = Vec::new();
    for string_index in 0..instrument.string_count() {
        for fret in 0..=instrument.fret_count(string_index) {
            let position = FretboardPosition { string_index, fret };
            if degree_at(instrument, key, mode, position).is_none() {
                let degree = spelled_degree_at(instrument, key, mode, position);
                accidentals.push(Dot {
                    position,
                    label: label_of(label_mode, key, mode, degree),
                    octave: degree.octave,
                    is_root: false,
                });
            }
        }
    }

    (in_key, accidentals)
}

pub fn initial_chunk() -> Word {
    Word::new(vec![
        Slot::Note(DegreeReference::natural(1, 0)),
        Slot::Note(DegreeReference::new(
            ScaleDegree::new(3, Alteration::Flat),
            0,
        )),
        Slot::Note(DegreeReference::natural(5, 0)),
        Slot::Note(DegreeReference::natural(6, 0)),
    ])
}

pub fn second_chunk() -> Word {
    Word::new(vec![
        Slot::Note(DegreeReference::natural(1, 0)),
        Slot::Note(DegreeReference::natural(5, 0)),
    ])
}

// The placeholder phrase: two bars of sixteenth notes demonstrating the
// rhythm row's standard notation — one variation per beat: a quarter,
// two eighths, a sixteenth pair plus an eighth, an eighth plus a sixteenth
// pair; then a quarter, a sixteenth and a dotted eighth, and two silent
// beats. Placeholder content; the demo needs the values, beams, flags,
// dots, and rests, not any particular pitches.
pub fn phrase_chunk() -> Word {
    let note = |degree: i32| Slot::Note(DegreeReference::natural(degree, 0));
    Word::new(vec![
        // Bar 1: O--- | O-O- | OOO- | O-OO.
        note(1),
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
        note(3),
        Slot::Rest,
        note(5),
        Slot::Rest,
        note(6),
        note(5),
        note(4),
        Slot::Rest,
        note(3),
        Slot::Rest,
        note(5),
        note(6),
        // Bar 2: O--- | OO-- | rest | rest.
        note(5),
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
        note(6),
        note(5),
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
        Slot::Rest,
    ])
}

// Every fingering of the chunk over one chord root, ranked by reach distance
// from [anchor] with consecutive moves within [fret_cap] frets, plus the
// slot-to-note index map (None for rests) that aligns the chunk's slots with
// the fingering rows.
// The chunk's notes and the slot-to-note alignment (None for rests).
fn notes_of(chunk: &Word, root_degree: i32) -> (Vec<DegreeReference>, Vec<Option<usize>>) {
    let reframed = &chunk::apply_progression(chunk, &[root_degree])[0];
    let mut slot_notes: Vec<Option<usize>> = vec![None; reframed.len()];
    let mut notes = Vec::new();
    for (i, slot) in reframed.iter().enumerate() {
        if let Slot::Note(dr) = slot {
            slot_notes[i] = Some(notes.len());
            notes.push(*dr);
        }
    }
    (notes, slot_notes)
}

pub fn solve(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    chunk: &Word,
    root_degree: i32,
    anchor: FretboardPosition,
    fret_cap: i32,
) -> (Vec<Fingering>, Vec<Option<usize>>) {
    let (notes, slot_notes) = notes_of(chunk, root_degree);
    let fingerings = chunk_solver::positions(key, mode, instrument, anchor, fret_cap, &notes);
    (fingerings, slot_notes)
}

// Every fingering of the chunk whose first note is played exactly at
// [first] — the rotation catalog around a pinned root note.
pub fn solve_pinned(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    chunk: &Word,
    root_degree: i32,
    first: FretboardPosition,
    fret_cap: i32,
) -> (Vec<Fingering>, Vec<Option<usize>>) {
    let (notes, slot_notes) = notes_of(chunk, root_degree);
    let fingerings = chunk_solver::positions_from(key, mode, instrument, first, fret_cap, &notes);
    (fingerings, slot_notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_label_marks_each_octave_against_the_anchor_note() {
        assert_eq!(slot_label(DegreeReference::natural(2, 0)), "2");
        assert_eq!(slot_label(DegreeReference::natural(2, 1)), "2'");
        assert_eq!(
            slot_label(DegreeReference::new(
                ScaleDegree::new(3, Alteration::Flat),
                2
            )),
            "b3''"
        );
        assert_eq!(slot_label(DegreeReference::natural(7, -1)), "7,");
    }

    #[test]
    fn chunk_names_letter_roots_and_number_variants_under_their_root() {
        let lineage = [None, Some(0), Some(0), None, Some(3)];
        assert_eq!(
            chunk_names(&lineage),
            vec!["A", "A1", "A2", "B", "B1"],
            "roots lettered in order, variants numbered under their root"
        );
        assert_eq!(chunk_names(&[None, None]), vec!["A", "B"]);
    }

    #[test]
    fn degree_note_in_register_names_the_register_the_chunk_sits_in() {
        let key = key_of(TONICS[0], Quality::Major);
        let note = degree_note_in_register(key, Mode::Ionian, 2, DegreeReference::natural(3, 0));
        assert_eq!(note.to_string(), "E2");
    }

    #[test]
    fn board_dots_carry_spn_labels_and_key_registers() {
        let instrument = instrument();
        let key = key_of(TONICS[0], Quality::Major);
        let f4 = FretboardPosition {
            string_index: 5,
            fret: 1,
        };

        let (dots, _) = board(&instrument, key, Mode::Ionian, LabelMode::Notes);
        let dot = dots
            .iter()
            .find(|dot| dot.position == f4)
            .expect("F4 is in key");
        assert_eq!(dot.label, "F4", "note names are full SPN");
        assert_eq!(dot.octave, 4, "the key-anchored register");

        let (dots, _) = board(&instrument, key, Mode::Ionian, LabelMode::Degrees);
        let dot = dots
            .iter()
            .find(|dot| dot.position == f4)
            .expect("F4 is in key");
        assert_eq!(dot.label, "4");
        assert_eq!(
            dot.octave, 4,
            "one octave count per cycle of 7 degrees above the tonic's register"
        );
    }

    #[test]
    fn accidentals_carry_spelled_degree_labels() {
        let instrument = instrument();
        let key = key_of(TONICS[0], Quality::Major);
        let eb4 = FretboardPosition {
            string_index: 4,
            fret: 4,
        };

        let (_, accidentals) = board(&instrument, key, Mode::Ionian, LabelMode::Degrees);
        let dot = accidentals
            .iter()
            .find(|dot| dot.position == eb4)
            .expect("Eb4 is off-key in C major");
        assert_eq!(dot.label, "b3", "the spelled degree label");
        assert_eq!(dot.octave, 4, "the key-referenced octave count");

        let (_, accidentals) = board(&instrument, key, Mode::Ionian, LabelMode::Notes);
        let dot = accidentals
            .iter()
            .find(|dot| dot.position == eb4)
            .expect("Eb4 is off-key in C major");
        assert_eq!(dot.label, "Eb4", "full SPN in note-name mode");
    }

    #[test]
    fn instrument_presets_round_trip_through_their_instrument() {
        for preset in InstrumentPreset::ALL {
            assert_eq!(InstrumentPreset::of(&preset.instrument()), preset);
        }
    }

    #[test]
    fn the_default_instrument_is_standard_with_twenty_two_frets() {
        let instrument = instrument();
        assert_eq!(instrument, InstrumentPreset::Standard.instrument());
        assert_eq!(instrument.max_fret(), 22);
    }
}
