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
    Instrument::create_uniform(tuning::standard(), 22)
}

pub fn tonic_list() -> Vec<SpelledPitch> {
    use Alteration::{Flat, Sharp};
    use Letter::{A, B, C, D, E, F, G};
    vec![
        SpelledPitch::natural(C),
        SpelledPitch::create(D, Flat),
        SpelledPitch::natural(D),
        SpelledPitch::create(E, Flat),
        SpelledPitch::natural(E),
        SpelledPitch::natural(F),
        SpelledPitch::create(F, Sharp),
        SpelledPitch::natural(G),
        SpelledPitch::create(A, Flat),
        SpelledPitch::natural(A),
        SpelledPitch::create(B, Flat),
        SpelledPitch::natural(B),
    ]
}

pub fn key_of(tonic_index: usize, quality: Quality) -> Key {
    Key::create(tonic_list()[tonic_index], quality)
}

pub fn mode_of(quality: Quality) -> Mode {
    Key::mode_of_quality(quality)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LabelMode {
    Degrees,
    Notes,
}

pub struct Dot {
    pub position: FretboardPosition,
    pub degree: DegreeReference,
    pub label: String,
    pub is_root: bool,
}

pub fn degree_label(dr: DegreeReference) -> String {
    let apostrophes = "'".repeat(dr.octave.clamp(0, 4) as usize);
    format!("{}{}", dr.scale_degree.label(), apostrophes)
}

fn label_of(label_mode: LabelMode, key: Key, mode: Mode, dr: DegreeReference) -> String {
    match label_mode {
        LabelMode::Degrees => degree_label(dr),
        LabelMode::Notes => degree_note(key, mode, dr).spelled_pitch.to_string(),
    }
}

// Every playable position, split into in-key dots (with degree and label) and
// off-key positions. Order is fixed — string, then fret — and the main view
// zips animation state against it by index.
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
            degree,
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
    vec![
        Slot::Note(DegreeReference::natural(1, 0)),
        Slot::Note(DegreeReference::create(
            ScaleDegree::create(3, Alteration::Flat),
            0,
        )),
        Slot::Note(DegreeReference::natural(5, 0)),
        Slot::Note(DegreeReference::natural(6, 0)),
    ]
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
