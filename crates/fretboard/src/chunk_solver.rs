use relative::degree_reference::DegreeReference;
use relative::key::Key;
use relative::mode::Mode;

use crate::fingering::{self, Fingering};
use crate::fretboard;
use crate::fretboard_position::FretboardPosition;
use crate::instrument::Instrument;

fn root_semitone(key: Key) -> i32 {
    fretboard::root_note(key).semitone()
}

fn base_interval(key: Key, mode: Mode, dr: DegreeReference) -> i32 {
    DegreeReference { octave: 0, ..dr }.interval(key.tonic_pitch_class(), mode)
}

fn first_note_octaves(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    dr: DegreeReference,
) -> Vec<i32> {
    let (lowest, highest) = instrument.range();
    let base = base_interval(key, mode, dr);
    let root = root_semitone(key);
    let octave_lo = -(-(lowest - root - base)).div_euclid(12);
    let octave_hi = (highest - root - base).div_euclid(12);
    if octave_lo > octave_hi {
        Vec::new()
    } else {
        (octave_lo..=octave_hi).collect()
    }
}

fn first_note_candidates(
    key: Key,
    mode: Mode,
    instrument: &Instrument,
    start_anchor: FretboardPosition,
    max_fret_distance: i32,
    dr: DegreeReference,
) -> Vec<FretboardPosition> {
    first_note_octaves(instrument, key, mode, dr)
        .into_iter()
        .flat_map(|octave| {
            fretboard::degree_positions(instrument, key, mode, DegreeReference { octave, ..dr })
        })
        .filter(|p| (p.fret - start_anchor.fret).abs() <= max_fret_distance)
        .collect()
}

fn candidates_for_semitone(
    instrument: &Instrument,
    near: FretboardPosition,
    max_fret_distance: i32,
    target_semitone: i32,
) -> Vec<FretboardPosition> {
    fretboard::positions_of_semitone(instrument, target_semitone)
        .into_iter()
        .filter(|p| (p.fret - near.fret).abs() <= max_fret_distance)
        .collect()
}

fn chain(
    key: Key,
    mode: Mode,
    instrument: &Instrument,
    max_fret_distance: i32,
    offset: i32,
    anchor: FretboardPosition,
    rest: &[DegreeReference],
) -> Vec<Vec<FretboardPosition>> {
    match rest {
        [] => vec![Vec::new()],
        [first, tail @ ..] => {
            let target = fretboard::degree_note(key, mode, *first).semitone() + offset;
            candidates_for_semitone(instrument, anchor, max_fret_distance, target)
                .into_iter()
                .flat_map(|position| {
                    chain(
                        key,
                        mode,
                        instrument,
                        max_fret_distance,
                        offset,
                        position,
                        tail,
                    )
                    .into_iter()
                    .map(move |tail_positions| {
                        let mut positions = Vec::with_capacity(tail_positions.len() + 1);
                        positions.push(position);
                        positions.extend(tail_positions);
                        positions
                    })
                })
                .collect()
        }
    }
}

// Every fingering continuing from [origin], which already voices [first]:
// the rest chain at the register offset [origin] implies, prepended with
// [origin] itself.
fn anchored(
    key: Key,
    mode: Mode,
    instrument: &Instrument,
    max_fret_distance: i32,
    origin: FretboardPosition,
    first: DegreeReference,
    rest: &[DegreeReference],
) -> Vec<Fingering> {
    let offset =
        instrument.semitone_at(origin) - fretboard::degree_note(key, mode, first).semitone();
    chain(
        key,
        mode,
        instrument,
        max_fret_distance,
        offset,
        origin,
        rest,
    )
    .into_iter()
    .map(|mut tail| {
        let mut with_first = Vec::with_capacity(tail.len() + 1);
        with_first.push(origin);
        with_first.append(&mut tail);
        Fingering::new(with_first)
    })
    .collect()
}

pub fn positions(
    key: Key,
    mode: Mode,
    instrument: &Instrument,
    start_anchor: FretboardPosition,
    max_fret_distance: i32,
    notes: &[DegreeReference],
) -> Vec<Fingering> {
    match notes {
        [] => vec![Fingering::new(Vec::new())],
        [first, rest @ ..] => {
            // Resolve each note's absolute semitone (via degree_note) FIRST,
            // then choose only *which string* — skipping this broke every
            // non-tonic chord.
            let mut shapes: Vec<Fingering> = first_note_candidates(
                key,
                mode,
                instrument,
                start_anchor,
                max_fret_distance,
                *first,
            )
            .into_iter()
            .flat_map(|position| {
                anchored(
                    key,
                    mode,
                    instrument,
                    max_fret_distance,
                    position,
                    *first,
                    rest,
                )
            })
            .collect();
            shapes.sort_by_key(|fingering| fingering::total_distance(start_anchor, fingering));
            shapes
        }
    }
}

// Every fingering whose first note is played exactly at [first]; the rest
// enumerate within the cap of it, ranked by reach from it.
pub fn positions_from(
    key: Key,
    mode: Mode,
    instrument: &Instrument,
    first: FretboardPosition,
    max_fret_distance: i32,
    notes: &[DegreeReference],
) -> Vec<Fingering> {
    match notes {
        [] => vec![Fingering::new(Vec::new())],
        [first_note, rest @ ..] => {
            let mut shapes = anchored(
                key,
                mode,
                instrument,
                max_fret_distance,
                first,
                *first_note,
                rest,
            );
            shapes.sort_by_key(|fingering| fingering::total_distance(first, fingering));
            shapes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{positions, positions_from};
    use crate::fingering::{Fingering, distance};
    use crate::fretboard_position::FretboardPosition;
    use crate::instrument::Instrument;
    use crate::tuning;
    use absolute::letter::Letter;
    use absolute::spelled_pitch::SpelledPitch;
    use relative::degree_reference::DegreeReference;
    use relative::key::{Key, Quality};
    use relative::mode::Mode;

    fn c_major() -> Key {
        Key::new(SpelledPitch::natural(Letter::C), Quality::Major)
    }

    fn instrument() -> Instrument {
        Instrument::new_uniform(tuning::standard(), 22)
    }

    fn low_e_fret_8() -> FretboardPosition {
        FretboardPosition {
            string_index: 0,
            fret: 8,
        }
    }

    fn natural_degrees(degrees_and_octaves: &[(i32, i32)]) -> Vec<DegreeReference> {
        degrees_and_octaves
            .iter()
            .map(|(degree, octave)| DegreeReference::natural(*degree, *octave))
            .collect()
    }

    fn semitone_of(position: FretboardPosition) -> i32 {
        instrument().semitone_at(position)
    }

    #[test]
    fn distance_is_fret_delta_plus_string_delta() {
        assert_eq!(
            distance(
                FretboardPosition {
                    string_index: 0,
                    fret: 3
                },
                FretboardPosition {
                    string_index: 2,
                    fret: 6
                }
            ),
            5,
            "3 frets + 2 strings apart"
        );
    }

    fn root_third_fifth() -> Vec<DegreeReference> {
        natural_degrees(&[(1, 0), (3, 0), (5, 0)])
    }

    #[test]
    fn root_third_fifth_ascends_within_the_fret_cap() {
        let shapes = positions(
            c_major(),
            Mode::Ionian,
            &instrument(),
            low_e_fret_8(),
            7,
            &root_third_fifth(),
        );
        assert!(!shapes.is_empty(), "at least one valid shape found");
        let relative_semitones: Vec<i32> = shapes[0].iter().map(|p| semitone_of(*p)).collect();
        let mut sorted = relative_semitones.clone();
        sorted.sort();
        assert!(
            sorted == relative_semitones
                && relative_semitones
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    == relative_semitones.len(),
            "strictly ascending pitch"
        );
    }

    #[test]
    fn every_consecutive_move_stays_within_the_fret_cap() {
        let shapes = positions(
            c_major(),
            Mode::Ionian,
            &instrument(),
            low_e_fret_8(),
            7,
            &root_third_fifth(),
        );
        for shape in &shapes {
            let mut prev = low_e_fret_8();
            for position in shape.iter() {
                assert!(
                    (position.fret - prev.fret).abs() <= 7,
                    "no move exceeds the fret cap"
                );
                prev = *position;
            }
        }
    }

    #[test]
    fn shapes_are_sorted_by_ascending_total_distance() {
        let shapes = positions(
            c_major(),
            Mode::Ionian,
            &instrument(),
            low_e_fret_8(),
            7,
            &natural_degrees(&[(1, 0), (5, 0)]),
        );
        let totals: Vec<i32> = shapes
            .iter()
            .map(|shape| {
                let mut total = 0;
                let mut prev = low_e_fret_8();
                for position in shape.iter() {
                    total += distance(prev, *position);
                    prev = *position;
                }
                total
            })
            .collect();
        let mut sorted = totals.clone();
        sorted.sort();
        assert_eq!(sorted, totals, "totals non-decreasing");
    }

    #[test]
    fn an_impossibly_tight_cap_finds_nothing() {
        // A 0-fret cap pins every note to the exact same fret column across
        // all 6 strings — at most 6 distinct pitch classes are reachable
        // there, so all 7 diatonic degrees can never simultaneously fit.
        let degrees = natural_degrees(&[(1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]);
        let shapes = positions(
            c_major(),
            Mode::Ionian,
            &instrument(),
            low_e_fret_8(),
            0,
            &degrees,
        );
        assert!(
            shapes.is_empty(),
            "no shape fits all 7 degrees into one fret column"
        );
    }

    fn intervals_of(shape: &[FretboardPosition]) -> Vec<i32> {
        let relative_semitones: Vec<i32> = shape.iter().map(|p| semitone_of(*p)).collect();
        relative_semitones
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect()
    }

    #[test]
    fn a_reframed_wrapped_octave_still_lands_a_real_minor_third_up() {
        // [1; 3; 5] reframed onto root degree 6 (reframe: degree 3 wraps to
        // degree 1, octave+1; degree 5 wraps to degree 3, octave+1) should
        // still resolve to A-C-E — a real minor third then a real major
        // third, both ascending — not the wrapped octave being taken
        // literally and landing a full 12 semitones higher than that.
        let degrees = natural_degrees(&[(6, 0), (1, 1), (3, 1)]);
        let shapes = positions(
            c_major(),
            Mode::Ionian,
            &instrument(),
            low_e_fret_8(),
            7,
            &degrees,
        );
        assert!(!shapes.is_empty(), "at least one valid shape found");
        assert_eq!(
            intervals_of(&shapes[0]),
            vec![3, 4],
            "minor third then major third, both ascending"
        );
    }

    #[test]
    fn an_authored_octave_leap_is_preserved() {
        // degree 1 at octave 0 then octave 1 is a full octave leap by
        // construction — the solver must realize that leap exactly (12
        // semitones), not silently resolve the second note to whatever's
        // nearest within the fret cap.
        let degrees = natural_degrees(&[(1, 0), (1, 1)]);
        let shapes = positions(
            c_major(),
            Mode::Ionian,
            &instrument(),
            low_e_fret_8(),
            7,
            &degrees,
        );
        assert!(!shapes.is_empty(), "at least one valid shape found");
        assert_eq!(
            intervals_of(&shapes[0]),
            vec![12],
            "a full octave, ascending"
        );
    }

    #[test]
    fn pinned_positions_keep_the_first_note_at_its_position() {
        // The first note pinned at C4 on the B string's first fret; the
        // fifth (G4) is reachable at the high E's 3rd fret or the B
        // string's 8th within a cap of 8, ranked by reach from the pin.
        let shapes = positions_from(
            c_major(),
            Mode::Ionian,
            &instrument(),
            FretboardPosition {
                string_index: 4,
                fret: 1,
            },
            8,
            &natural_degrees(&[(1, 0), (5, 0)]),
        );
        assert_eq!(
            shapes,
            vec![
                Fingering::new(vec![
                    FretboardPosition {
                        string_index: 4,
                        fret: 1
                    },
                    FretboardPosition {
                        string_index: 5,
                        fret: 3
                    },
                ]),
                Fingering::new(vec![
                    FretboardPosition {
                        string_index: 4,
                        fret: 1
                    },
                    FretboardPosition {
                        string_index: 4,
                        fret: 8
                    },
                ]),
            ],
            "the pinned note leads every fingering; the rest rank by reach from it"
        );
    }
}
