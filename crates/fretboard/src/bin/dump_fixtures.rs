use absolute::alteration::Alteration;
use absolute::letter::Letter;
use absolute::spelled_pitch::SpelledPitch;
use fretboard::chunk_solver;
use fretboard::fretboard as fb;
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::shape;
use fretboard::tuning;
use relative::chunk::{self, Chunk, Slot};
use relative::degree_reference::DegreeReference;
use relative::key::{Key, Quality};
use relative::scale_degree::ScaleDegree;
use relative::scales;

const ANCHOR: FretboardPosition = FretboardPosition {
    string_index: 0,
    fret: 8,
};

fn keys() -> Vec<(&'static str, Key)> {
    vec![
        (
            "C-major",
            Key::create(SpelledPitch::natural(Letter::C), Quality::Major),
        ),
        (
            "Eb-major",
            Key::create(
                SpelledPitch::create(Letter::E, Alteration::Flat),
                Quality::Major,
            ),
        ),
        (
            "F#-minor",
            Key::create(
                SpelledPitch::create(Letter::F, Alteration::Sharp),
                Quality::Minor,
            ),
        ),
        (
            "A-minor",
            Key::create(SpelledPitch::natural(Letter::A), Quality::Minor),
        ),
    ]
}

fn instruments() -> Vec<(&'static str, Instrument)> {
    vec![
        (
            "standard",
            Instrument::create_uniform(tuning::standard(), 22),
        ),
        ("drop-d", Instrument::create_uniform(tuning::drop_d(), 22)),
        (
            "bass-4",
            Instrument::create_uniform(tuning::bass_four(), 22),
        ),
        (
            "7-string",
            Instrument::create_uniform(tuning::standard_seven_string(), 22),
        ),
    ]
}

fn position_string(position: FretboardPosition) -> String {
    format!("{}.{}", position.string_index, position.fret)
}

fn shape_string(shape: &[FretboardPosition]) -> String {
    shape
        .iter()
        .map(|p| position_string(*p))
        .collect::<Vec<_>>()
        .join(" ")
}

fn dr_label(dr: DegreeReference) -> String {
    format!("{}/{}", dr.scale_degree.label(), dr.octave)
}

fn dr(degree: i32, octave: i32) -> DegreeReference {
    DegreeReference::natural(degree, octave)
}

fn altered(degree: i32, alteration: Alteration, octave: i32) -> DegreeReference {
    DegreeReference::create(ScaleDegree::create(degree, alteration), octave)
}

fn chunk_notes(chunk: &Chunk) -> Vec<DegreeReference> {
    chunk
        .iter()
        .filter_map(|slot| match slot {
            Slot::Rest => None,
            Slot::Note(dr) => Some(*dr),
        })
        .collect()
}

fn solver_total(shape: &[FretboardPosition]) -> i32 {
    let mut total = 0;
    let mut prev = ANCHOR;
    for position in shape {
        total += chunk_solver::distance(prev, *position);
        prev = *position;
    }
    total
}

fn print_canonical_shapes(shapes: Vec<Vec<FretboardPosition>>) {
    let mut canonical: Vec<(i32, String)> = shapes
        .iter()
        .map(|shape| (solver_total(shape), shape_string(shape)))
        .collect();
    canonical.sort();
    for (total, shape) in canonical {
        println!("  {total} {shape}");
    }
}

fn print_solver_section() {
    let base_chunk: Chunk = vec![
        Slot::Note(dr(1, 0)),
        Slot::Rest,
        Slot::Note(altered(3, Alteration::Flat, 0)),
        Slot::Note(dr(5, 0)),
    ];
    let progression_chunks: Vec<(String, Vec<DegreeReference>)> =
        chunk::apply_progression(&base_chunk, &[1, 4, 5, 6])
            .iter()
            .enumerate()
            .map(|(i, c)| (format!("prog-{}", i + 1), chunk_notes(c)))
            .collect();
    let mut note_sets: Vec<(String, Vec<DegreeReference>)> = vec![
        ("triad".to_string(), vec![dr(1, 0), dr(3, 0), dr(5, 0)]),
        ("leap".to_string(), vec![dr(1, 0), dr(1, 1), dr(5, 1)]),
        (
            "chromatic".to_string(),
            vec![
                altered(1, Alteration::Sharp, 0),
                altered(3, Alteration::Flat, 1),
                dr(5, 0),
            ],
        ),
    ];
    note_sets.extend(progression_chunks);
    for (key_name, key) in keys() {
        let mode = Key::mode_of_quality(key.quality);
        for (instrument_name, instrument) in instruments() {
            for (set_name, notes) in &note_sets {
                println!("solver {key_name} {instrument_name} {set_name} cap=4");
                print_canonical_shapes(chunk_solver::positions(
                    key,
                    mode,
                    &instrument,
                    ANCHOR,
                    4,
                    notes,
                ));
            }
        }
    }
}

fn print_shape_section() {
    let presets: Vec<(&str, Vec<ScaleDegree>, i32)> = vec![
        ("diatonic-3nps", ScaleDegree::diatonic(), 3),
        ("pentatonic-major-2nps", scales::pentatonic_major(), 2),
        ("pentatonic-minor-2nps", scales::pentatonic_minor(), 2),
        ("arpeggio-1nps", scales::arpeggio_major(), 1),
    ];
    for (key_name, key) in keys() {
        let mode = Key::mode_of_quality(key.quality);
        for (instrument_name, instrument) in instruments() {
            for (preset_name, degrees, notes_per_string) in &presets {
                println!(
                    "shapes {key_name} {instrument_name} {preset_name} nps={notes_per_string}"
                );
                for shape in
                    shape::positions(&instrument, key, mode, degrees, *notes_per_string, ANCHOR)
                {
                    println!("  {}", shape_string(&shape));
                }
            }
        }
    }
}

fn print_window_section() {
    for (key_name, key) in keys() {
        let mode = Key::mode_of_quality(key.quality);
        for (instrument_name, instrument) in instruments() {
            println!("window {key_name} {instrument_name} 0-12");
            for (position, degree) in fb::degrees_in_window(&instrument, key, mode, 0, 12) {
                println!("  {}:{}", position_string(position), dr_label(degree));
            }
        }
    }
}

fn print_tuning_section() {
    for (name, instrument) in instruments() {
        let open_notes: Vec<String> = (0..instrument.string_count())
            .map(|i| tuning::open_note(&instrument.tuning, i).to_string())
            .collect();
        println!("tuning {} {}", name, open_notes.join(" "));
    }
}

fn main() {
    print_tuning_section();
    print_solver_section();
    print_shape_section();
    print_window_section();
}
