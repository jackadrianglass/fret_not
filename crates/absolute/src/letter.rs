use std::fmt;

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

pub const LETTERS: [Letter; 7] = [
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

    pub fn from_index(i: i32) -> Letter {
        LETTERS[i.rem_euclid(7) as usize]
    }

    pub fn offset(self, n: i32) -> Letter {
        Letter::from_index(self.index() as i32 + n)
    }

    pub fn as_str(self) -> &'static str {
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

impl From<Letter> for PitchClass {
    fn from(letter: Letter) -> PitchClass {
        match letter {
            Letter::C => PitchClass::C,
            Letter::D => PitchClass::D,
            Letter::E => PitchClass::E,
            Letter::F => PitchClass::F,
            Letter::G => PitchClass::G,
            Letter::A => PitchClass::A,
            Letter::B => PitchClass::B,
        }
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::{LETTERS, Letter};
    use crate::pitch_class::PitchClass;

    #[test]
    fn naturals_are_not_the_full_chromatic_set() {
        assert_eq!(
            LETTERS
                .iter()
                .map(|letter| i32::from(PitchClass::from(*letter)))
                .collect::<Vec<_>>(),
            vec![0, 2, 4, 5, 7, 9, 11],
            "natural letters"
        );
    }

    #[test]
    fn offset_wraps_around_the_letter_cycle() {
        assert_eq!(
            (1..=6)
                .map(|i| Letter::F.offset(i).as_str())
                .collect::<Vec<_>>(),
            vec!["G", "A", "B", "C", "D", "E"],
            "F stepped forward six letters"
        );
    }

    #[test]
    fn negative_offset_also_wraps() {
        assert_eq!(
            Letter::C.offset(-1).as_str(),
            "B",
            "C stepped back one letter"
        );
    }

    #[test]
    fn from_index_round_trips_through_index() {
        for letter in LETTERS {
            assert_eq!(
                letter,
                Letter::from_index(letter.index() as i32),
                "{} round trips",
                letter.as_str()
            );
        }
    }
}
