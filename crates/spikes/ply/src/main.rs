mod board;
mod model;

use ply_engine::prelude::*;

use board::{BoardGeom, Visual, draw_board};
use fretboard::fretboard::degree_at;
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::tuning;
use relative::chunk::{Chunk, Slot};
use relative::key::Quality;
use relative::scale_degree::ScaleDegree;

use model::LabelMode;

const FONT_PATH: &str = "/System/Library/Fonts/Supplemental/Arial.ttf";
const SLOT_MS: f64 = 500.0;
const BLOOM_MS: f64 = 250.0;
const BLOOM_FRET_STAGGER_MS: f64 = 18.0;
const HOVER_EASE_MS: f64 = 120.0;

struct FretNot {
    instrument: Instrument,
    tonic_index: usize,
    quality: Quality,
    notes_mode: bool,
    sweep: bool,
    selected: Option<ScaleDegree>,
    hover: Option<FretboardPosition>,
    hover_since: Option<f64>,
    chunk: Chunk,
    slot: usize,
    chord_index: usize,
    fingering_index: usize,
    solved: Vec<Vec<FretboardPosition>>,
    slot_notes: Vec<Option<usize>>,
    play_start: Option<f64>,
    bloom_start: f64,
    tonic_menu_open: bool,
    now: f64,
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
            bloom_start: 0.0,
            tonic_menu_open: false,
            now: 0.0,
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
        let elapsed = (self.now - start) * 1000.0;
        let slot = (elapsed / SLOT_MS) as usize % self.chunk.len();
        let frac = (elapsed % SLOT_MS) / SLOT_MS;
        Some((slot, 1.0 + 0.3 * (1.0 - frac as f32).powi(2)))
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
                let t = (((self.now - since) * 1000.0) / HOVER_EASE_MS) as f32;
                1.0 + 0.35 * ease_out_cubic(t.clamp(0.0, 1.0))
            }
            _ => 1.0,
        };
        let pulse = self.pulse();
        let pulse_position = self.pulse_position(pulse.map(|(slot, _)| slot));
        let pulse_factor = pulse.map(|(_, factor)| factor).unwrap_or(1.0);
        let bloom_elapsed = (self.now - self.bloom_start) * 1000.0;

        let bloom_of = |fret: i32| {
            ease_out_cubic(
                (((bloom_elapsed - fret as f64 * BLOOM_FRET_STAGGER_MS) / BLOOM_MS) as f32)
                    .clamp(0.0, 1.0),
            )
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
            0.5 + 0.5 * (std::f32::consts::TAU * (self.now as f32 / 1.2) - fret as f32 * 0.35).sin()
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

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "fret_not — ply spike".to_owned(),
            window_width: 1400,
            window_height: 900,
            high_dpi: true,
            sample_count: 4,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn flat_button(ui: &mut Ui, id: Id, label: &str) {
    ui.element()
        .id(id)
        .width(fit!())
        .height(fit!())
        .layout(|l| l.padding(8))
        .corner_radius(6.0)
        .background_color(0x1c1c26)
        .children(|ui| {
            ui.text(label, |t| t.font_size(14).color(0xFFFFFF));
        });
}

fn build_ui(ui: &mut Ui, state: &FretNot) {
    let pulse = state.pulse();
    let tonic_names: Vec<String> = model::tonic_list()
        .iter()
        .map(|tonic| tonic.to_string())
        .collect();

    ui.element()
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding(12).gap(10))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| {
                    l.direction(LeftToRight)
                        .padding(8)
                        .gap(10)
                        .align(Left, CenterY)
                })
                .background_color(0x1c1c26)
                .children(|ui| {
                    ui.text("fret_not — ply spike", |t| {
                        t.font_size(16).color(0xff8a3d)
                    });
                    flat_button(
                        ui,
                        Id::new("tonic-btn"),
                        &format!("{} ▾", tonic_names[state.tonic_index]),
                    );
                    flat_button(
                        ui,
                        Id::new("quality-btn"),
                        match state.quality {
                            Quality::Major => "Major",
                            Quality::Minor => "Minor",
                        },
                    );
                    flat_button(
                        ui,
                        Id::new("notes-btn"),
                        if state.notes_mode {
                            "note names ✓"
                        } else {
                            "note names"
                        },
                    );
                    flat_button(ui, Id::new("sweep-btn"), "sweep");

                    if state.tonic_menu_open {
                        ui.element()
                            .id(Id::new("tonic-menu"))
                            .floating(|f| {
                                f.anchor((Left, Top), (Left, Top))
                                    .offset((140.0, 56.0))
                                    .z_index(100)
                            })
                            .width(fixed!(120.0))
                            .height(fit!())
                            .layout(|l| l.direction(TopToBottom).padding(4).gap(2))
                            .background_color(0x1c1c26)
                            .corner_radius(6.0)
                            .children(|ui| {
                                for (i, name) in tonic_names.iter().enumerate() {
                                    ui.element()
                                        .id(Id::new_index("tonic-opt", i as u32))
                                        .width(grow!())
                                        .height(fit!())
                                        .layout(|l| l.padding(4))
                                        .children(|ui| {
                                            ui.text(name, |t| t.font_size(13).color(0xFFFFFF));
                                        });
                                }
                            });
                    }
                });

            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| l.direction(LeftToRight).gap(4).align(Left, CenterY))
                .children(|ui| {
                    for (i, slot) in state.chunk.iter().enumerate() {
                        let (label, is_rest) = match slot {
                            Slot::Note(dr) => (model::degree_label(*dr), false),
                            Slot::Rest => (".".to_string(), true),
                        };
                        let is_current = pulse.map(|(s, _)| s) == Some(i);
                        let is_selected = i == state.slot;
                        ui.element()
                            .id(Id::new_index("slot", i as u32))
                            .width(fixed!(44.0))
                            .height(fit!())
                            .layout(|l| l.padding(6).align(CenterX, CenterY))
                            .corner_radius(4.0)
                            .border(|b| {
                                if is_selected {
                                    b.color(0xff8a3d).all(2)
                                } else {
                                    b
                                }
                            })
                            .background_color(if is_selected { 0x22222c } else { 0x1c1c26 })
                            .children(|ui| {
                                ui.text(&label, |t| {
                                    t.font_size(13).color(if is_current {
                                        0xff8a3d
                                    } else if is_rest {
                                        0x888888
                                    } else {
                                        0xFFFFFF
                                    })
                                });
                            });
                    }
                    flat_button(ui, Id::new("add-btn"), "+");
                    flat_button(ui, Id::new("remove-btn"), "-");
                });

            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| l.direction(LeftToRight).gap(8).align(Left, CenterY))
                .children(|ui| {
                    flat_button(ui, Id::new("fingering-prev"), "prev");
                    flat_button(ui, Id::new("fingering-next"), "next");
                    let fingering_label = format!(
                        "fingering {}/{}",
                        state.fingering_index + 1,
                        state.solved.len()
                    );
                    ui.text(&fingering_label, |t| t.font_size(14).color(0xcccccc));
                    flat_button(ui, Id::new("chord-prev"), "< chord");
                    ui.text(model::PROGRESSION_LABELS[state.chord_index], |t| {
                        t.font_size(14).color(0xcccccc)
                    });
                    flat_button(ui, Id::new("chord-next"), "chord >");
                    flat_button(
                        ui,
                        Id::new("play-btn"),
                        if state.play_start.is_some() {
                            "stop"
                        } else {
                            "play"
                        },
                    );
                });

            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| l.direction(TopToBottom).gap(2))
                .children(|ui| {
                    for line in state.tab_lines() {
                        ui.text(&line, |t| t.font_size(13).color(0xcccccc));
                    }
                });

            ui.element()
                .id(Id::new("board"))
                .width(grow!())
                .height(grow!())
                .background_color(0x181820)
                .empty();
        });
}

#[macroquad::main(window_conf)]
async fn main() {
    static DEFAULT_FONT: FontAsset = FontAsset::Path(FONT_PATH);
    let mut ply = Ply::new(&DEFAULT_FONT).await;
    let font = load_ttf_font(FONT_PATH).await.unwrap();
    let mut state = FretNot::new();

    loop {
        clear_background(MacroquadColor::from_rgba(0x16, 0x16, 0x1c, 0xFF));
        state.now = get_time();

        let mut ui = ply.begin();
        build_ui(&mut ui, &state);
        ui.show(|_| {}).await;

        if ui.is_just_released(Id::new("tonic-btn")) {
            state.tonic_menu_open = !state.tonic_menu_open;
        }
        if ui.is_just_released(Id::new("quality-btn")) {
            state.quality = if state.quality == Quality::Major {
                Quality::Minor
            } else {
                Quality::Major
            };
            state.rekey();
        }
        if ui.is_just_released(Id::new("notes-btn")) {
            state.notes_mode = !state.notes_mode;
        }
        if ui.is_just_released(Id::new("sweep-btn")) {
            state.sweep = !state.sweep;
        }
        if ui.is_just_released(Id::new("add-btn")) && state.chunk.len() < model::MAX_SLOTS {
            state.chunk.push(Slot::Rest);
            state.resolve_fingerings();
        }
        if ui.is_just_released(Id::new("remove-btn")) && !state.chunk.is_empty() {
            let slot = state.slot;
            state.chunk.remove(slot);
            state.slot = slot.saturating_sub(1);
            state.resolve_fingerings();
        }
        if ui.is_just_released(Id::new("fingering-prev")) {
            let len = state.solved.len();
            if len > 0 {
                state.fingering_index = (state.fingering_index + len - 1) % len;
            }
        }
        if ui.is_just_released(Id::new("fingering-next")) {
            let len = state.solved.len();
            if len > 0 {
                state.fingering_index = (state.fingering_index + 1) % len;
            }
        }
        if ui.is_just_released(Id::new("chord-prev")) {
            state.chord_index =
                (state.chord_index + model::PROGRESSION.len() - 1) % model::PROGRESSION.len();
            state.resolve_fingerings();
        }
        if ui.is_just_released(Id::new("chord-next")) {
            state.chord_index = (state.chord_index + 1) % model::PROGRESSION.len();
            state.resolve_fingerings();
        }
        if ui.is_just_released(Id::new("play-btn")) {
            state.play_start = match state.play_start {
                Some(_) => None,
                None => Some(state.now),
            };
        }
        for i in 0..model::tonic_list().len() {
            if ui.is_just_released(Id::new_index("tonic-opt", i as u32)) {
                if state.tonic_index != i {
                    state.tonic_index = i;
                    state.rekey();
                }
                state.tonic_menu_open = false;
            }
        }
        if state.tonic_menu_open
            && is_mouse_button_pressed(MouseButton::Left)
            && !ui.pointer_over(Id::new("tonic-menu"))
            && !ui.pointer_over(Id::new("tonic-btn"))
        {
            state.tonic_menu_open = false;
        }
        for i in 0..state.chunk.len() {
            if ui.is_just_released(Id::new_index("slot", i as u32)) {
                state.slot = i;
            }
        }

        if let Some(bbox) = ui.bounding_box(Id::new("board")) {
            let geom = BoardGeom::new(
                bbox,
                state.instrument.string_count(),
                state.instrument.max_fret(),
            );

            let hovered = geom.hit_test(mouse_position());
            if hovered != state.hover {
                state.hover = hovered;
                state.hover_since = if hovered.is_some() {
                    Some(state.now)
                } else {
                    None
                };
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(position) = geom.hit_test(mouse_position()) {
                    state.select_at(position);
                }
            }

            draw_board(&geom, &state.visuals(), &font);
        }

        next_frame().await;
    }
}
