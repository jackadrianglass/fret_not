use crate::pitch_class::PitchClass;
use crate::spelled_pitch::SpelledPitch;

// A note in scientific pitch notation: a spelled pitch plus an octave, e.g.
// C4, E2, Bb3. The octave is the SPN octave: it increments at B -> C, so B3
// sounds below C4, and B#3 sounds as C4's pitch class. The derived equality is
// spelling-sensitive: C#4 <> Db4.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Note {
    pub spelled_pitch: SpelledPitch,
    pub octave: i32,
}

impl Note {
    pub fn create(spelled_pitch: SpelledPitch, octave: i32) -> Note {
        Note {
            spelled_pitch,
            octave,
        }
    }

    pub fn of_parts(
        letter: crate::letter::Letter,
        alteration: crate::alteration::Alteration,
        octave: i32,
    ) -> Note {
        Note::create(SpelledPitch::create(letter, alteration), octave)
    }

    pub fn natural(letter: crate::letter::Letter, octave: i32) -> Note {
        Note::create(SpelledPitch::natural(letter), octave)
    }

    // Sounding semitones above C0; 12 * (octave + 1) + spelled semitone.
    pub fn semitone(self) -> i32 {
        12 * (self.octave + 1) + self.spelled_pitch.semitone()
    }

    pub fn pitch_class(self) -> PitchClass {
        PitchClass::of_int(self.semitone())
    }

    pub fn to_string(self) -> String {
        format!("{}{}", self.spelled_pitch.to_string(), self.octave)
    }

    pub fn shift_octave(self, n: i32) -> Note {
        Note {
            spelled_pitch: self.spelled_pitch,
            octave: self.octave + n,
        }
    }
}

// Not derived: the order is by sounding semitone, not by the record
// fields; spelling only breaks ties.
impl Ord for Note {
    fn cmp(&self, other: &Note) -> std::cmp::Ordering {
        self.semitone()
            .cmp(&other.semitone())
            .then_with(|| self.spelled_pitch.cmp(&other.spelled_pitch))
    }
}

impl PartialOrd for Note {
    fn partial_cmp(&self, other: &Note) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::Note;
    use crate::alteration::Alteration;
    use crate::letter::Letter;

    fn note(letter: Letter, octave: i32) -> Note {
        Note::of_parts(letter, Alteration::Natural, octave)
    }

    #[test]
    fn semitone_matches_scientific_pitch_notation() {
        // Middle C is C4 and A440 is A4; B#3 sounds as C4 and Cb4 as B3.
        assert_eq!(
            vec![
                note(Letter::C, 4).semitone(),
                note(Letter::A, 4).semitone(),
                note(Letter::E, 2).semitone(),
                Note::of_parts(Letter::B, Alteration::Sharp, 3).semitone(),
                Note::of_parts(Letter::C, Alteration::Flat, 4).semitone(),
                note(Letter::C, 0).semitone(),
            ],
            vec![60, 69, 40, 60, 59, 12],
            "semitones"
        );
    }

    #[test]
    fn to_string_round_trips_the_common_cases() {
        assert_eq!(
            vec![
                note(Letter::C, 4).to_string(),
                note(Letter::E, 2).to_string(),
                Note::of_parts(Letter::C, Alteration::Sharp, 4).to_string(),
                Note::of_parts(Letter::E, Alteration::Flat, 3).to_string(),
                Note::of_parts(Letter::F, Alteration::DoubleSharp, 2).to_string(),
            ],
            vec!["C4", "E2", "C#4", "Eb3", "F##2"],
            "names"
        );
    }

    #[test]
    fn compare_breaks_enharmonic_ties_by_spelling() {
        let c_sharp_4 = Note::of_parts(Letter::C, Alteration::Sharp, 4);
        let d_flat_4 = Note::of_parts(Letter::D, Alteration::Flat, 4);
        assert!(
            c_sharp_4 < d_flat_4,
            "C#4 sorts before Db4 on the spelling tiebreak"
        );
    }

    #[test]
    fn compare_orders_by_sounding_semitone() {
        let b3 = note(Letter::B, 3);
        let c4 = note(Letter::C, 4);
        let c_sharp_4 = Note::of_parts(Letter::C, Alteration::Sharp, 4);
        assert!(b3 < c4 && c4 < c_sharp_4 && c_sharp_4 > b3, "B3 < C4 < C#4");
    }

    #[test]
    fn pitch_class_folds_through_the_octave_wrap() {
        assert!(
            Note::of_parts(Letter::B, Alteration::Sharp, 3).pitch_class()
                == crate::pitch_class::PitchClass::C,
            "B#3 sounds as C"
        );
    }

    #[test]
    fn shift_octave_moves_the_register() {
        assert_eq!(
            note(Letter::E, 2).shift_octave(1).to_string(),
            "E3",
            "E2 shifted up an octave"
        );
    }
}
