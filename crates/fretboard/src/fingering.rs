use std::ops::Deref;

use crate::fretboard_position::FretboardPosition;
use crate::reach;

// One concrete way to play a sequence of notes: a position per note, in
// note order.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Fingering(Vec<FretboardPosition>);

impl Fingering {
    pub fn new(positions: Vec<FretboardPosition>) -> Fingering {
        Fingering(positions)
    }
}

impl Deref for Fingering {
    type Target = [FretboardPosition];

    fn deref(&self) -> &[FretboardPosition] {
        &self.0
    }
}

// The chunk bridge: the fingering projected back onto slot-aligned
// positions — one entry per slot, None where the slot is a rest. [slot_notes]
// maps each slot to its note's index in the fingering (None for rests).
pub fn slot_positions(
    fingering: &Fingering,
    slot_notes: &[Option<usize>],
) -> Vec<Option<FretboardPosition>> {
    slot_notes
        .iter()
        .map(|note| note.and_then(|index| fingering.get(index).copied()))
        .collect()
}

// The cost of one move: fret distance plus string distance, weighted
// equally — the unit the solver ranks fingerings by.
pub fn distance(a: FretboardPosition, b: FretboardPosition) -> i32 {
    let stretch = reach::between(a, b);
    stretch.fret_span + stretch.string_span
}

// The ranking metric: the total reach walked from [anchor] through the
// fingering, note by note.
pub fn total_distance(anchor: FretboardPosition, fingering: &Fingering) -> i32 {
    let mut total = 0;
    let mut prev = anchor;
    for position in fingering.iter() {
        total += distance(prev, *position);
        prev = *position;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::{Fingering, distance, slot_positions, total_distance};
    use crate::fretboard_position::FretboardPosition;

    fn pos(string_index: usize, fret: i32) -> FretboardPosition {
        FretboardPosition { string_index, fret }
    }

    #[test]
    fn deref_exposes_the_positions() {
        let fingering = Fingering::new(vec![pos(0, 0), pos(2, 4)]);
        assert_eq!(fingering.len(), 2, "one position per note");
        assert_eq!(fingering[1], pos(2, 4), "in note order");
    }

    #[test]
    fn slot_positions_align_notes_and_leave_rests_none() {
        let fingering = Fingering::new(vec![pos(1, 3), pos(0, 3)]);
        assert_eq!(
            slot_positions(&fingering, &[None, Some(0), Some(1)]),
            vec![None, Some(pos(1, 3)), Some(pos(0, 3))],
            "rests project to None, notes to their positions, in slot order"
        );
    }

    #[test]
    fn total_distance_walks_the_reach_from_the_anchor() {
        let fingering = Fingering::new(vec![pos(0, 3), pos(2, 5)]);
        assert_eq!(
            total_distance(pos(0, 0), &fingering),
            distance(pos(0, 0), pos(0, 3)) + distance(pos(0, 3), pos(2, 5)),
            "the anchor's reach to the first note plus each move between notes"
        );
    }
}
