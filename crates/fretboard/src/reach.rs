use crate::fretboard_position::FretboardPosition;
use crate::instrument::Instrument;

// Finger-stretch geometry between positions. The fret span is the physical
// stretch along the neck; the string span is the spread across strings. The
// sounding span between two positions also depends on the tuning's
// cross-string intervals, not just the fret delta. The derived compare
// orders by fret span first — it dominates the physical difficulty of a
// stretch — then by string span.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Reach {
    pub fret_span: i32,
    pub string_span: i32,
}

pub fn between(a: FretboardPosition, b: FretboardPosition) -> Reach {
    Reach {
        fret_span: (a.fret - b.fret).abs(),
        string_span: (a.string_index as i32 - b.string_index as i32).abs(),
    }
}

// The bounding box of a whole shape; None for no positions.
pub fn of_positions(positions: &[FretboardPosition]) -> Option<Reach> {
    let frets = positions.iter().map(|p| p.fret);
    let strings = positions.iter().map(|p| p.string_index);
    let min_fret = frets.clone().min()?;
    let max_fret = frets.max()?;
    let min_string = strings.clone().min()?;
    let max_string = strings.max()?;
    Some(Reach {
        fret_span: max_fret - min_fret,
        string_span: (max_string - min_string) as i32,
    })
}

pub fn semitone_span(instrument: &Instrument, a: FretboardPosition, b: FretboardPosition) -> i32 {
    (instrument.semitone_at(a) - instrument.semitone_at(b)).abs()
}

#[cfg(test)]
mod tests {
    use super::{between, of_positions, semitone_span};
    use crate::fretboard_position::FretboardPosition;
    use crate::instrument::Instrument;
    use crate::tuning;

    fn position(string_index: usize, fret: i32) -> FretboardPosition {
        FretboardPosition { string_index, fret }
    }

    fn standard_instrument() -> Instrument {
        Instrument::create_uniform(tuning::standard(), 22)
    }

    #[test]
    fn between_measures_fret_and_string_spans() {
        let reach = between(position(0, 3), position(2, 6));
        assert_eq!(reach.fret_span, 3, "fret span");
        assert_eq!(reach.string_span, 2, "string span");
    }

    #[test]
    fn of_positions_bounding_boxes_the_shape() {
        let reach = of_positions(&[position(0, 3), position(2, 6), position(1, 0)]).unwrap();
        assert_eq!(reach.fret_span, 6, "fret span 0 to 6");
        assert_eq!(reach.string_span, 2, "string span 0 to 2");
        assert!(of_positions(&[]).is_none(), "no positions is None");
    }

    #[test]
    fn semitone_span_accounts_for_cross_string_intervals() {
        // Same 10-fret gap, but across different strings the sounding
        // interval differs by the tuning's interval between the open
        // strings.
        let instrument = standard_instrument();
        assert_eq!(
            semitone_span(&instrument, position(0, 0), position(0, 10)),
            10,
            "same string is the fret delta"
        );
        assert_eq!(
            semitone_span(&instrument, position(0, 0), position(2, 5)),
            15,
            "low E fret 0 to D fret 5 is 15 semitones"
        );
        assert_eq!(
            semitone_span(&instrument, position(0, 8), position(1, 3)),
            0,
            "low E fret 8 to A fret 3 sounds the same note"
        );
    }
}
