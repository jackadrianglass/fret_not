use std::time::{Duration, Instant};

mod fretboard_canvas;
mod model;

use iced::animation::{Animation, Easing};
use iced::widget::{button, canvas, column, container, pick_list, row, text, toggler};
use iced::{Border, Color, Element, Fill, Font};

use fretboard::fretboard::degree_at;
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::tuning;
use relative::chunk::{Chunk, Slot};
use relative::key::Quality;
use relative::scale_degree::ScaleDegree;

use crate::fretboard_canvas::{ACCENT, FretboardCanvas, Visual};
use crate::model::LabelMode;

const SLOT_MS: f32 = 500.0;
const BLOOM_MS: u64 = 250;
const BLOOM_FRET_STAGGER_MS: u64 = 18;

fn main() -> iced::Result {
    iced::application(FretNot::new, update, view)
        .title("fret_not — iced spike")
        .theme(theme)
        .window(iced::window::Settings {
            size: iced::Size::new(1400.0, 900.0),
            min_size: Some(iced::Size::new(1000.0, 620.0)),
            resizable: true,
            ..iced::window::Settings::default()
        })
        .subscription(subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    TonicPicked(String),
    QualityPicked(String),
    NotesToggled(bool),
    SweepToggled(bool),
    Tick(Instant),
    Hovered(Option<FretboardPosition>),
    Clicked(FretboardPosition),
    SlotSelected(usize),
    AddSlot,
    RemoveSlot,
    FingeringPrev,
    FingeringNext,
    ChordPrev,
    ChordNext,
    PlayToggled,
}

struct FretNot {
    instrument: Instrument,
    tonic_index: usize,
    quality: Quality,
    notes_mode: bool,
    sweep: bool,
    selected: Option<ScaleDegree>,
    hover: Option<FretboardPosition>,
    hover_anim: Animation<bool>,
    bloom: Vec<Animation<bool>>,
    chunk: Chunk,
    slot: usize,
    chord_index: usize,
    fingering_index: usize,
    solved: Vec<Vec<FretboardPosition>>,
    slot_notes: Vec<Option<usize>>,
    play_start: Option<Instant>,
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
            hover_anim: Animation::new(false).quick().easing(Easing::EaseOutQuad),
            bloom: Vec::new(),
            chunk: model::initial_chunk(),
            slot: 0,
            chord_index: 0,
            fingering_index: 0,
            solved: Vec::new(),
            slot_notes: Vec::new(),
            play_start: None,
            now: Instant::now(),
        };
        state.resolve_fingerings();
        state.rebuild_bloom();
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

    // One animation per visual, in the exact order the view zips against them:
    // in-key dots, then off-key positions.
    fn rebuild_bloom(&mut self) {
        let (dots, off_key) =
            model::board(&self.instrument, self.key(), self.mode(), self.label_mode());
        let now = self.now;
        self.bloom = dots
            .iter()
            .map(|d| d.position.fret)
            .chain(off_key.iter().map(|p| p.fret))
            .map(|fret| {
                Animation::new(false)
                    .duration(Duration::from_millis(BLOOM_MS))
                    .delay(Duration::from_millis(
                        fret.max(0) as u64 * BLOOM_FRET_STAGGER_MS,
                    ))
                    .easing(Easing::EaseOutCubic)
                    .go(true, now)
            })
            .collect();
    }

    fn pulse(&self) -> Option<(usize, f32)> {
        let start = self.play_start?;
        if self.chunk.is_empty() {
            return None;
        }
        let elapsed = self.now.duration_since(start).as_secs_f32() * 1000.0 / SLOT_MS;
        let slot = (elapsed.floor() as usize) % self.chunk.len();
        let frac = elapsed.fract();
        let factor = 1.0 + 0.3 * (1.0 - frac).powi(2);
        Some((slot, factor))
    }

    fn pulse_position(&self, pulse_slot: Option<usize>) -> Option<FretboardPosition> {
        let note_index = self.slot_notes.get(pulse_slot?).and_then(|note| *note)?;
        self.solved
            .get(self.fingering_index)?
            .get(note_index)
            .copied()
    }
}

fn theme(_state: &FretNot) -> iced::Theme {
    iced::Theme::Dracula
}

fn subscription(_state: &FretNot) -> iced::Subscription<Message> {
    iced::time::every(Duration::from_millis(16)).map(Message::Tick)
}

fn update(state: &mut FretNot, message: Message) {
    match message {
        Message::Tick(now) => state.now = now,
        Message::TonicPicked(name) => {
            if let Some(index) = model::tonic_list()
                .iter()
                .position(|tonic| tonic.to_string() == name)
            {
                if index != state.tonic_index {
                    state.tonic_index = index;
                    state.resolve_fingerings();
                    state.rebuild_bloom();
                }
            }
        }
        Message::QualityPicked(name) => {
            if let Some(quality) = model::quality_named(&name) {
                if quality != state.quality {
                    state.quality = quality;
                    state.resolve_fingerings();
                    state.rebuild_bloom();
                }
            }
        }
        Message::NotesToggled(enabled) => state.notes_mode = enabled,
        Message::SweepToggled(enabled) => state.sweep = enabled,
        Message::Hovered(hover) => {
            if state.hover != hover {
                state.hover = hover;
                state.hover_anim.go_mut(hover.is_some(), state.now);
            }
        }
        Message::Clicked(position) => {
            state.selected = degree_at(&state.instrument, state.key(), state.mode(), position)
                .map(|dr| dr.scale_degree);
        }
        Message::SlotSelected(index) => {
            if !state.chunk.is_empty() {
                state.slot = index.min(state.chunk.len() - 1);
            }
        }
        Message::AddSlot => {
            if state.chunk.len() < model::MAX_SLOTS {
                state.chunk.push(Slot::Rest);
                state.resolve_fingerings();
            }
        }
        Message::RemoveSlot => {
            if !state.chunk.is_empty() {
                state.chunk.remove(state.slot);
                state.slot = state.slot.saturating_sub(1);
                state.resolve_fingerings();
            }
        }
        Message::FingeringPrev => {
            let len = state.solved.len();
            if len > 0 {
                state.fingering_index = (state.fingering_index + len - 1) % len;
            }
        }
        Message::FingeringNext => {
            let len = state.solved.len();
            if len > 0 {
                state.fingering_index = (state.fingering_index + 1) % len;
            }
        }
        Message::ChordPrev => {
            state.chord_index =
                (state.chord_index + model::PROGRESSION.len() - 1) % model::PROGRESSION.len();
            state.resolve_fingerings();
        }
        Message::ChordNext => {
            state.chord_index = (state.chord_index + 1) % model::PROGRESSION.len();
            state.resolve_fingerings();
        }
        Message::PlayToggled => {
            state.play_start = match state.play_start {
                Some(_) => None,
                None => Some(state.now),
            };
        }
    }
}

fn bloom_factor(state: &FretNot, index: usize, now: Instant) -> f32 {
    state
        .bloom
        .get(index)
        .map(|animation| animation.interpolate(0.0, 1.0, now))
        .unwrap_or(1.0)
}

fn sweep_wave(now: Instant, fret: i32) -> f32 {
    let t = now.elapsed().as_secs_f32();
    let phase = std::f32::consts::TAU * (t / 1.2) - fret as f32 * 0.35;
    0.4 + 0.6 * (0.5 + 0.5 * phase.sin())
}

fn view(state: &FretNot) -> Element<'_, Message> {
    let key = state.key();
    let mode = state.mode();
    let now = state.now;
    let (dots, off_key) = model::board(&state.instrument, key, mode, state.label_mode());

    let hover_scale = state.hover_anim.interpolate(1.0, 1.35, now);
    let pulse = state.pulse();
    let pulse_position = state.pulse_position(pulse.map(|(slot, _)| slot));

    let scale_of = |position: FretboardPosition| {
        let mut scale = 1.0;
        if state.hover == Some(position) {
            scale *= hover_scale;
        }
        if pulse_position == Some(position) {
            scale *= pulse.map(|(_, factor)| factor).unwrap_or(1.0);
        }
        scale
    };

    let mut visuals = Vec::with_capacity(dots.len() + off_key.len());
    for (i, dot) in dots.iter().enumerate() {
        visuals.push(Visual {
            position: dot.position,
            label: Some(dot.label.clone()),
            in_key: true,
            root_halo: dot.is_root,
            selected: Some(dot.degree.scale_degree) == state.selected,
            bloom: bloom_factor(state, i, now),
            wave: if state.sweep {
                sweep_wave(now, dot.position.fret)
            } else {
                1.0
            },
            scale: scale_of(dot.position),
        });
    }
    for (j, position) in off_key.iter().enumerate() {
        visuals.push(Visual {
            position: *position,
            label: None,
            in_key: false,
            root_halo: false,
            selected: false,
            bloom: bloom_factor(state, dots.len() + j, now),
            wave: 1.0,
            scale: scale_of(*position),
        });
    }

    let controls = container(
        row![
            text("fret_not — iced spike").color(ACCENT),
            pick_list(
                model::tonic_list()
                    .iter()
                    .map(|tonic| tonic.to_string())
                    .collect::<Vec<_>>(),
                Some(model::tonic_list()[state.tonic_index].to_string()),
                Message::TonicPicked,
            ),
            pick_list(
                model::QUALITY_NAMES
                    .iter()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>(),
                Some(model::QUALITY_NAMES[state.quality as usize].to_string(),),
                Message::QualityPicked,
            ),
            toggler(state.notes_mode)
                .label("note names")
                .on_toggle(Message::NotesToggled),
            toggler(state.sweep)
                .label("sweep")
                .on_toggle(Message::SweepToggled),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    )
    .padding(8)
    .style(|_theme| container::Style {
        background: Some(Color::from_rgba(0.08, 0.09, 0.13, 1.0).into()),
        ..container::Style::default()
    });

    let mut strip = row![].spacing(4);
    for (i, slot) in state.chunk.iter().enumerate() {
        let (label, dim) = match slot {
            Slot::Note(dr) => (model::degree_label(*dr), false),
            Slot::Rest => (".".to_string(), true),
        };
        let is_current = pulse.map(|(pulse_slot, _)| pulse_slot) == Some(i);
        let is_selected = i == state.slot;
        let color = if is_current {
            ACCENT
        } else if dim {
            Color::from_rgba8(0x88, 0x88, 0x88, 1.0)
        } else {
            Color::WHITE
        };
        strip = strip.push(
            button(text(label).font(Font::MONOSPACE).color(color))
                .on_press(Message::SlotSelected(i))
                .style(move |_theme, _status| button::Style {
                    border: if is_selected {
                        Border {
                            radius: 4.0.into(),
                            width: 2.0,
                            color: ACCENT,
                        }
                    } else {
                        Border::default()
                    },
                    ..button::Style::default()
                }),
        );
    }
    let editor = row![
        strip,
        button("+").on_press(Message::AddSlot),
        button("-").on_press(Message::RemoveSlot),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let fingering = state
        .solved
        .get(state.fingering_index)
        .cloned()
        .unwrap_or_default();
    let lines: Vec<String> = (0..state.instrument.string_count())
        .map(|string_index| {
            let mut line = format!(
                "{:>3}|",
                tuning::open_note(&state.instrument.tuning, string_index).to_string()
            );
            for (i, _slot) in state.chunk.iter().enumerate() {
                let played = state
                    .slot_notes
                    .get(i)
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
        .collect();

    let solver = row![
        button("prev").on_press(Message::FingeringPrev),
        button("next").on_press(Message::FingeringNext),
        text(format!(
            "fingering {}/{}",
            state.fingering_index + 1,
            state.solved.len()
        )),
        button("< chord").on_press(Message::ChordPrev),
        text(model::PROGRESSION_LABELS[state.chord_index]),
        button("chord >").on_press(Message::ChordNext),
        button(if state.play_start.is_some() {
            "stop"
        } else {
            "play"
        })
        .on_press(Message::PlayToggled),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let tab = column(
        lines
            .into_iter()
            .map(|line| text(line).font(Font::MONOSPACE).size(13.0).into())
            .collect::<Vec<Element<Message>>>(),
    );

    let board = canvas(FretboardCanvas {
        visuals,
        string_count: state.instrument.string_count(),
        max_fret: state.instrument.max_fret(),
    })
    .width(Fill)
    .height(Fill);

    column![controls, editor, solver, tab, board]
        .spacing(10)
        .padding(12)
        .into()
}
