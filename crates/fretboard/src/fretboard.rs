use absolute::note::Note;
use absolute::pitch_class::PitchClass;
use relative::degree_reference::DegreeReference;
use relative::key::Key;
use relative::mode::Mode;
use relative::scale_degree::ScaleDegree;

use crate::fretboard_position::FretboardPosition;
use crate::instrument::Instrument;
use crate::tuning;

// The tonic as a note in octave register 0; the octave anchor all degree
// resolution in this module is relative to.
pub fn root_note(key: Key) -> Note {
    Note::create(key.tonic, 0)
}

pub fn degree_note(key: Key, mode: Mode, dr: DegreeReference) -> Note {
    dr.note(root_note(key), mode)
}

// Every playable position sounding exactly that semitone, one per string at
// most, ascending by string.
pub fn positions_of_semitone(
    instrument: &Instrument,
    target_semitone: i32,
) -> Vec<FretboardPosition> {
    (0..instrument.string_count())
        .filter_map(|string_index| {
            let fret = target_semitone - tuning::open_semitone(&instrument.tuning, string_index);
            let position = FretboardPosition { string_index, fret };
            if instrument.playable(position) {
                Some(position)
            } else {
                None
            }
        })
        .collect()
}

// Every playable position sounding that exact degree and octave, ascending
// by string.
pub fn degree_positions(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    dr: DegreeReference,
) -> Vec<FretboardPosition> {
    positions_of_semitone(instrument, degree_note(key, mode, dr).semitone())
}

// None when the position sounds off-key.
pub fn degree_at(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    position: FretboardPosition,
) -> Option<DegreeReference> {
    let position_semitone = instrument.semitone_at(position);
    let position_pitch_class = PitchClass::of_int(position_semitone);
    let matching_degree = key
        .mode_pitch_classes(mode)
        .iter()
        .position(|pitch_class| *pitch_class == position_pitch_class)
        .map(|i| i as i32 + 1)?;
    let base = DegreeReference::natural(matching_degree, 0).interval(key.tonic_pitch_class(), mode);
    Some(DegreeReference {
        scale_degree: ScaleDegree::natural(matching_degree),
        octave: (position_semitone - root_note(key).semitone() - base) / 12,
    })
}

// Every playable in-key position with its degree, ascending by string then
// fret.
pub fn degrees_in_window(
    instrument: &Instrument,
    key: Key,
    mode: Mode,
    min_fret: i32,
    max_fret: i32,
) -> Vec<(FretboardPosition, DegreeReference)> {
    (0..instrument.string_count())
        .flat_map(|string_index| {
            (min_fret..=max_fret).filter_map(move |fret| {
                let position = FretboardPosition { string_index, fret };
                if instrument.playable(position) {
                    degree_at(instrument, key, mode, position).map(|degree| (position, degree))
                } else {
                    None
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{degree_at, degree_positions, degrees_in_window};
    use crate::fretboard_position::FretboardPosition;
    use crate::instrument::Instrument;
    use crate::tuning;
    use absolute::letter::Letter;
    use absolute::spelled_pitch::SpelledPitch;
    use relative::degree_reference::DegreeReference;
    use relative::key::{Key, Quality};
    use relative::mode::Mode;

    fn c_major() -> Key {
        Key::create(SpelledPitch::natural(Letter::C), Quality::Major)
    }

    fn ionian() -> Mode {
        Mode::Ionian
    }

    fn instrument() -> Instrument {
        Instrument::create_uniform(tuning::standard(), 22)
    }

    fn six_fret_instrument() -> Instrument {
        Instrument::create_uniform(tuning::standard(), 6)
    }

    fn as_pairs(positions: &[FretboardPosition]) -> Vec<(usize, i32)> {
        positions.iter().map(|p| (p.string_index, p.fret)).collect()
    }

    fn degree_positions_of(degree: i32, octave: i32) -> Vec<FretboardPosition> {
        degree_positions(
            &instrument(),
            c_major(),
            ionian(),
            DegreeReference::natural(degree, octave),
        )
    }

    #[test]
    fn degree_one_octave_3_is_playable_on_two_strings() {
        assert_eq!(
            as_pairs(&degree_positions_of(1, 3)),
            vec![(0, 8), (1, 3)],
            "C3 on the low E and A strings, ascending by string"
        );
    }

    #[test]
    fn degrees_below_the_instruments_range_have_no_positions() {
        assert_eq!(
            as_pairs(&degree_positions_of(1, 0)),
            Vec::<(usize, i32)>::new(),
            "C at the tonic's register 0 is below every open string"
        );
    }

    #[test]
    fn positions_of_a_degree_all_sound_the_same_semitone() {
        let semitones: Vec<i32> = degree_positions_of(3, 3)
            .iter()
            .map(|p| instrument().semitone_at(*p))
            .collect();
        assert_eq!(semitones, vec![52, 52, 52], "every E3 position sounds 52");
    }

    #[test]
    fn degree_at_round_trips_through_positions() {
        for (degree, octave) in [(1, 3), (3, 3), (5, 3), (7, 2), (4, 4)] {
            let dr = DegreeReference::natural(degree, octave);
            for position in degree_positions_of(degree, octave) {
                assert_eq!(
                    degree_at(&instrument(), c_major(), ionian(), position),
                    Some(dr),
                    "degree {} octave {} round trips",
                    degree,
                    octave
                );
            }
        }
    }

    #[test]
    fn degree_at_returns_none_off_key() {
        let chromatic_position = FretboardPosition {
            string_index: 0,
            fret: 6,
        };
        assert!(
            degree_at(&instrument(), c_major(), ionian(), chromatic_position).is_none(),
            "Bb on the low E string is off key in C major"
        );
    }

    #[test]
    fn open_low_e_is_degree_3_at_octave_2() {
        // Tonic-anchored octaves: the octave counts tonic registers, and E2
        // sits two registers above the register C is anchored at (register
        // 0 = C0).
        let open_low_e = FretboardPosition {
            string_index: 0,
            fret: 0,
        };
        let degree = degree_at(&instrument(), c_major(), ionian(), open_low_e).unwrap();
        assert_eq!(degree.scale_degree.degree, 3, "degree");
        assert_eq!(degree.octave, 2, "octave");
    }

    #[test]
    fn every_string_shows_all_seven_degrees_within_twelve_frets() {
        let window = degrees_in_window(&instrument(), c_major(), ionian(), 0, 12);
        let frets_on = |string_index: usize| {
            let mut frets: Vec<i32> = window
                .iter()
                .filter(|(p, _)| p.string_index == string_index)
                .map(|(p, _)| p.fret)
                .collect();
            frets.sort();
            frets.dedup();
            frets
        };
        assert_eq!(frets_on(0), vec![0, 1, 3, 5, 7, 8, 10, 12], "low E");
        assert_eq!(frets_on(1), vec![0, 2, 3, 5, 7, 8, 10, 12], "A");
        assert_eq!(frets_on(2), vec![0, 2, 3, 5, 7, 9, 10, 12], "D");
        assert_eq!(frets_on(3), vec![0, 2, 4, 5, 7, 9, 10, 12], "G");
        assert_eq!(frets_on(4), vec![0, 1, 3, 5, 6, 8, 10, 12], "B");
        assert_eq!(frets_on(5), vec![0, 1, 3, 5, 7, 8, 10, 12], "high E");
    }

    #[test]
    fn degrees_in_window_returns_the_degree_with_each_position() {
        let open_low_e = FretboardPosition {
            string_index: 0,
            fret: 0,
        };
        let (_, degree) = degrees_in_window(&instrument(), c_major(), ionian(), 0, 12)
            .into_iter()
            .find(|(position, _)| *position == open_low_e)
            .unwrap();
        assert_eq!(degree.scale_degree.degree, 3, "degree");
        assert_eq!(degree.octave, 2, "octave");
    }

    #[test]
    fn degrees_in_window_clamps_to_the_frets_each_string_actually_has() {
        let positions: Vec<FretboardPosition> =
            degrees_in_window(&six_fret_instrument(), c_major(), ionian(), 0, 12)
                .into_iter()
                .map(|(position, _)| position)
                .collect();
        assert!(
            positions.iter().all(|p| p.fret <= 6),
            "nothing beyond fret 6"
        );
        assert_eq!(
            as_pairs(
                &positions
                    .into_iter()
                    .filter(|p| p.string_index == 0)
                    .collect::<Vec<_>>()
            ),
            vec![(0, 0), (0, 1), (0, 3), (0, 5)],
            "low E keeps only its in-key frets up to 6"
        );
    }
}
