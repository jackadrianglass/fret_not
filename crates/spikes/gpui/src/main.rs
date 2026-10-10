mod board;
mod model;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::AnimationExt;
use gpui::prelude::*;
use gpui::{
    App, Application, Bounds, ClickEvent, Context, IntoElement, Render, SharedString, Window,
    WindowBounds, WindowOptions, div, ease_in_out, px,
};

use board::{BoardGeom, BoardSnapshot, Visual, accent, paint_board};
use fretboard::fretboard::degree_at;
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::tuning;
use relative::chunk::{Chunk, Slot};
use relative::key::Quality;
use relative::scale_degree::ScaleDegree;

use model::LabelMode;

const SLOT_MS: f32 = 500.0;
const BLOOM_ANIMATION_MS: f32 = 700.0;
const HOVER_EASE_MS: f32 = 120.0;

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
    now: Instant,
    tonic_menu_open: bool,
    geom: Rc<RefCell<Option<BoardGeom>>>,
    bloom_delta: Rc<RefCell<f32>>,
    bloom_generation: usize,
}

impl FretNot {
    fn new(cx: &mut Context<Self>) -> FretNot {
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
            now: Instant::now(),
            tonic_menu_open: false,
            geom: Rc::new(RefCell::new(None)),
            bloom_delta: Rc::new(RefCell::new(1.0)),
            bloom_generation: 0,
        };
        state.resolve_fingerings();
        state.start_clock(cx);
        state
    }

    // 16ms tick: playhead, sweep, and hover easing are all driven from
    // `now`; bloom is driven by with_animation instead and needs no tick.
    fn start_clock(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let Ok(_) = this.update(cx, |state, cx| {
                    state.now = Instant::now();
                    if state.play_start.is_some() || state.sweep || state.hover.is_some() {
                        cx.notify();
                    }
                }) else {
                    break;
                };
            }
        })
        .detach();
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

    fn visuals(&self) -> (Vec<Visual>, Vec<(FretboardPosition, SharedString)>) {
        let (in_key, off_key) =
            model::board(&self.instrument, self.key(), self.mode(), self.label_mode());

        let hover_scale = match (self.hover, self.hover_since) {
            (Some(_), Some(since)) => {
                1.0 + 0.35
                    * ease_in_out(
                        (self.now.duration_since(since).as_secs_f32() * 1000.0 / HOVER_EASE_MS)
                            .min(1.0),
                    )
            }
            _ => 1.0,
        };
        let pulse = self.pulse();
        let pulse_position = self.pulse_position(pulse.map(|(slot, _)| slot));
        let pulse_factor = pulse.map(|(_, factor)| factor).unwrap_or(1.0);

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

        let mut visuals = Vec::with_capacity(in_key.len() + off_key.len());
        let mut labels = Vec::with_capacity(in_key.len());
        for dot in &in_key {
            visuals.push(Visual {
                position: dot.position,
                in_key: true,
                root_halo: dot.is_root,
                selected: Some(dot.degree.scale_degree) == self.selected,
                wave: if self.sweep {
                    0.4 + 0.6
                        * (0.5
                            + 0.5
                                * (std::f32::consts::TAU
                                    * (self.now.elapsed().as_secs_f32() / 1.2)
                                    - dot.position.fret as f32 * 0.35)
                                    .sin())
                } else {
                    1.0
                },
                scale: scale_of(dot.position),
            });
            labels.push((dot.position, SharedString::from(dot.label.clone())));
        }
        for position in &off_key {
            visuals.push(Visual {
                position: *position,
                in_key: false,
                root_halo: false,
                selected: false,
                wave: 1.0,
                scale: scale_of(*position),
            });
        }
        (visuals, labels)
    }

    fn tab_lines(&self) -> Vec<SharedString> {
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
                SharedString::from(line)
            })
            .collect()
    }

    fn select_at(&mut self, position: FretboardPosition) {
        self.selected = degree_at(&self.instrument, self.key(), self.mode(), position)
            .map(|dr| dr.scale_degree);
    }
}

fn dim(color: gpui::Hsla, alpha: f32) -> gpui::Hsla {
    gpui::hsla(color.h, color.s, color.l, alpha)
}

fn text_toggle(
    label: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(SharedString::from(label))
        .px_3()
        .py_1()
        .rounded_md()
        .text_size(px(13.))
        .when(active, |el| el.bg(accent()).text_color(gpui::black()))
        .when(!active, |el| {
            el.bg(gpui::hsla(0.0, 0.0, 0.0, 0.9))
                .text_color(gpui::white())
                .hover(|el| el.bg(dim(accent(), 0.7)))
        })
        .on_click(on_click)
        .child(SharedString::from(label))
}

impl Render for FretNot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (visuals, labels) = self.visuals();
        let tab_lines = self.tab_lines();
        let pulse = self.pulse();

        let snapshot = BoardSnapshot {
            visuals,
            string_count: self.instrument.string_count(),
            max_fret: self.instrument.max_fret(),
            geom: self.geom.clone(),
            bloom_delta: self.bloom_delta.clone(),
        };

        let mut board = div()
            .id("board")
            .relative()
            .flex_1()
            .min_h(px(200.))
            .overflow_hidden()
            .child(
                gpui::canvas(
                    move |bounds, _window, _cx| bounds,
                    move |bounds, _prepaint, window, cx| {
                        paint_board(&snapshot, bounds, window, cx);
                    },
                )
                .size_full(),
            );

        // Overlay children read the geometry cached by the previous frame's
        // paint — one frame of staleness, which only shows during resize.
        let cached_geom = *self.geom.borrow();
        if let Some(geom) = cached_geom {
            let hit_r = geom.dot_r * 2.0;
            for (i, (position, label)) in labels.iter().enumerate() {
                let position = *position;
                let label = label.clone();
                board = board.child(
                    div()
                        .id(("dot", i))
                        .absolute()
                        .left(px(geom.local_center_x(position.fret) - hit_r))
                        .top(px(geom.local_string_y(position.string_index) - hit_r))
                        .size(px(hit_r * 2.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(13.))
                        .text_color(gpui::white())
                        .hover(|el| el.cursor_pointer())
                        .on_hover(cx.listener(
                            move |state: &mut FretNot, hovered: &bool, _window, cx| {
                                if *hovered {
                                    state.hover = Some(position);
                                    state.hover_since = Some(state.now);
                                } else if state.hover == Some(position) {
                                    state.hover = None;
                                }
                                cx.notify();
                            },
                        ))
                        .on_click(
                            cx.listener(move |state: &mut FretNot, _event, _window, cx| {
                                state.select_at(position);
                                cx.notify();
                            }),
                        )
                        .child(label.clone()),
                );
            }
            for fret in 1..=geom.max_fret {
                board = board.child(
                    div()
                        .absolute()
                        .left(px(geom.local_center_x(fret) - 10.0))
                        .top(px(geom.local_string_y(geom.string_count - 1) + 4.0))
                        .w(px(20.))
                        .text_size(px(12.))
                        .text_color(gpui::hsla(0.0, 0.0, 0.55, 1.0))
                        .flex()
                        .justify_center()
                        .child(SharedString::from(fret.to_string())),
                );
            }
        }

        let tonic_name = model::tonic_list()[self.tonic_index].to_string();
        let controls =
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_size(px(16.))
                        .text_color(accent())
                        .child("fret_not — gpui spike"),
                )
                .child(
                    div()
                        .id("tonic")
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .text_size(px(13.))
                        .bg(gpui::hsla(0.0, 0.0, 0.0, 0.9))
                        .text_color(gpui::white())
                        .hover(|el| el.bg(dim(accent(), 0.7)))
                        .on_click(cx.listener(|state: &mut FretNot, _event, _window, cx| {
                            state.tonic_menu_open = !state.tonic_menu_open;
                            cx.notify();
                        }))
                        .child(format!("{} ▾", tonic_name)),
                )
                .when(self.tonic_menu_open, |el| {
                    el.child(
                        div()
                            .absolute()
                            .left(px(140.))
                            .top(px(56.))
                            .w(px(80.))
                            .rounded_md()
                            .bg(gpui::hsla(0.0, 0.0, 0.1, 1.0))
                            .flex()
                            .flex_col()
                            .p_1()
                            .gap_1()
                            .children(model::tonic_list().iter().enumerate().map(
                                |(index, tonic)| {
                                    let name = tonic.to_string();
                                    div()
                                        .id(("tonic-option", index))
                                        .px_2()
                                        .py_0p5()
                                        .rounded_sm()
                                        .text_size(px(13.))
                                        .text_color(gpui::white())
                                        .hover(|el| el.bg(dim(accent(), 0.5)))
                                        .on_click(cx.listener(
                                            move |state: &mut FretNot, _event, _window, cx| {
                                                if state.tonic_index != index {
                                                    state.tonic_index = index;
                                                    state.resolve_fingerings();
                                                    state.bloom_generation += 1;
                                                }
                                                state.tonic_menu_open = false;
                                                cx.notify();
                                            },
                                        ))
                                        .child(name)
                                },
                            )),
                    )
                })
                .child(text_toggle(
                    "Minor",
                    self.quality == Quality::Minor,
                    cx.listener(|state: &mut FretNot, _event, _window, cx| {
                        state.quality = if state.quality == Quality::Major {
                            Quality::Minor
                        } else {
                            Quality::Major
                        };
                        state.resolve_fingerings();
                        state.bloom_generation += 1;
                        cx.notify();
                    }),
                ))
                .child(text_toggle(
                    "note names",
                    self.notes_mode,
                    cx.listener(|state: &mut FretNot, _event, _window, cx| {
                        state.notes_mode = !state.notes_mode;
                        cx.notify();
                    }),
                ))
                .child(text_toggle(
                    "sweep",
                    self.sweep,
                    cx.listener(|state: &mut FretNot, _event, _window, cx| {
                        state.sweep = !state.sweep;
                        cx.notify();
                    }),
                ));

        let mut strip = div().flex().items_center().gap_1();
        for (i, slot) in self.chunk.iter().enumerate() {
            let (label, is_rest) = match slot {
                Slot::Note(dr) => (model::degree_label(*dr), false),
                Slot::Rest => (".".to_string(), true),
            };
            let is_current = pulse.map(|(slot, _)| slot) == Some(i);
            strip = strip.child(
                div()
                    .id(("slot", i))
                    .w(px(46.))
                    .py_1()
                    .flex()
                    .justify_center()
                    .rounded_md()
                    .text_size(px(13.))
                    .font(gpui::font("Menlo"))
                    .when(i == self.slot, |el| {
                        el.border_1()
                            .border_color(accent())
                            .bg(gpui::hsla(0.0, 0.0, 0.12, 1.0))
                    })
                    .when(is_current, |el| el.text_color(accent()))
                    .when(!is_current && is_rest, |el| {
                        el.text_color(gpui::hsla(0.0, 0.0, 0.5, 1.0))
                    })
                    .when(!is_current && !is_rest, |el| el.text_color(gpui::white()))
                    .on_click(
                        cx.listener(move |state: &mut FretNot, _event, _window, cx| {
                            if !state.chunk.is_empty() {
                                state.slot = i.min(state.chunk.len() - 1);
                                cx.notify();
                            }
                        }),
                    )
                    .child(label),
            );
        }
        strip = strip
            .child(text_toggle(
                "+",
                false,
                cx.listener(|state: &mut FretNot, _event, _window, cx| {
                    if state.chunk.len() < model::MAX_SLOTS {
                        state.chunk.push(Slot::Rest);
                        state.resolve_fingerings();
                        cx.notify();
                    }
                }),
            ))
            .child(text_toggle(
                "-",
                false,
                cx.listener(|state: &mut FretNot, _event, _window, cx| {
                    if !state.chunk.is_empty() {
                        let slot = state.slot;
                        state.chunk.remove(slot);
                        state.slot = slot.saturating_sub(1);
                        state.resolve_fingerings();
                        cx.notify();
                    }
                }),
            ));

        let solver = div()
            .flex()
            .items_center()
            .gap_2()
            .text_size(px(13.))
            .child(text_toggle(
                "prev",
                false,
                cx.listener(|state: &mut FretNot, _event, _window, cx| {
                    let len = state.solved.len();
                    if len > 0 {
                        state.fingering_index = (state.fingering_index + len - 1) % len;
                        cx.notify();
                    }
                }),
            ))
            .child(text_toggle(
                "next",
                false,
                cx.listener(|state: &mut FretNot, _event, _window, cx| {
                    let len = state.solved.len();
                    if len > 0 {
                        state.fingering_index = (state.fingering_index + 1) % len;
                        cx.notify();
                    }
                }),
            ))
            .child(
                div()
                    .text_color(gpui::hsla(0.0, 0.0, 0.8, 1.0))
                    .child(format!(
                        "fingering {}/{}",
                        self.fingering_index + 1,
                        self.solved.len()
                    )),
            )
            .child(text_toggle(
                "< chord",
                false,
                cx.listener(|state: &mut FretNot, _event, _window, cx| {
                    state.chord_index = (state.chord_index + model::PROGRESSION.len() - 1)
                        % model::PROGRESSION.len();
                    state.resolve_fingerings();
                    cx.notify();
                }),
            ))
            .child(
                div()
                    .text_color(gpui::hsla(0.0, 0.0, 0.8, 1.0))
                    .child(model::PROGRESSION_LABELS[self.chord_index]),
            )
            .child(text_toggle(
                "chord >",
                false,
                cx.listener(|state: &mut FretNot, _event, _window, cx| {
                    state.chord_index = (state.chord_index + 1) % model::PROGRESSION.len();
                    state.resolve_fingerings();
                    cx.notify();
                }),
            ))
            .child(text_toggle(
                if self.play_start.is_some() {
                    "stop"
                } else {
                    "play"
                },
                self.play_start.is_some(),
                cx.listener(|state: &mut FretNot, _event, _window, cx| {
                    state.play_start = match state.play_start {
                        Some(_) => None,
                        None => Some(state.now),
                    };
                    cx.notify();
                }),
            ));

        let tab = div()
            .flex()
            .flex_col()
            .gap_0p5()
            .children(tab_lines.into_iter().map(|line| {
                div()
                    .text_size(px(13.))
                    .font(gpui::font("Menlo"))
                    .text_color(gpui::hsla(0.0, 0.0, 0.8, 1.0))
                    .child(line)
            }));

        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .bg(gpui::hsla(0.0, 0.0, 0.09, 1.0))
            .child(controls)
            .child(strip)
            .child(solver)
            .child(tab)
            .child(board)
            .with_animation(
                SharedString::from(format!("bloom-{}", self.bloom_generation)),
                gpui::Animation::new(Duration::from_millis(BLOOM_ANIMATION_MS as u64)),
                {
                    let bloom_delta = self.bloom_delta.clone();
                    move |element, delta| {
                        *bloom_delta.borrow_mut() = delta;
                        element
                    }
                },
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, gpui::size(px(1400.), px(900.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| FretNot::new(cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
