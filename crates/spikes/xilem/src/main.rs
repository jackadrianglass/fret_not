mod board;
mod model;

use std::time::{Duration, Instant};

use xilem::core::fork;
use xilem::view::{flex_col, flex_row, label, task, text_button};
use xilem::{EventLoop, WidgetView, WindowOptions, Xilem};

use board::fretboard;
use fretboard::fretboard::degree_at;
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::tuning;
use relative::chunk::{Chunk, Slot};
use relative::key::Quality;
use relative::scale_degree::ScaleDegree;

use model::LabelMode;

const SLOT_MS: f32 = 500.0;
const BLOOM_MS: f32 = 250.0;
const BLOOM_FRET_STAGGER_MS: f32 = 18.0;
const HOVER_EASE_MS: f32 = 120.0;

// The visual snapshot the board widget paints; every factor is
// pre-interpolated in the app state, the widget only draws.
pub struct Visual {
    pub position: FretboardPosition,
    pub in_key: bool,
    pub root_halo: bool,
    pub selected: bool,
    pub label: Option<String>,
    pub bloom: f32,
    pub wave: f32,
    pub scale: f32,
}

struct FretNot {
    instrument: Instrument,
    tonic_index: usize,
    quality: Quality,
    notes_mode: bool,
    sweep: bool,
    selected: Option<ScaleDegree>,
    hover: Option<FretboardPosition>,
    hover_since: Option<Instant>,
    chunk: Chunk,
    slot: usize,
    chord_index: usize,
    fingering_index: usize,
    solved: Vec<Vec<FretboardPosition>>,
    slot_notes: Vec<Option<usize>>,
    play_start: Option<Instant>,
    bloom_start: Instant,
    now: Instant,
}

impl FretNot {
    fn new() -> FretNot {
        let mut state = FretNot {
            instrument: model::instrument(),
            tonic_index: 0,
            quality: Quality::Major,
            notes_mode: false,
            sweep: false,
            selected: None,
            hover: None,
            hover_since: None,
            chunk: model::initial_chunk(),
            slot: 0,
            chord_index: 0,
            fingering_index: 0,
            solved: Vec::new(),
            slot_notes: Vec::new(),
            play_start: None,
            bloom_start: Instant::now(),
            now: Instant::now(),
        };
        state.resolve_fingerings();
        state
    }

    fn key(&self) -> relative::key::Key {
        model::key_of(self.tonic_index, self.quality)
    }

    fn mode(&self) -> relative::mode::Mode {
        model::mode_of(self.quality)
    }

    fn label_mode(&self) -> LabelMode {
        if self.notes_mode {
            LabelMode::Notes
        } else {
            LabelMode::Degrees
        }
    }

    fn resolve_fingerings(&mut self) {
        let (solved, slot_notes) = model::solve(
            &self.instrument,
            self.key(),
            self.mode(),
            &self.chunk,
            model::PROGRESSION[self.chord_index],
        );
        self.solved = solved;
        self.slot_notes = slot_notes;
        self.fingering_index = 0;
        if self.slot >= self.chunk.len() {
            self.slot = self.chunk.len().saturating_sub(1);
        }
    }

    fn rekey(&mut self) {
        self.resolve_fingerings();
        self.bloom_start = self.now;
    }

    fn pulse(&self) -> Option<(usize, f32)> {
        let start = self.play_start?;
        if self.chunk.is_empty() {
            return None;
        }
        let elapsed = self.now.duration_since(start).as_secs_f32() * 1000.0;
        let slot = (elapsed / SLOT_MS) as usize % self.chunk.len();
        let frac = (elapsed % SLOT_MS) / SLOT_MS;
        Some((slot, 1.0 + 0.3 * (1.0 - frac).powi(2)))
    }

    fn pulse_position(&self, pulse_slot: Option<usize>) -> Option<FretboardPosition> {
        let note = *self.slot_notes.get(pulse_slot?)?;
        self.solved.get(self.fingering_index)?.get(note?).copied()
    }

    fn visuals(&self) -> Vec<Visual> {
        let (in_key, off_key) =
            model::board(&self.instrument, self.key(), self.mode(), self.label_mode());

        let hover_scale = match (self.hover, self.hover_since) {
            (Some(_), Some(since)) => {
                let t = (self.now.duration_since(since).as_secs_f32() * 1000.0 / HOVER_EASE_MS)
                    .min(1.0);
                1.0 + 0.35 * t * t
            }
            _ => 1.0,
        };
        let pulse = self.pulse();
        let pulse_position = self.pulse_position(pulse.map(|(slot, _)| slot));
        let pulse_factor = pulse.map(|(_, factor)| factor).unwrap_or(1.0);
        let bloom_elapsed = self.now.duration_since(self.bloom_start).as_secs_f32() * 1000.0;

        let bloom_of = |fret: i32| {
            ((bloom_elapsed - fret as f32 * BLOOM_FRET_STAGGER_MS) / BLOOM_MS).clamp(0.0, 1.0)
        };
        let scale_of = |position: FretboardPosition| {
            let mut scale = if Some(position) == self.hover {
                hover_scale
            } else {
                1.0
            };
            if Some(position) == pulse_position {
                scale *= pulse_factor;
            }
            scale
        };
        let wave_of = |fret: i32| -> f32 {
            if !self.sweep {
                return 1.0;
            }
            0.5 + 0.5
                * (std::f32::consts::TAU * (self.now.elapsed().as_secs_f32() / 1.2)
                    - fret as f32 * 0.35)
                    .sin()
        };

        let mut visuals = Vec::with_capacity(in_key.len() + off_key.len());
        for dot in &in_key {
            visuals.push(Visual {
                position: dot.position,
                in_key: true,
                root_halo: dot.is_root,
                selected: Some(dot.degree.scale_degree) == self.selected,
                label: Some(dot.label.clone()),
                bloom: bloom_of(dot.position.fret),
                wave: wave_of(dot.position.fret),
                scale: scale_of(dot.position),
            });
        }
        for position in &off_key {
            visuals.push(Visual {
                position: *position,
                in_key: false,
                root_halo: false,
                selected: false,
                label: None,
                bloom: bloom_of(position.fret),
                wave: 1.0,
                scale: scale_of(*position),
            });
        }
        visuals
    }

    fn tab_lines(&self) -> Vec<String> {
        let fingering = self
            .solved
            .get(self.fingering_index)
            .cloned()
            .unwrap_or_default();
        (0..self.instrument.string_count())
            .map(|string_index| {
                let mut line = format!(
                    "{:>3}|",
                    tuning::open_note(&self.instrument.tuning, string_index).to_string()
                );
                for (index, _slot) in self.chunk.iter().enumerate() {
                    let played = self
                        .slot_notes
                        .get(index)
                        .and_then(|note| *note)
                        .and_then(|note_index| fingering.get(note_index))
                        .filter(|p| p.string_index == string_index);
                    line += &match played {
                        Some(position) => format!("{:>4}", position.fret),
                        None => "   -".to_string(),
                    };
                }
                line
            })
            .collect()
    }

    fn select_at(&mut self, position: FretboardPosition) {
        self.selected = degree_at(&self.instrument, self.key(), self.mode(), position)
            .map(|dr| dr.scale_degree);
    }
}

fn tick(state: &mut FretNot) {
    state.now = Instant::now();
}

fn clock(
    proxy: xilem::core::MessageProxy<()>,
    _state: &mut FretNot,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
    Box::pin(async move {
        loop {
            xilem::tokio::time::sleep(Duration::from_millis(16)).await;
            if proxy.message(()).is_err() {
                break;
            }
        }
    })
}

fn app_logic(state: &mut FretNot) -> impl WidgetView<FretNot> + use<> {
    let pulse = state.pulse();

    let mut tonics = Vec::new();
    for (index, tonic) in model::tonic_list().iter().enumerate() {
        let name = tonic.to_string();
        let text = if index == state.tonic_index {
            format!("[{}]", name)
        } else {
            name
        };
        tonics.push(text_button(text, move |state: &mut FretNot| {
            if state.tonic_index != index {
                state.tonic_index = index;
                state.rekey();
            }
        }));
    }

    let quality_label = match state.quality {
        Quality::Major => "quality: Major",
        Quality::Minor => "quality: Minor",
    };

    let mut slots = Vec::new();
    for (i, slot) in state.chunk.iter().enumerate() {
        let (text, is_current) = match slot {
            Slot::Note(dr) => (model::degree_label(*dr), pulse.map(|(s, _)| s) == Some(i)),
            Slot::Rest => (".".to_string(), pulse.map(|(s, _)| s) == Some(i)),
        };
        let text = if i == state.slot {
            format!("({})", text)
        } else if is_current {
            format!(">{}<", text)
        } else {
            text
        };
        slots.push(text_button(text, move |state: &mut FretNot| {
            if !state.chunk.is_empty() {
                state.slot = i.min(state.chunk.len() - 1);
            }
        }));
    }

    let solver_row = flex_row((
        text_button("prev", |state: &mut FretNot| {
            let len = state.solved.len();
            if len > 0 {
                state.fingering_index = (state.fingering_index + len - 1) % len;
            }
        }),
        text_button("next", |state: &mut FretNot| {
            let len = state.solved.len();
            if len > 0 {
                state.fingering_index = (state.fingering_index + 1) % len;
            }
        }),
        label(format!(
            "fingering {}/{}",
            state.fingering_index + 1,
            state.solved.len()
        )),
        text_button("< chord", |state: &mut FretNot| {
            state.chord_index =
                (state.chord_index + model::PROGRESSION.len() - 1) % model::PROGRESSION.len();
            state.resolve_fingerings();
        }),
        label(model::PROGRESSION_LABELS[state.chord_index]),
        text_button("chord >", |state: &mut FretNot| {
            state.chord_index = (state.chord_index + 1) % model::PROGRESSION.len();
            state.resolve_fingerings();
        }),
        text_button(
            if state.play_start.is_some() {
                "stop"
            } else {
                "play"
            },
            |state: &mut FretNot| {
                state.play_start = match state.play_start {
                    Some(_) => None,
                    None => Some(state.now),
                };
            },
        ),
    ));

    let tab = flex_col(
        state
            .tab_lines()
            .into_iter()
            .map(|line| label(line))
            .collect::<Vec<_>>(),
    );

    fork(
        flex_col((
            flex_row((
                label("fret_not — xilem spike"),
                flex_row(tonics),
                text_button(quality_label, |state: &mut FretNot| {
                    state.quality = if state.quality == Quality::Major {
                        Quality::Minor
                    } else {
                        Quality::Major
                    };
                    state.rekey();
                }),
                text_button(
                    if state.notes_mode {
                        "note names ✓"
                    } else {
                        "note names"
                    },
                    |state: &mut FretNot| state.notes_mode = !state.notes_mode,
                ),
                text_button("sweep", |state: &mut FretNot| state.sweep = !state.sweep),
            )),
            flex_row((
                slots,
                text_button("+", |state: &mut FretNot| {
                    if state.chunk.len() < model::MAX_SLOTS {
                        state.chunk.push(Slot::Rest);
                        state.resolve_fingerings();
                    }
                }),
                text_button("-", |state: &mut FretNot| {
                    if !state.chunk.is_empty() {
                        let slot = state.slot;
                        state.chunk.remove(slot);
                        state.slot = slot.saturating_sub(1);
                        state.resolve_fingerings();
                    }
                }),
            )),
            solver_row,
            tab,
            fretboard(),
        )),
        task(clock, |state: &mut FretNot, (): ()| tick(state)),
    )
}

fn main() -> Result<(), xilem::winit::error::EventLoopError> {
    let app = Xilem::new_simple(
        FretNot::new(),
        app_logic,
        WindowOptions::new("fret_not — xilem spike"),
    );
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
