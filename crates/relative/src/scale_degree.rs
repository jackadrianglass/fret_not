use absolute::alteration::Alteration;
use absolute::pitch_class::PitchClass;
use absolute::spelled_pitch::SpelledPitch;

use crate::mode::Mode;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ScaleDegree {
    pub degree: i32,
    pub alteration: Alteration,
}

impl ScaleDegree {
    pub fn create(degree: i32, alteration: Alteration) -> ScaleDegree {
        ScaleDegree { degree, alteration }
    }

    pub fn natural(degree: i32) -> ScaleDegree {
        ScaleDegree {
            degree,
            alteration: Alteration::Natural,
        }
    }

    pub fn diatonic() -> Vec<ScaleDegree> {
        (1..=7).map(ScaleDegree::natural).collect()
    }

    pub fn label(self) -> String {
        format!("{}{}", self.alteration.to_string(), self.degree)
    }

    pub fn pitch_class(self, root: PitchClass, mode: Mode) -> PitchClass {
        let base = mode.pitch_classes(root)[(self.degree - 1) as usize];
        base.add(self.alteration.semitones())
    }

    // The degree as a spelled pitch: the letter is the root's letter stepped
    // [degree - 1] letters up; the alteration field is the accidental.
    // Panics when the spelling would need more than a double sharp/flat.
    pub fn spelled(self, root: SpelledPitch, mode: Mode) -> SpelledPitch {
        let letter = root.letter.offset(self.degree - 1);
        SpelledPitch::of_pitch_class_exn(letter, self.pitch_class(root.pitch_class(), mode))
    }
}

#[cfg(test)]
mod tests {
    use super::ScaleDegree;
    use crate::mode::Mode;
    use absolute::alteration::Alteration;
    use absolute::pitch_class::PitchClass;

    #[test]
    fn diatonic_is_seven_natural_degrees() {
        assert_eq!(
            ScaleDegree::diatonic()
                .iter()
                .map(|sd| sd.degree)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6, 7],
            "degrees 1..7"
        );
        assert!(
            ScaleDegree::diatonic()
                .iter()
                .all(|sd| sd.alteration == Alteration::Natural),
            "all natural"
        );
    }

    #[test]
    fn pitch_class_follows_the_mode_from_its_root() {
        let root = PitchClass::of_int(2);
        assert_eq!(
            ScaleDegree::diatonic()
                .iter()
                .map(|sd| sd.pitch_class(root, Mode::Dorian).to_string())
                .collect::<Vec<_>>(),
            vec!["D", "E", "F", "G", "A", "B", "C"],
            "D Dorian"
        );
    }

    #[test]
    fn alteration_shifts_the_degree_pitch_class() {
        let root = PitchClass::of_int(0);
        assert_eq!(
            [1, 2, 4, 5, 6]
                .iter()
                .map(|degree| {
                    ScaleDegree::create(*degree, Alteration::Sharp)
                        .pitch_class(root, Mode::Ionian)
                        .to_string()
                })
                .collect::<Vec<_>>(),
            vec!["C#", "D#", "F#", "G#", "A#"],
            "C major chromatic degrees"
        );
    }

    #[test]
    fn label_marks_the_alteration() {
        let cases = [
            (1, Alteration::Natural),
            (3, Alteration::Flat),
            (4, Alteration::Sharp),
            (4, Alteration::DoubleSharp),
            (7, Alteration::DoubleFlat),
        ];
        assert_eq!(
            cases
                .iter()
                .map(|(degree, alteration)| { ScaleDegree::create(*degree, *alteration).label() })
                .collect::<Vec<_>>(),
            vec!["1", "b3", "#4", "##4", "bb7"],
            "labels"
        );
    }
}
