use crate::degree_reference::DegreeReference;
use crate::scale_degree::ScaleDegree;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Slot {
    Rest,
    Note(DegreeReference),
}

pub type Chunk = Vec<Slot>;

fn reframe_degree_reference(dr: DegreeReference, root_degree: i32) -> DegreeReference {
    let raw_degree = dr.scale_degree.degree - 1 + (root_degree - 1);
    DegreeReference {
        scale_degree: ScaleDegree::create(raw_degree.rem_euclid(7) + 1, dr.scale_degree.alteration),
        octave: dr.octave + raw_degree.div_euclid(7),
    }
}

// Shifts every note's degree by [root_degree - 1], carrying the octave when
// the degree wraps past 7 — interval content is what's preserved.
pub fn reframe(chunk: &Chunk, root_degree: i32) -> Chunk {
    chunk
        .iter()
        .map(|slot| match slot {
            Slot::Rest => Slot::Rest,
            Slot::Note(dr) => Slot::Note(reframe_degree_reference(*dr, root_degree)),
        })
        .collect()
}

// One reframed chunk per root degree, e.g. [1, 4, 5].
pub fn apply_progression(chunk: &Chunk, root_degrees: &[i32]) -> Vec<Chunk> {
    root_degrees
        .iter()
        .map(|root_degree| reframe(chunk, *root_degree))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Chunk, Slot, apply_progression, reframe};
    use crate::degree_reference::DegreeReference;
    use crate::scale_degree::ScaleDegree;
    use absolute::alteration::Alteration;

    fn note(degree: i32, octave: i32, alteration: Alteration) -> Slot {
        Slot::Note(DegreeReference::create(
            ScaleDegree::create(degree, alteration),
            octave,
        ))
    }

    fn as_tuples(chunk: &Chunk) -> Vec<Option<(i32, i32, i32)>> {
        chunk
            .iter()
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
        let chunk = vec![
            note(1, 0, Alteration::Natural),
            Slot::Rest,
            note(1, 0, Alteration::Natural),
        ];
        assert_eq!(
            as_tuples(&reframe(&chunk, 1)),
            as_tuples(&chunk),
            "root degree 1 leaves degree-1 slots unchanged"
        );
    }

    #[test]
    fn reframing_over_the_fourth_shifts_the_tonic_to_degree_four() {
        let chunk = vec![note(1, 0, Alteration::Natural)];
        assert_eq!(
            as_tuples(&reframe(&chunk, 4)),
            vec![Some((4, 0, 0))],
            "degree 1 over root degree 4 becomes degree 4"
        );
    }

    #[test]
    fn reframing_carries_an_octave_when_the_degree_wraps_past_seven() {
        let chunk = vec![note(5, 0, Alteration::Natural)];
        assert_eq!(
            as_tuples(&reframe(&chunk, 5)),
            vec![Some((2, 1, 0))],
            "degree 5 over root degree 5 wraps to degree 2, one octave higher"
        );
    }

    #[test]
    fn reframing_preserves_alteration_and_existing_octave() {
        let chunk = vec![note(2, 1, Alteration::Flat)];
        assert_eq!(
            as_tuples(&reframe(&chunk, 4)),
            vec![Some((5, 1, -1))],
            "alteration and any pre-existing octave carry through untouched"
        );
    }

    #[test]
    fn rests_pass_through_unchanged() {
        let chunk = vec![Slot::Rest, note(1, 0, Alteration::Natural), Slot::Rest];
        assert_eq!(
            as_tuples(&reframe(&chunk, 4)),
            vec![None, Some((4, 0, 0)), None],
            "rests stay rests"
        );
    }

    fn bare_note(degree: i32) -> Slot {
        Slot::Note(DegreeReference::natural(degree, 0))
    }

    fn as_degrees(chunk: &Chunk) -> Vec<Option<i32>> {
        chunk
            .iter()
            .map(|slot| match slot {
                Slot::Rest => None,
                Slot::Note(dr) => Some(dr.scale_degree.degree),
            })
            .collect()
    }

    #[test]
    fn applying_a_i_iv_v_progression_reinterprets_a_bare_tonic_chunk() {
        let chunk = vec![bare_note(1)];
        let progression = [1, 4, 5];
        let reprojected = apply_progression(&chunk, &progression);
        assert_eq!(
            reprojected.iter().map(as_degrees).collect::<Vec<_>>(),
            vec![vec![Some(1)], vec![Some(4)], vec![Some(5)]],
            "I-IV-V reprojection of a bare tonic chunk"
        );
    }

    #[test]
    fn apply_to_chunk_produces_one_chunk_per_step() {
        let progression = [1, 4, 5];
        assert_eq!(
            apply_progression(&vec![bare_note(1)], &progression).len(),
            3,
            "one reprojected chunk per progression step"
        );
    }
}
