use std::ops::Deref;

use crate::degree_reference::DegreeReference;
use crate::scale_degree::ScaleDegree;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Slot {
    Rest,
    Note(DegreeReference),
}

// A word is a movable phrase of slots: rest or degree-relative note, all
// of equal duration — slot presence IS the rhythm.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Word(Vec<Slot>);

impl Word {
    pub fn new(slots: Vec<Slot>) -> Word {
        Word(slots)
    }
}

impl Deref for Word {
    type Target = [Slot];

    fn deref(&self) -> &[Slot] {
        &self.0
    }
}

pub fn push(word: &mut Word, slot: Slot) {
    word.0.push(slot);
}

pub fn insert(word: &mut Word, index: usize, slot: Slot) {
    word.0.insert(index, slot);
}

pub fn remove(word: &mut Word, index: usize) -> Slot {
    word.0.remove(index)
}

pub fn set(word: &mut Word, index: usize, slot: Slot) -> Slot {
    std::mem::replace(&mut word.0[index], slot)
}

// Shift every note's octave by [delta]; rests pass through. Interval content
// is preserved — only the register the octaves count from moves.
pub fn shift_octaves(word: &mut Word, delta: i32) {
    for slot in word.0.iter_mut() {
        if let Slot::Note(dr) = slot {
            dr.octave += delta;
        }
    }
}

fn reframe_degree_reference(dr: DegreeReference, root_degree: i32) -> DegreeReference {
    let raw_degree = dr.scale_degree.degree - 1 + (root_degree - 1);
    DegreeReference {
        scale_degree: ScaleDegree::new(raw_degree.rem_euclid(7) + 1, dr.scale_degree.alteration),
        octave: dr.octave + raw_degree.div_euclid(7),
    }
}

// Shifts every note's degree by [root_degree - 1], carrying the octave when
// the degree wraps past 7 — interval content is what's preserved.
pub fn reframe(word: &Word, root_degree: i32) -> Word {
    Word::new(
        word.iter()
            .map(|slot| match slot {
                Slot::Rest => Slot::Rest,
                Slot::Note(dr) => Slot::Note(reframe_degree_reference(*dr, root_degree)),
            })
            .collect(),
    )
}

// One reframed word per root degree, e.g. [1, 4, 5].
pub fn apply_progression(word: &Word, root_degrees: &[i32]) -> Vec<Word> {
    root_degrees
        .iter()
        .map(|root_degree| reframe(word, *root_degree))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Slot, Word, apply_progression, insert, push, reframe, remove, set, shift_octaves};
    use crate::degree_reference::DegreeReference;
    use crate::scale_degree::ScaleDegree;
    use absolute::alteration::Alteration;

    fn note(degree: i32, octave: i32, alteration: Alteration) -> Slot {
        Slot::Note(DegreeReference::new(
            ScaleDegree::new(degree, alteration),
            octave,
        ))
    }

    fn as_tuples(word: &Word) -> Vec<Option<(i32, i32, i32)>> {
        word.iter()
            .map(|slot| match slot {
                Slot::Rest => None,
                Slot::Note(dr) => Some((
                    dr.scale_degree.degree,
                    dr.octave,
                    dr.scale_degree.alteration.semitones(),
                )),
            })
            .collect()
    }

    #[test]
    fn reframing_over_the_tonic_step_is_a_no_op() {
        let word = Word::new(vec![
            note(1, 0, Alteration::Natural),
            Slot::Rest,
            note(1, 0, Alteration::Natural),
        ]);
        assert_eq!(
            as_tuples(&reframe(&word, 1)),
            as_tuples(&word),
            "root degree 1 leaves degree-1 slots unchanged"
        );
    }

    #[test]
    fn reframing_over_the_fourth_shifts_the_tonic_to_degree_four() {
        let word = Word::new(vec![note(1, 0, Alteration::Natural)]);
        assert_eq!(
            as_tuples(&reframe(&word, 4)),
            vec![Some((4, 0, 0))],
            "degree 1 over root degree 4 becomes degree 4"
        );
    }

    #[test]
    fn reframing_carries_an_octave_when_the_degree_wraps_past_seven() {
        let word = Word::new(vec![note(5, 0, Alteration::Natural)]);
        assert_eq!(
            as_tuples(&reframe(&word, 5)),
            vec![Some((2, 1, 0))],
            "degree 5 over root degree 5 wraps to degree 2, one octave higher"
        );
    }

    #[test]
    fn reframing_preserves_alteration_and_existing_octave() {
        let word = Word::new(vec![note(2, 1, Alteration::Flat)]);
        assert_eq!(
            as_tuples(&reframe(&word, 4)),
            vec![Some((5, 1, -1))],
            "alteration and any pre-existing octave carry through untouched"
        );
    }

    #[test]
    fn rests_pass_through_unchanged() {
        let word = Word::new(vec![
            Slot::Rest,
            note(1, 0, Alteration::Natural),
            Slot::Rest,
        ]);
        assert_eq!(
            as_tuples(&reframe(&word, 4)),
            vec![None, Some((4, 0, 0)), None],
            "rests stay rests"
        );
    }

    fn bare_note(degree: i32) -> Slot {
        Slot::Note(DegreeReference::natural(degree, 0))
    }

    fn as_degrees(word: &Word) -> Vec<Option<i32>> {
        word.iter()
            .map(|slot| match slot {
                Slot::Rest => None,
                Slot::Note(dr) => Some(dr.scale_degree.degree),
            })
            .collect()
    }

    #[test]
    fn applying_a_i_iv_v_progression_reinterprets_a_bare_tonic_word() {
        let word = Word::new(vec![bare_note(1)]);
        let progression = [1, 4, 5];
        let reprojected = apply_progression(&word, &progression);
        assert_eq!(
            reprojected.iter().map(as_degrees).collect::<Vec<_>>(),
            vec![vec![Some(1)], vec![Some(4)], vec![Some(5)]],
            "I-IV-V reprojection of a bare tonic word"
        );
    }

    #[test]
    fn apply_to_word_produces_one_word_per_step() {
        let progression = [1, 4, 5];
        assert_eq!(
            apply_progression(&Word::new(vec![bare_note(1)]), &progression).len(),
            3,
            "one reprojected word per progression step"
        );
    }

    #[test]
    fn push_and_remove_edit_the_slot_list() {
        let mut word = Word::new(vec![bare_note(1)]);
        push(&mut word, Slot::Rest);
        assert_eq!(word.len(), 2, "push appends a slot");
        let removed = remove(&mut word, 0);
        assert!(
            matches!(removed, Slot::Note(dr) if dr.scale_degree.degree == 1),
            "remove returns the slot at the index"
        );
    }

    #[test]
    fn insert_places_a_slot_at_the_index() {
        let mut word = Word::new(vec![bare_note(1), bare_note(2)]);
        insert(&mut word, 1, Slot::Rest);
        assert_eq!(
            as_degrees(&word),
            vec![Some(1), None, Some(2)],
            "insert shifts the tail right, growing the word by one"
        );
    }

    #[test]
    fn set_replaces_the_slot_in_place() {
        let mut word = Word::new(vec![bare_note(1), Slot::Rest]);
        let replaced = set(&mut word, 0, Slot::Rest);
        assert!(
            matches!(replaced, Slot::Note(dr) if dr.scale_degree.degree == 1),
            "set returns the slot it replaced"
        );
        assert_eq!(word.len(), 2, "set keeps the slot count");
    }

    #[test]
    fn shift_octaves_moves_notes_and_leaves_rests_alone() {
        let mut word = Word::new(vec![
            note(1, 0, Alteration::Natural),
            Slot::Rest,
            note(2, 1, Alteration::Flat),
        ]);
        shift_octaves(&mut word, 1);
        assert_eq!(
            as_tuples(&word),
            vec![Some((1, 1, 0)), None, Some((2, 2, -1))],
            "every note shifts one octave, rests pass through"
        );
    }
}
