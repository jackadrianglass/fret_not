use crate::scale_degree::ScaleDegree;

// Degrees 1 2 3 5 6.
pub const PENTATONIC_MAJOR: [ScaleDegree; 5] = [
    ScaleDegree::natural(1),
    ScaleDegree::natural(2),
    ScaleDegree::natural(3),
    ScaleDegree::natural(5),
    ScaleDegree::natural(6),
];

// Degrees 1 3 4 5 7.
pub const PENTATONIC_MINOR: [ScaleDegree; 5] = [
    ScaleDegree::natural(1),
    ScaleDegree::natural(3),
    ScaleDegree::natural(4),
    ScaleDegree::natural(5),
    ScaleDegree::natural(7),
];

// The triad: degrees 1 3 5.
pub const ARPEGGIO_MAJOR: [ScaleDegree; 3] = [
    ScaleDegree::natural(1),
    ScaleDegree::natural(3),
    ScaleDegree::natural(5),
];

// Same degrees as the major triad: scale-relative degrees already spell the
// minor triad in a minor key.
pub const ARPEGGIO_MINOR: [ScaleDegree; 3] = ARPEGGIO_MAJOR;

#[cfg(test)]
mod tests {
    use super::{ARPEGGIO_MAJOR, ARPEGGIO_MINOR, PENTATONIC_MAJOR, PENTATONIC_MINOR};
    use crate::mode::Mode;
    use crate::scale_degree::ScaleDegree;
    use absolute::pitch_class::PitchClass;

    fn degree_names(root: PitchClass, mode: Mode, degrees: &[ScaleDegree]) -> Vec<&'static str> {
        degrees
            .iter()
            .map(|sd| sd.pitch_class(root, mode).as_str())
            .collect()
    }

    #[test]
    fn c_major_pentatonic_drops_the_fourth_and_seventh() {
        assert_eq!(
            degree_names(PitchClass::C, Mode::Ionian, &PENTATONIC_MAJOR),
            vec!["C", "D", "E", "G", "A"],
            "C major pentatonic"
        );
    }

    #[test]
    fn a_minor_pentatonic_drops_the_second_and_sixth() {
        assert_eq!(
            degree_names(PitchClass::A, Mode::Aeolian, &PENTATONIC_MINOR),
            vec!["A", "C", "D", "E", "G"],
            "A minor pentatonic"
        );
    }

    #[test]
    fn c_major_arpeggio_is_the_major_triad() {
        assert_eq!(
            degree_names(PitchClass::C, Mode::Ionian, &ARPEGGIO_MAJOR),
            vec!["C", "E", "G"],
            "C major arpeggio"
        );
    }

    #[test]
    fn a_minor_arpeggio_is_the_minor_triad() {
        assert_eq!(
            degree_names(PitchClass::A, Mode::Aeolian, &ARPEGGIO_MINOR),
            vec!["A", "C", "E"],
            "A minor arpeggio"
        );
    }
}
