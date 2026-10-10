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
    // The seven natural degrees of any mode are the same notational objects.
    pub const DIATONIC: [ScaleDegree; 7] = [
        ScaleDegree::natural(1),
        ScaleDegree::natural(2),
        ScaleDegree::natural(3),
        ScaleDegree::natural(4),
        ScaleDegree::natural(5),
        ScaleDegree::natural(6),
        ScaleDegree::natural(7),
    ];

    pub fn new(degree: i32, alteration: Alteration) -> ScaleDegree {
        ScaleDegree { degree, alteration }
    }

    pub const fn natural(degree: i32) -> ScaleDegree {
        ScaleDegree {
            degree,
            alteration: Alteration::Natural,
        }
    }

    pub fn label(self) -> String {
        format!("{}{}", self.alteration, self.degree)
    }

    pub fn pitch_class(self, root: PitchClass, mode: Mode) -> PitchClass {
        let base = mode.pitch_classes(root)[(self.degree - 1) as usize];
        base + self.alteration.semitones()
    }

    // The degree as a spelled pitch: the letter is the root's letter stepped
    // [degree - 1] letters up; the alteration field is the accidental.
    // Panics when the spelling would need more than a double sharp/flat.
    pub fn spelled(self, root: SpelledPitch, mode: Mode) -> SpelledPitch {
        let letter = root.letter.offset(self.degree - 1);
        SpelledPitch::expect_pitch_class(letter, self.pitch_class(root.pitch_class(), mode))
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
            ScaleDegree::DIATONIC
                .iter()
                .map(|sd| sd.degree)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6, 7],
            "degrees 1..7"
        );
        assert!(
            ScaleDegree::DIATONIC
                .iter()
                .all(|sd| sd.alteration == Alteration::Natural),
            "all natural"
        );
    }

    #[test]
    fn pitch_class_follows_the_mode_from_its_root() {
        let root = PitchClass::from_int(2);
        assert_eq!(
            ScaleDegree::DIATONIC
                .iter()
                .map(|sd| sd.pitch_class(root, Mode::Dorian).to_string())
                .collect::<Vec<_>>(),
            vec!["D", "E", "F", "G", "A", "B", "C"],
            "D Dorian"
        );
    }

    #[test]
    fn alteration_shifts_the_degree_pitch_class() {
        let root = PitchClass::from_int(0);
        assert_eq!(
            [1, 2, 4, 5, 6]
                .iter()
                .map(|degree| {
                    ScaleDegree::new(*degree, Alteration::Sharp)
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
                .map(|(degree, alteration)| { ScaleDegree::new(*degree, *alteration).label() })
                .collect::<Vec<_>>(),
            vec!["1", "b3", "#4", "##4", "bb7"],
            "labels"
        );
    }
}
