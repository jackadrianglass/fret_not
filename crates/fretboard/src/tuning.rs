use absolute::letter::{ALL, Letter};
use absolute::note::Note;
use absolute::pitch_class::PitchClass;
use absolute::spelled_pitch::SpelledPitch;

// Open-string pitches from lowest to highest, as absolute SPN notes.
pub type Tuning = Vec<Note>;

pub fn create(open_string_notes: Vec<Note>) -> Tuning {
    open_string_notes
}

pub fn string_count(tuning: &Tuning) -> usize {
    tuning.len()
}

pub fn open_note(tuning: &Tuning, string_index: usize) -> Note {
    tuning[string_index]
}

pub fn open_semitone(tuning: &Tuning, string_index: usize) -> i32 {
    open_note(tuning, string_index).semitone()
}

// Sounding semitones above C0 of a position: the open string's semitone plus
// the fret.
pub fn semitone_at(tuning: &Tuning, string_index: usize, fret: i32) -> i32 {
    open_semitone(tuning, string_index) + fret
}

fn spelled_pitch_of_pitch_class(pitch_class: PitchClass) -> SpelledPitch {
    ALL.iter()
        .filter_map(|letter| {
            SpelledPitch::alteration_for(*letter, pitch_class)
                .map(|alteration| SpelledPitch::create(*letter, alteration))
        })
        .min_by(|a, b| {
            let a_semitones = a.alteration.semitones();
            let b_semitones = b.alteration.semitones();
            a_semitones
                .abs()
                .cmp(&b_semitones.abs())
                .then(a_semitones.cmp(&b_semitones))
        })
        .unwrap_or_else(|| {
            panic!(
                "no accidental spells pitch class {}",
                pitch_class.to_string()
            )
        })
}

// Shift one string's open note by semitones, spelling the result with the
// smallest accidental available. Drop tunings derive from [standard].
pub fn retune_string(tuning: &Tuning, string_index: usize, semitones: i32) -> Tuning {
    tuning
        .iter()
        .enumerate()
        .map(|(i, note)| {
            if i == string_index {
                let pitch_class = note.pitch_class().add(semitones);
                let spelled = spelled_pitch_of_pitch_class(pitch_class);
                Note::create(
                    spelled,
                    (note.semitone() + semitones - spelled.semitone()) / 12 - 1,
                )
            } else {
                *note
            }
        })
        .collect()
}

pub fn standard() -> Tuning {
    vec![
        Note::natural(Letter::E, 2),
        Note::natural(Letter::A, 2),
        Note::natural(Letter::D, 3),
        Note::natural(Letter::G, 3),
        Note::natural(Letter::B, 3),
        Note::natural(Letter::E, 4),
    ]
}

pub fn drop_d() -> Tuning {
    retune_string(&standard(), 0, -2)
}

pub fn standard_seven_string() -> Tuning {
    let mut tuning = vec![Note::natural(Letter::B, 1)];
    tuning.extend(standard());
    tuning
}

pub fn bass_four() -> Tuning {
    standard()
        .iter()
        .take(4)
        .map(|note| note.shift_octave(-1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        Tuning, bass_four, create, drop_d, open_note, retune_string, semitone_at, standard,
        standard_seven_string, string_count,
    };

    fn open_notes(tuning: &Tuning) -> Vec<String> {
        (0..string_count(tuning))
            .map(|string_index| open_note(tuning, string_index).to_string())
            .collect()
    }

    #[test]
    fn standard_tuning_open_strings_are_absolute_spn_notes() {
        assert_eq!(
            open_notes(&standard()),
            vec!["E2", "A2", "D3", "G3", "B3", "E4"],
            "standard tuning"
        );
    }

    #[test]
    fn drop_d_only_lowers_the_low_string() {
        assert_eq!(
            open_notes(&drop_d()),
            vec!["D2", "A2", "D3", "G3", "B3", "E4"],
            "drop D"
        );
    }

    #[test]
    fn seven_string_tuning_adds_a_low_b() {
        assert_eq!(
            open_notes(&standard_seven_string()),
            vec!["B1", "E2", "A2", "D3", "G3", "B3", "E4"],
            "seven string"
        );
    }

    #[test]
    fn bass_four_is_an_octave_below_the_guitar() {
        assert_eq!(
            open_notes(&bass_four()),
            vec!["E1", "A1", "D2", "G2"],
            "four string bass"
        );
    }

    #[test]
    fn semitone_at_is_open_semitone_plus_fret() {
        assert_eq!(
            vec![
                semitone_at(&standard(), 0, 0),
                semitone_at(&standard(), 0, 12),
                semitone_at(&standard(), 4, 1),
            ],
            vec![40, 52, 60],
            "E2 open, E3 at the 12th fret, C4 on the B string's first fret"
        );
    }

    #[test]
    fn retune_string_shifts_one_string_and_spells_minimally() {
        let retuned = retune_string(&standard(), 1, -2);
        assert_eq!(
            open_notes(&retuned),
            vec!["E2", "G2", "D3", "G3", "B3", "E4"],
            "A string down a whole step, others untouched"
        );
    }

    #[test]
    fn create_is_the_identity_on_the_notes() {
        assert_eq!(open_notes(&create(standard())), open_notes(&standard()));
    }
}
