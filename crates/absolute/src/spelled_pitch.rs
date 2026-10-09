use crate::alteration::Alteration;
use crate::letter::Letter;
use crate::pitch_class::PitchClass;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SpelledPitch {
    pub letter: Letter,
    pub alteration: Alteration,
}

impl SpelledPitch {
    pub fn create(letter: Letter, alteration: Alteration) -> SpelledPitch {
        SpelledPitch { letter, alteration }
    }

    pub fn natural(letter: Letter) -> SpelledPitch {
        SpelledPitch {
            letter,
            alteration: Alteration::Natural,
        }
    }

    // Sounding semitones relative to the same letter's natural, NOT folded
    // into an octave: Cb = -1, B# = 12. This is what lets Note handle the
    // B/C octave wrap correctly.
    pub fn semitone(self) -> i32 {
        self.letter.to_pitch_class().to_int() + self.alteration.semitones()
    }

    pub fn pitch_class(self) -> PitchClass {
        PitchClass::of_int(self.semitone())
    }

    // The accidental that makes `letter` sound as `pitch_class`, if one
    // exists within double sharp/flat.
    pub fn alteration_for(letter: Letter, pitch_class: PitchClass) -> Option<Alteration> {
        let natural_pitch_class = letter.to_pitch_class();
        let mut delta = (pitch_class.to_int() - natural_pitch_class.to_int()).rem_euclid(12);
        if delta > 6 {
            delta -= 12;
        }
        Alteration::of_semitones(delta)
    }

    pub fn of_pitch_class_exn(letter: Letter, pitch_class: PitchClass) -> SpelledPitch {
        match SpelledPitch::alteration_for(letter, pitch_class) {
            Some(alteration) => SpelledPitch { letter, alteration },
            None => panic!(
                "no accidental makes {} sound as {}",
                letter.to_string(),
                pitch_class.to_string()
            ),
        }
    }

    pub fn to_string(self) -> String {
        format!("{}{}", self.letter.to_string(), self.alteration.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{Alteration, Letter, SpelledPitch};

    #[test]
    fn to_string_spells_the_accidental() {
        assert_eq!(
            vec![
                SpelledPitch::create(Letter::C, Alteration::Sharp).to_string(),
                SpelledPitch::create(Letter::B, Alteration::Flat).to_string(),
                SpelledPitch::natural(Letter::F).to_string(),
                SpelledPitch::create(Letter::E, Alteration::DoubleSharp).to_string(),
                SpelledPitch::create(Letter::C, Alteration::DoubleFlat).to_string(),
            ],
            vec!["C#", "Bb", "F", "E##", "Cbb"],
            "spellings"
        );
    }

    #[test]
    fn semitone_is_not_folded_into_an_octave() {
        // What lets Note handle the B/C octave wrap: Cb sits a semitone
        // below C, B# a semitone above B, rather than being folded to their
        // pitch classes.
        assert_eq!(
            vec![
                SpelledPitch::create(Letter::C, Alteration::Flat).semitone(),
                SpelledPitch::create(Letter::B, Alteration::Sharp).semitone(),
            ],
            vec![-1, 12],
            "semitones"
        );
    }

    #[test]
    fn pitch_class_folds_through_the_wrap() {
        assert_eq!(
            vec![
                SpelledPitch::create(Letter::C, Alteration::Flat)
                    .pitch_class()
                    .to_int(),
                SpelledPitch::create(Letter::B, Alteration::Sharp)
                    .pitch_class()
                    .to_int(),
                SpelledPitch::create(Letter::F, Alteration::Sharp)
                    .pitch_class()
                    .to_int(),
            ],
            vec![11, 0, 6],
            "pitch classes: Cb -> 11 (B), B# -> 0 (C), F# -> 6"
        );
    }

    #[test]
    fn alteration_for_picks_the_smallest_representable_accidental() {
        let of_pair = |(letter, pitch_class): (Letter, i32)| {
            SpelledPitch::alteration_for(
                letter,
                crate::pitch_class::PitchClass::of_int(pitch_class),
            )
            .map(|alteration| alteration.semitones().to_string())
        };
        // G sounding as F# is spelled Gb, C sounding as B is spelled Cb,
        // and no double accidental can stretch C to E.
        assert_eq!(
            vec![
                of_pair((Letter::G, 6)),
                of_pair((Letter::C, 11)),
                of_pair((Letter::F, 5)),
                of_pair((Letter::C, 4)),
            ],
            vec![
                Some("-1".to_string()),
                Some("-1".to_string()),
                Some("0".to_string()),
                None,
            ],
            "accidentals in semitones: -1 = flat, 0 = natural"
        );
    }
}
