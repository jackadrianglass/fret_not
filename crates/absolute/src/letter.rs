use crate::pitch_class::PitchClass;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

pub const ALL: [Letter; 7] = [
    Letter::C,
    Letter::D,
    Letter::E,
    Letter::F,
    Letter::G,
    Letter::A,
    Letter::B,
];

impl Letter {
    pub fn index(self) -> usize {
        match self {
            Letter::C => 0,
            Letter::D => 1,
            Letter::E => 2,
            Letter::F => 3,
            Letter::G => 4,
            Letter::A => 5,
            Letter::B => 6,
        }
    }

    pub fn of_index(i: i32) -> Letter {
        ALL[i.rem_euclid(7) as usize]
    }

    pub fn offset(self, n: i32) -> Letter {
        Letter::of_index(self.index() as i32 + n)
    }

    pub fn to_pitch_class(self) -> PitchClass {
        match self {
            Letter::C => PitchClass::C,
            Letter::D => PitchClass::D,
            Letter::E => PitchClass::E,
            Letter::F => PitchClass::F,
            Letter::G => PitchClass::G,
            Letter::A => PitchClass::A,
            Letter::B => PitchClass::B,
        }
    }

    pub fn to_string(self) -> &'static str {
        match self {
            Letter::C => "C",
            Letter::D => "D",
            Letter::E => "E",
            Letter::F => "F",
            Letter::G => "G",
            Letter::A => "A",
            Letter::B => "B",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ALL, Letter};

    #[test]
    fn naturals_are_not_the_full_chromatic_set() {
        assert_eq!(
            ALL.iter()
                .map(|letter| letter.to_pitch_class().to_int())
                .collect::<Vec<_>>(),
            vec![0, 2, 4, 5, 7, 9, 11],
            "natural letters"
        );
    }

    #[test]
    fn offset_wraps_around_the_letter_cycle() {
        assert_eq!(
            (1..=6)
                .map(|i| Letter::F.offset(i).to_string())
                .collect::<Vec<_>>(),
            vec!["G", "A", "B", "C", "D", "E"],
            "F stepped forward six letters"
        );
    }

    #[test]
    fn negative_offset_also_wraps() {
        assert_eq!(
            Letter::C.offset(-1).to_string(),
            "B",
            "C stepped back one letter"
        );
    }

    #[test]
    fn of_index_round_trips_through_index() {
        for letter in ALL {
            assert_eq!(
                letter,
                Letter::of_index(letter.index() as i32),
                "{} round trips",
                letter.to_string()
            );
        }
    }
}
