use absolute::pitch_class::PitchClass;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Mode {
    Ionian,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Locrian,
}

pub const ALL: [Mode; 7] = [
    Mode::Ionian,
    Mode::Dorian,
    Mode::Phrygian,
    Mode::Lydian,
    Mode::Mixolydian,
    Mode::Aeolian,
    Mode::Locrian,
];

const MAJOR_SCALE_INTERVALS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Ionian => "Ionian",
            Mode::Dorian => "Dorian",
            Mode::Phrygian => "Phrygian",
            Mode::Lydian => "Lydian",
            Mode::Mixolydian => "Mixolydian",
            Mode::Aeolian => "Aeolian",
            Mode::Locrian => "Locrian",
        }
    }

    pub fn rotation_index(self) -> usize {
        match self {
            Mode::Ionian => 0,
            Mode::Dorian => 1,
            Mode::Phrygian => 2,
            Mode::Lydian => 3,
            Mode::Mixolydian => 4,
            Mode::Aeolian => 5,
            Mode::Locrian => 6,
        }
    }

    pub fn root_offset_semitones(self) -> i32 {
        MAJOR_SCALE_INTERVALS[self.rotation_index()]
    }

    // A mode is a rotation of the major scale: [root_offset_semitones]
    // indexes into the parent major scale's intervals, so the mode's
    // pitches are the parent scale rotated to start at the mode's root.
    pub fn pitch_classes(self, root: PitchClass) -> Vec<PitchClass> {
        let parent_major_root = root.add(-self.root_offset_semitones());
        let parent_scale: Vec<PitchClass> = MAJOR_SCALE_INTERVALS
            .iter()
            .map(|interval| parent_major_root.add(*interval))
            .collect();
        let idx = self.rotation_index();
        parent_scale
            .iter()
            .skip(idx)
            .chain(parent_scale.iter().take(idx))
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Mode;
    use absolute::pitch_class::PitchClass;

    #[test]
    fn dorian_from_root_matches_rotation() {
        let root = PitchClass::of_int(2);
        assert_eq!(
            Mode::Dorian
                .pitch_classes(root)
                .iter()
                .map(|pc| pc.to_string())
                .collect::<Vec<_>>(),
            vec!["D", "E", "F", "G", "A", "B", "C"],
            "D Dorian"
        );
    }
}
