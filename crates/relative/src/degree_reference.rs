use absolute::note::Note;
use absolute::pitch_class::PitchClass;

use crate::mode::Mode;
use crate::scale_degree::ScaleDegree;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DegreeReference {
    pub scale_degree: ScaleDegree,
    pub octave: i32,
}

impl DegreeReference {
    pub fn create(scale_degree: ScaleDegree, octave: i32) -> DegreeReference {
        DegreeReference {
            scale_degree,
            octave,
        }
    }

    pub fn natural(degree: i32, octave: i32) -> DegreeReference {
        DegreeReference::create(ScaleDegree::natural(degree), octave)
    }

    // Semitones above the tonic. The octave is tonic-anchored: octave 0 spans
    // from the tonic up to (but not including) the octave above, so degree 1
    // octave 0 IS the tonic and the flat 1 of octave 0 sounds a semitone
    // below the octave above the tonic.
    pub fn interval(self, root: PitchClass, mode: Mode) -> i32 {
        let pitch_class = self.scale_degree.pitch_class(root, mode);
        (pitch_class.to_int() - root.to_int()).rem_euclid(12) + 12 * self.octave
    }

    // Absolute spelled note; [root] supplies both the spelling anchor and
    // the octave register.
    pub fn note(self, root: Note, mode: Mode) -> Note {
        let spelled = self.scale_degree.spelled(root.spelled_pitch, mode);
        let sounding = root.semitone() + self.interval(root.pitch_class(), mode);
        Note::create(spelled, (sounding - spelled.semitone()) / 12 - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::DegreeReference;
    use crate::mode::Mode;
    use crate::scale_degree::ScaleDegree;
    use absolute::alteration::Alteration;
    use absolute::letter::Letter;
    use absolute::note::Note;
    use absolute::pitch_class::PitchClass;

    fn altered(degree: i32, alteration: Alteration, octave: i32) -> DegreeReference {
        DegreeReference::create(ScaleDegree::create(degree, alteration), octave)
    }

    #[test]
    fn interval_counts_semitones_above_the_tonic() {
        let root = PitchClass::C;
        assert_eq!(
            ScaleDegree::diatonic()
                .iter()
                .map(|sd| { DegreeReference::create(*sd, 0).interval(root, Mode::Ionian) })
                .collect::<Vec<_>>(),
            vec![0, 2, 4, 5, 7, 9, 11],
            "intervals of the C major degrees at octave 0"
        );
    }

    #[test]
    fn interval_anchors_the_octave_at_the_tonic() {
        assert_eq!(
            DegreeReference::natural(1, 1).interval(PitchClass::C, Mode::Ionian),
            12,
            "degree 1 octave 1 is 12 above the tonic"
        );
        assert_eq!(
            altered(1, Alteration::Flat, 0).interval(PitchClass::C, Mode::Ionian),
            11,
            "flat 1 of octave 0 sits under the octave above"
        );
        assert_eq!(
            DegreeReference::natural(7, 0).interval(PitchClass::A, Mode::Ionian),
            11,
            "interval is relative to any root"
        );
    }

    #[test]
    fn note_resolves_to_spn_relative_to_a_root_note() {
        let c4 = Note::natural(Letter::C, 4);
        assert_eq!(
            ScaleDegree::diatonic()
                .iter()
                .map(|sd| {
                    DegreeReference::create(*sd, 0)
                        .note(c4, Mode::Ionian)
                        .to_string()
                })
                .collect::<Vec<_>>(),
            vec!["C4", "D4", "E4", "F4", "G4", "A4", "B4"],
            "C major degrees anchored at C4"
        );
        assert_eq!(
            DegreeReference::natural(1, 1)
                .note(c4, Mode::Ionian)
                .to_string(),
            "C5",
            "degree 1 octave 1 is the octave above"
        );
    }

    #[test]
    fn note_spells_alterations_and_wraps_the_octave() {
        let c4 = Note::natural(Letter::C, 4);
        assert_eq!(
            altered(3, Alteration::Flat, 0)
                .note(c4, Mode::Ionian)
                .to_string(),
            "Eb4",
            "flat 3 of C major is spelled Eb"
        );
        assert_eq!(
            altered(1, Alteration::Flat, 0)
                .note(c4, Mode::Ionian)
                .to_string(),
            "Cb5",
            "flat 1 is spelled Cb and sounds as B"
        );
        let e3 = Note::natural(Letter::E, 3);
        assert_eq!(
            DegreeReference::natural(3, 0)
                .note(e3, Mode::Ionian)
                .to_string(),
            "G#3",
            "third of E major is spelled G#"
        );
    }
}
