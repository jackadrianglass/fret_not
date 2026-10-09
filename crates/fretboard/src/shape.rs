use absolute::pitch_class::PitchClass;
use relative::degree_reference::DegreeReference;
use relative::key::Key;
use relative::mode::Mode;
use relative::scale_degree::ScaleDegree;

use crate::fretboard_position::FretboardPosition;
use crate::instrument::Instrument;
use crate::tuning;

// Notes-per-string shapes over an arbitrary degree list. The degree list
// drives everything: the diatonic degrees give the seven 3-notes-per-string
// positions, Pentatonic's degrees give the five 2-notes-per-string
// positions, Arpeggio's the three 1-note-per-string inversions.

// The cyclic degree sequence starting [start] notes into the cycle, with the
// octave carrying once per cycle. Empty degrees give no notes.
pub fn scale_notes(degrees: &[ScaleDegree], start: i32, count: i32) -> Vec<DegreeReference> {
    let cycle = degrees.len() as i32;
    if cycle == 0 {
        return Vec::new();
    }
    (0..count)
        .map(|i| {
            let index = start + i;
            DegreeReference {
                scale_degree: degrees[index.rem_euclid(cycle) as usize],
                octave: index.div_euclid(cycle),
            }
        })
        .collect()
}

fn start_semitone(instrument: &Instrument, key: Key, anchor: FretboardPosition) -> i32 {
    let anchor_semitone = instrument.semitone_at(anchor);
    anchor_semitone
        + (key.tonic_pitch_class().to_int() - PitchClass::of_int(anchor_semitone).to_int())
            .rem_euclid(12)
}

fn drop_to_lowest_playable_octave(frets: Vec<i32>) -> Vec<i32> {
    let min_fret = *frets.iter().min().unwrap();
    let octave_drop = min_fret.max(0) / 12 * 12;
    frets.into_iter().map(|fret| fret - octave_drop).collect()
}

// One shape per degree: shape k starts [k] scale notes above the anchor's
// key tonic occurrence, laid out [notes_per_string] notes per string
// ascending. Each shape is dropped to its lowest playable octave; shapes
// that still leave unplayable frets are omitted.
pub fn positions(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    degrees: &[ScaleDegree],
    notes_per_string: i32,
    anchor: FretboardPosition,
) -> Vec<Vec<FretboardPosition>> {
    let string_count = instrument.string_count() as i32;
    let shape_count = degrees.len() as i32;
    let note_count = string_count * notes_per_string;
    (0..shape_count)
        .map(|shape_start| {
            let notes = scale_notes(degrees, shape_start, note_count);
            let frets: Vec<i32> = notes
                .iter()
                .enumerate()
                .map(|(note_index, dr)| {
                    let string_index = (note_index as i32 / notes_per_string) as usize;
                    let semitone = start_semitone(instrument, key, anchor)
                        + dr.interval(key.tonic_pitch_class(), mode);
                    semitone - tuning::open_semitone(&instrument.tuning, string_index)
                })
                .collect();
            let dropped = drop_to_lowest_playable_octave(frets);
            dropped
                .iter()
                .enumerate()
                .map(|(note_index, fret)| FretboardPosition {
                    string_index: (note_index as i32 / notes_per_string) as usize,
                    fret: *fret,
                })
                .collect()
        })
        .filter(|shape: &Vec<FretboardPosition>| shape.iter().all(|p| instrument.playable(*p)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{positions, scale_notes};
    use crate::fretboard_position::FretboardPosition;
    use crate::instrument::Instrument;
    use crate::tuning;
    use absolute::letter::Letter;
    use absolute::spelled_pitch::SpelledPitch;
    use relative::key::{Key, Quality};
    use relative::mode::Mode;
    use relative::scale_degree::ScaleDegree;
    use relative::scales;

    fn c_major() -> Key {
        Key::create(SpelledPitch::natural(Letter::C), Quality::Major)
    }

    fn ionian() -> Mode {
        Mode::Ionian
    }

    fn open_low_e() -> FretboardPosition {
        FretboardPosition {
            string_index: 0,
            fret: 0,
        }
    }

    fn instrument() -> Instrument {
        Instrument::create_uniform(tuning::standard(), 22)
    }

    fn as_pairs(shape: &[FretboardPosition]) -> Vec<(usize, i32)> {
        shape.iter().map(|p| (p.string_index, p.fret)).collect()
    }

    #[test]
    fn scale_notes_cycles_degrees_with_octave_carry() {
        let notes = scale_notes(&ScaleDegree::diatonic(), 0, 9);
        assert_eq!(
            notes
                .iter()
                .map(|dr| dr.scale_degree.degree)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6, 7, 1, 2],
            "degrees cycle 1..7,1,2"
        );
        assert_eq!(
            notes.iter().map(|dr| dr.octave).collect::<Vec<_>>(),
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1],
            "the octave carries once per cycle"
        );
    }

    #[test]
    fn scale_notes_starts_midway_through_the_cycle() {
        let notes = scale_notes(&ScaleDegree::diatonic(), 5, 3);
        assert_eq!(
            notes
                .iter()
                .map(|dr| dr.scale_degree.degree)
                .collect::<Vec<_>>(),
            vec![6, 7, 1],
            "degrees 6,7,1"
        );
        assert_eq!(
            notes.iter().map(|dr| dr.octave).collect::<Vec<_>>(),
            vec![0, 0, 1],
            "the carried degree keeps its octave"
        );
    }

    #[test]
    fn scale_notes_of_empty_degrees_is_empty() {
        assert!(scale_notes(&[], 0, 5).is_empty(), "no degrees, no notes");
    }

    fn diatonic_shapes() -> Vec<Vec<FretboardPosition>> {
        positions(
            &instrument(),
            c_major(),
            ionian(),
            &ScaleDegree::diatonic(),
            3,
            open_low_e(),
        )
    }

    #[test]
    fn diatonic_three_notes_per_string_shape_one_matches_the_known_shape() {
        let shapes = diatonic_shapes();
        assert_eq!(shapes.len(), 7, "seven shapes, one per degree");
        assert_eq!(
            as_pairs(&shapes[0]),
            vec![
                (0, 8),
                (0, 10),
                (0, 12),
                (1, 8),
                (1, 10),
                (1, 12),
                (2, 9),
                (2, 10),
                (2, 12),
                (3, 9),
                (3, 10),
                (3, 12),
                (4, 10),
                (4, 12),
                (4, 13),
                (5, 10),
                (5, 12),
                (5, 13),
            ],
            "C major Ionian shape 1"
        );
    }

    #[test]
    fn diatonic_shape_three_is_dropped_to_its_lowest_playable_octave() {
        let shapes = diatonic_shapes();
        assert_eq!(
            as_pairs(&shapes[2]),
            vec![
                (0, 0),
                (0, 1),
                (0, 3),
                (1, 0),
                (1, 2),
                (1, 3),
                (2, 0),
                (2, 2),
                (2, 3),
                (3, 0),
                (3, 2),
                (3, 4),
                (4, 1),
                (4, 3),
                (4, 5),
                (5, 1),
                (5, 3),
                (5, 5),
            ],
            "shape 3, dropped an octave to stay on the neck"
        );
    }

    #[test]
    fn every_shape_is_playable_on_the_instrument() {
        assert!(
            diatonic_shapes().iter().all(|shape| {
                shape
                    .iter()
                    .all(|position| instrument().playable(*position))
            }),
            "every position playable"
        );
    }

    #[test]
    fn shapes_reaching_past_the_fret_limit_are_omitted() {
        let three_fret_instrument = Instrument::create_uniform(tuning::standard(), 3);
        let shapes = positions(
            &three_fret_instrument,
            c_major(),
            ionian(),
            &ScaleDegree::diatonic(),
            3,
            open_low_e(),
        );
        assert!(
            shapes.iter().all(|shape| shape.iter().all(|p| p.fret <= 3)),
            "shapes needing frets past 3 don't survive"
        );
    }

    #[test]
    fn pentatonic_degrees_drive_two_notes_per_string_shapes() {
        let shapes = positions(
            &instrument(),
            c_major(),
            ionian(),
            &scales::pentatonic_major(),
            2,
            open_low_e(),
        );
        assert_eq!(shapes.len(), 5, "five shapes, one per pentatonic degree");
        assert_eq!(
            as_pairs(&shapes[0]),
            vec![
                (0, 8),
                (0, 10),
                (1, 7),
                (1, 10),
                (2, 7),
                (2, 10),
                (3, 7),
                (3, 9),
                (4, 8),
                (4, 10),
                (5, 8),
                (5, 10),
            ],
            "C major pentatonic shape 1"
        );
    }

    #[test]
    fn arpeggio_degrees_drive_one_note_per_string_shapes() {
        let shapes = positions(
            &instrument(),
            c_major(),
            ionian(),
            &scales::arpeggio_major(),
            1,
            open_low_e(),
        );
        assert_eq!(shapes.len(), 3, "three shapes, one per arpeggio degree");
        assert_eq!(
            as_pairs(&shapes[0]),
            vec![(0, 8), (1, 7), (2, 5), (3, 5), (4, 5), (5, 3)],
            "C major arpeggio root shape"
        );
    }
}
