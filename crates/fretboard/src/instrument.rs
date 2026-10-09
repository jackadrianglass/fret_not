use absolute::pitch_class::PitchClass;

use crate::fretboard_position::FretboardPosition;
use crate::tuning::{self, Tuning};

// A physical instrument: a tuning plus the frets actually available on each
// string. Everything that asks "can this be played here" goes through an
// instrument.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Instrument {
    pub tuning: Tuning,
    pub fret_count: Vec<i32>,
}

impl Instrument {
    pub fn create(tuning: Tuning, fret_count: Vec<i32>) -> Instrument {
        if fret_count.len() != tuning::string_count(&tuning) {
            panic!("Instrument::create: one fret count per string is required");
        }
        Instrument { tuning, fret_count }
    }

    pub fn create_uniform(tuning: Tuning, fret_count: i32) -> Instrument {
        let count = tuning::string_count(&tuning);
        Instrument {
            tuning,
            fret_count: vec![fret_count; count],
        }
    }

    pub fn string_count(&self) -> usize {
        tuning::string_count(&self.tuning)
    }

    pub fn fret_count(&self, string_index: usize) -> i32 {
        self.fret_count[string_index]
    }

    pub fn max_fret(&self) -> i32 {
        *self.fret_count.iter().max().unwrap()
    }

    pub fn playable(&self, position: FretboardPosition) -> bool {
        position.string_index < self.string_count()
            && position.fret >= 0
            && position.fret <= self.fret_count(position.string_index)
    }

    pub fn semitone_at(&self, position: FretboardPosition) -> i32 {
        tuning::semitone_at(&self.tuning, position.string_index, position.fret)
    }

    pub fn pitch_class_at(&self, position: FretboardPosition) -> PitchClass {
        PitchClass::of_int(self.semitone_at(position))
    }

    // Lowest and highest sounding semitone reachable on the instrument.
    pub fn range(&self) -> (i32, i32) {
        let lowest = tuning::open_semitone(&self.tuning, 0);
        let last = self.string_count() - 1;
        let highest = tuning::open_semitone(&self.tuning, last) + self.fret_count(last);
        (lowest, highest)
    }
}

#[cfg(test)]
mod tests {
    use super::Instrument;
    use crate::fretboard_position::FretboardPosition;
    use crate::tuning::{self, Tuning};

    fn position(string_index: usize, fret: i32) -> FretboardPosition {
        FretboardPosition { string_index, fret }
    }

    fn standard_instrument() -> Instrument {
        Instrument::create_uniform(tuning::standard(), 22)
    }

    #[test]
    #[should_panic(expected = "Instrument::create: one fret count per string is required")]
    fn create_rejects_a_fret_count_per_string_mismatch() {
        let tuning: Tuning = tuning::standard();
        Instrument::create(tuning, vec![22, 22, 22, 22, 22]);
    }

    #[test]
    fn playable_respects_per_string_fret_counts() {
        let uneven = Instrument::create(tuning::standard(), vec![22, 22, 22, 22, 22, 24]);
        assert!(uneven.playable(position(5, 24)), "high E fret 24 playable");
        assert!(
            !uneven.playable(position(4, 24)),
            "B string fret 24 not playable"
        );
        assert!(
            !uneven.playable(position(6, 0)),
            "beyond the string count not playable"
        );
        assert!(
            !uneven.playable(position(0, -1)),
            "negative fret not playable"
        );
    }

    #[test]
    fn max_fret_is_the_widest_string() {
        let uneven = Instrument::create(tuning::standard(), vec![22, 22, 22, 22, 22, 24]);
        assert_eq!(uneven.max_fret(), 24, "max fret");
    }

    #[test]
    fn semitone_at_resolves_absolute_pitch() {
        let instrument = standard_instrument();
        assert_eq!(
            instrument.semitone_at(position(0, 0)),
            40,
            "open low E is E2"
        );
        assert_eq!(
            instrument.semitone_at(position(5, 12)),
            76,
            "high E fret 12 is E5"
        );
    }

    #[test]
    fn range_spans_the_reachable_semitones() {
        assert_eq!(
            standard_instrument().range(),
            (40, 86),
            "E2 open to E6 at fret 22"
        );
    }
}
