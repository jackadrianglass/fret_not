use crate::scale_degree::ScaleDegree;

fn keep(allowed: &[i32]) -> Vec<ScaleDegree> {
    ScaleDegree::diatonic()
        .into_iter()
        .filter(|sd| allowed.contains(&sd.degree))
        .collect()
}

// Degrees 1 2 3 5 6.
pub fn pentatonic_major() -> Vec<ScaleDegree> {
    keep(&[1, 2, 3, 5, 6])
}

// Degrees 1 3 4 5 7.
pub fn pentatonic_minor() -> Vec<ScaleDegree> {
    keep(&[1, 3, 4, 5, 7])
}

// The triad: degrees 1 3 5.
pub fn arpeggio_major() -> Vec<ScaleDegree> {
    keep(&[1, 3, 5])
}

// Same degrees as the major triad: scale-relative degrees already spell the
// minor triad in a minor key.
pub fn arpeggio_minor() -> Vec<ScaleDegree> {
    arpeggio_major()
}

#[cfg(test)]
mod tests {
    use super::{arpeggio_major, arpeggio_minor, pentatonic_major, pentatonic_minor};
    use crate::mode::Mode;
    use crate::scale_degree::ScaleDegree;
    use absolute::pitch_class::PitchClass;

    fn degree_names(root: PitchClass, mode: Mode, degrees: &[ScaleDegree]) -> Vec<&'static str> {
        degrees
            .iter()
            .map(|sd| sd.pitch_class(root, mode).to_string())
            .collect()
    }

    #[test]
    fn c_major_pentatonic_drops_the_fourth_and_seventh() {
        assert_eq!(
            degree_names(PitchClass::C, Mode::Ionian, &pentatonic_major()),
            vec!["C", "D", "E", "G", "A"],
            "C major pentatonic"
        );
    }

    #[test]
    fn a_minor_pentatonic_drops_the_second_and_sixth() {
        assert_eq!(
            degree_names(PitchClass::A, Mode::Aeolian, &pentatonic_minor()),
            vec!["A", "C", "D", "E", "G"],
            "A minor pentatonic"
        );
    }

    #[test]
    fn c_major_arpeggio_is_the_major_triad() {
        assert_eq!(
            degree_names(PitchClass::C, Mode::Ionian, &arpeggio_major()),
            vec!["C", "E", "G"],
            "C major arpeggio"
        );
    }

    #[test]
    fn a_minor_arpeggio_is_the_minor_triad() {
        assert_eq!(
            degree_names(PitchClass::A, Mode::Aeolian, &arpeggio_minor()),
            vec!["A", "C", "E"],
            "A minor arpeggio"
        );
    }
}
