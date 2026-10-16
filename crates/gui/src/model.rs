use absolute::alteration::Alteration;
use absolute::letter::Letter;
use absolute::note::Note;
use absolute::spelled_pitch::SpelledPitch;
use fretboard::chunk_solver;
use fretboard::fretboard::{degree_at, degree_note, degrees_in_window};
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::tuning;
use relative::chunk::{self, Chunk, Slot};
use relative::degree_reference::DegreeReference;
use relative::key::{Key, Quality};
use relative::mode::Mode;
use relative::scale_degree::ScaleDegree;

pub const PROGRESSION: [i32; 3] = [1, 4, 5];
pub const PROGRESSION_LABELS: [&str; 3] = ["I", "IV", "V"];
pub const MAX_SLOTS: usize = 12;

// The solver's exploration window: fingerings must stay within this fret
// distance of the anchor.
pub const SOLVER_ANCHOR: FretboardPosition = FretboardPosition {
    string_index: 0,
    fret: 5,
};
pub const SOLVER_FRET_CAP: i32 = 6;

pub fn instrument() -> Instrument {
    Instrument::new_uniform(tuning::standard(), 22)
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LabelMode {
    Degrees,
    Notes,
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

pub fn chunk_name(index: usize) -> String {
    char::from(b'A' + (index % 26) as u8).to_string()
}

fn label_of(label_mode: LabelMode, key: Key, mode: Mode, dr: DegreeReference) -> String {
    match label_mode {
        LabelMode::Degrees => degree_label(dr),
        LabelMode::Notes => degree_note(key, mode, dr).to_string(),
    }
}

// Every playable position, split into in-key dots (labeled) and off-key
// positions. Order is fixed — string, then fret.
pub fn board(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    label_mode: LabelMode,
) -> (Vec<Dot>, Vec<FretboardPosition>) {
    let in_key: Vec<Dot> = degrees_in_window(instrument, key, mode, 0, instrument.max_fret())
        .into_iter()
        .map(|(position, degree)| Dot {
            position,
            label: label_of(label_mode, key, mode, degree),
            octave: degree.octave,
            is_root: degree.scale_degree.degree == 1,
        })
        .collect();

    let mut off_key = Vec::new();
    for string_index in 0..instrument.string_count() {
        for fret in 0..=instrument.fret_count(string_index) {
            let position = FretboardPosition { string_index, fret };
            if degree_at(instrument, key, mode, position).is_none() {
                off_key.push(position);
            }
        }
    }

    (in_key, off_key)
}

pub fn initial_chunk() -> Chunk {
    Chunk::new(vec![
        Slot::Note(DegreeReference::natural(1, 0)),
        Slot::Note(DegreeReference::new(
            ScaleDegree::new(3, Alteration::Flat),
            0,
        )),
        Slot::Note(DegreeReference::natural(5, 0)),
        Slot::Note(DegreeReference::natural(6, 0)),
    ])
}

pub fn second_chunk() -> Chunk {
    Chunk::new(vec![
        Slot::Note(DegreeReference::natural(1, 0)),
        Slot::Note(DegreeReference::natural(5, 0)),
    ])
}

// Every fingering of the chunk over one chord root, ranked by reach distance
// from [anchor] with consecutive moves within [fret_cap] frets, plus the
// slot-to-note index map (None for rests) that aligns the chunk's slots with
// the fingering rows.
pub fn solve(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    chunk: &Chunk,
    root_degree: i32,
    anchor: FretboardPosition,
    fret_cap: i32,
) -> (Vec<Vec<FretboardPosition>>, Vec<Option<usize>>) {
    let reframed = &chunk::apply_progression(chunk, &[root_degree])[0];
    let mut slot_notes: Vec<Option<usize>> = vec![None; reframed.len()];
    let mut notes = Vec::new();
    for (i, slot) in reframed.iter().enumerate() {
        if let Slot::Note(dr) = slot {
            slot_notes[i] = Some(notes.len());
            notes.push(*dr);
        }
    }
    let fingerings = chunk_solver::positions(key, mode, instrument, anchor, fret_cap, &notes);
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
}
