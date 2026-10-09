#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PitchClass(u8);

const NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

impl PitchClass {
    pub const C: PitchClass = PitchClass(0);
    pub const C_SHARP: PitchClass = PitchClass(1);
    pub const D: PitchClass = PitchClass(2);
    pub const D_SHARP: PitchClass = PitchClass(3);
    pub const E: PitchClass = PitchClass(4);
    pub const F: PitchClass = PitchClass(5);
    pub const F_SHARP: PitchClass = PitchClass(6);
    pub const G: PitchClass = PitchClass(7);
    pub const G_SHARP: PitchClass = PitchClass(8);
    pub const A: PitchClass = PitchClass(9);
    pub const A_SHARP: PitchClass = PitchClass(10);
    pub const B: PitchClass = PitchClass(11);

    pub fn of_int(n: i32) -> PitchClass {
        PitchClass(n.rem_euclid(12) as u8)
    }

    pub fn to_int(self) -> i32 {
        self.0 as i32
    }

    pub fn add(self, semitones: i32) -> PitchClass {
        PitchClass::of_int(self.to_int() + semitones)
    }

    pub fn to_string(self) -> &'static str {
        NAMES[self.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::PitchClass;

    #[test]
    fn pitch_class_wraps_past_b() {
        assert_eq!(
            PitchClass::of_int(11).add(1).to_string(),
            "C",
            "B + 1 semitone"
        );
    }

    #[test]
    fn pitch_class_wraps_below_c_with_negative_semitones() {
        assert_eq!(PitchClass::C.add(-1).to_string(), "B", "C - 1 semitone");
        assert_eq!(
            PitchClass::E.add(-9).to_string(),
            "G",
            "E - 9 semitones (the minor-key parent-major trick)"
        );
    }
}
