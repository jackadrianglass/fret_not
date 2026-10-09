#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Alteration {
    Natural,
    Sharp,
    Flat,
    DoubleSharp,
    DoubleFlat,
}

impl Alteration {
    pub fn semitones(self) -> i32 {
        match self {
            Alteration::Natural => 0,
            Alteration::Sharp => 1,
            Alteration::Flat => -1,
            Alteration::DoubleSharp => 2,
            Alteration::DoubleFlat => -2,
        }
    }

    pub fn of_semitones(semitones: i32) -> Option<Alteration> {
        match semitones {
            0 => Some(Alteration::Natural),
            1 => Some(Alteration::Sharp),
            -1 => Some(Alteration::Flat),
            2 => Some(Alteration::DoubleSharp),
            -2 => Some(Alteration::DoubleFlat),
            _ => None,
        }
    }

    pub fn to_string(self) -> &'static str {
        match self {
            Alteration::Natural => "",
            Alteration::Sharp => "#",
            Alteration::Flat => "b",
            Alteration::DoubleSharp => "##",
            Alteration::DoubleFlat => "bb",
        }
    }
}

// Not derived: declaration order is not semitone order.
impl Ord for Alteration {
    fn cmp(&self, other: &Alteration) -> std::cmp::Ordering {
        self.semitones().cmp(&other.semitones())
    }
}

impl PartialOrd for Alteration {
    fn partial_cmp(&self, other: &Alteration) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::Alteration;
    use std::cmp::Ordering;

    #[test]
    fn semitones_span_two_either_side_of_natural() {
        assert_eq!(
            vec![
                Alteration::DoubleFlat.semitones(),
                Alteration::Flat.semitones(),
                Alteration::Natural.semitones(),
                Alteration::Sharp.semitones(),
                Alteration::DoubleSharp.semitones(),
            ],
            vec![-2, -1, 0, 1, 2],
            "semitones"
        );
    }

    #[test]
    fn to_string_matches_notation() {
        assert_eq!(
            vec![
                Alteration::DoubleFlat.to_string(),
                Alteration::Flat.to_string(),
                Alteration::Natural.to_string(),
                Alteration::Sharp.to_string(),
                Alteration::DoubleSharp.to_string(),
            ],
            vec!["bb", "b", "", "#", "##"],
            "symbols"
        );
    }

    #[test]
    fn compare_orders_by_semitones() {
        assert!(
            Alteration::Flat < Alteration::Natural && Alteration::Sharp > Alteration::Natural,
            "flats sort before natural, sharps after"
        );
        assert_eq!(Alteration::Flat.cmp(&Alteration::Sharp), Ordering::Less);
    }
}
