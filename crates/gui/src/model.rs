use absolute::alteration::Alteration;
use absolute::letter::Letter;
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
    pub is_root: bool,
}

pub fn degree_label(dr: DegreeReference) -> String {
    dr.scale_degree.label().to_string()
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
        LabelMode::Notes => degree_note(key, mode, dr).spelled_pitch.to_string(),
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

// Every fingering of the chunk over one chord root, ranked by reach distance,
// plus the slot-to-note index map (None for rests) that aligns the chunk's
// slots with the fingering rows.
pub fn solve(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    chunk: &Chunk,
    root_degree: i32,
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
    let fingerings = chunk_solver::positions(
        key,
        mode,
        instrument,
        SOLVER_ANCHOR,
        SOLVER_FRET_CAP,
        &notes,
    );
    (fingerings, slot_notes)
}
