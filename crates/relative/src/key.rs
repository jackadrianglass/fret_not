use absolute::pitch_class::PitchClass;
use absolute::spelled_pitch::SpelledPitch;

use crate::mode::Mode;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Quality {
    Major,
    Minor,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key {
    pub tonic: SpelledPitch,
    pub quality: Quality,
}

impl Key {
    pub fn create(tonic: SpelledPitch, quality: Quality) -> Key {
        Key { tonic, quality }
    }

    pub fn tonic_pitch_class(self) -> PitchClass {
        self.tonic.pitch_class()
    }

    fn parent_major_root_pitch_class(self) -> PitchClass {
        match self.quality {
            Quality::Major => self.tonic_pitch_class(),
            Quality::Minor => self
                .tonic_pitch_class()
                .add(-Mode::Aeolian.root_offset_semitones()),
        }
    }

    fn parent_major_root(self) -> SpelledPitch {
        match self.quality {
            Quality::Major => self.tonic,
            Quality::Minor => {
                let letter = self.tonic.letter.offset(2);
                SpelledPitch::of_pitch_class_exn(letter, self.parent_major_root_pitch_class())
            }
        }
    }

    pub fn mode_of_quality(quality: Quality) -> Mode {
        match quality {
            Quality::Major => Mode::Ionian,
            Quality::Minor => Mode::Aeolian,
        }
    }

    pub fn mode_root(self, mode: Mode) -> SpelledPitch {
        let letter = self
            .parent_major_root()
            .letter
            .offset(mode.rotation_index() as i32);
        let pitch_class = self
            .parent_major_root_pitch_class()
            .add(mode.root_offset_semitones());
        SpelledPitch::of_pitch_class_exn(letter, pitch_class)
    }

    pub fn mode_pitch_classes(self, mode: Mode) -> Vec<PitchClass> {
        mode.pitch_classes(self.mode_root(mode).pitch_class())
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, Quality};
    use crate::mode::{ALL, Mode};
    use absolute::alteration::Alteration;
    use absolute::letter::Letter;
    use absolute::spelled_pitch::SpelledPitch;

    #[test]
    fn mode_root_spells_each_mode_of_c_major() {
        let key = Key::create(SpelledPitch::natural(Letter::C), Quality::Major);
        assert_eq!(
            ALL.iter()
                .map(|mode| key.mode_root(*mode).to_string())
                .collect::<Vec<_>>(),
            vec!["C", "D", "E", "F", "G", "A", "B"],
            "mode roots of C major"
        );
    }

    #[test]
    fn mode_root_spells_each_mode_of_a_minor() {
        // Ionian comes first in Mode::ALL, so a minor key's mode roots run
        // from the parent major's tonic, not the minor tonic.
        let key = Key::create(SpelledPitch::natural(Letter::A), Quality::Minor);
        assert_eq!(
            ALL.iter()
                .map(|mode| key.mode_root(*mode).to_string())
                .collect::<Vec<_>>(),
            vec!["C", "D", "E", "F", "G", "A", "B"],
            "mode roots of A minor, Ionian first"
        );
    }

    #[test]
    fn mode_root_spells_accidentals_off_the_parent_major() {
        let f_sharp_minor = Key::create(
            SpelledPitch::create(Letter::F, Alteration::Sharp),
            Quality::Minor,
        );
        assert_eq!(
            f_sharp_minor.mode_root(Mode::Aeolian).to_string(),
            "F#",
            "Aeolian root of F# minor is F#"
        );
        assert_eq!(
            f_sharp_minor.mode_root(Mode::Ionian).to_string(),
            "A",
            "Ionian root of F# minor is the parent major A"
        );
        let e_flat_major = Key::create(
            SpelledPitch::create(Letter::E, Alteration::Flat),
            Quality::Major,
        );
        assert_eq!(
            e_flat_major.mode_root(Mode::Lydian).to_string(),
            "Ab",
            "Lydian root of Eb major is Ab"
        );
    }

    #[test]
    fn mode_pitch_classes_are_the_modes_scale() {
        let key = Key::create(SpelledPitch::natural(Letter::C), Quality::Major);
        assert_eq!(
            key.mode_pitch_classes(Mode::Dorian)
                .iter()
                .map(|pc| pc.to_string())
                .collect::<Vec<_>>(),
            vec!["D", "E", "F", "G", "A", "B", "C"],
            "Dorian rooted on D"
        );
    }

    #[test]
    fn mode_of_quality_round_trips() {
        assert_eq!(Key::mode_of_quality(Quality::Major), Mode::Ionian);
        assert_eq!(Key::mode_of_quality(Quality::Minor), Mode::Aeolian);
    }
}
