mod model;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use slint::include_modules;
use slint::{ModelRc, SharedString, VecModel};

use fretboard::fretboard::degree_at;
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::tuning;
use relative::chunk::{Chunk, Slot};
use relative::key::Quality;
use relative::scale_degree::ScaleDegree;

use model::LabelMode;

include_modules!();

struct AppState {
    instrument: Instrument,
    tonic_index: i32,
    quality: Quality,
    notes_mode: bool,
    selected: Option<ScaleDegree>,
    chunk: Chunk,
    slot: usize,
    chord_index: usize,
    fingering_index: usize,
    solved: Vec<Vec<FretboardPosition>>,
    slot_notes: Vec<Option<usize>>,
}

impl AppState {
    fn new() -> AppState {
        let mut state = AppState {
            instrument: model::instrument(),
            tonic_index: 0,
            quality: Quality::Major,
            notes_mode: false,
            selected: None,
            chunk: model::initial_chunk(),
            slot: 0,
            chord_index: 0,
            fingering_index: 0,
            solved: Vec::new(),
            slot_notes: Vec::new(),
        };
        state.resolve_fingerings();
        state
    }

    fn key(&self) -> relative::key::Key {
        model::key_of(self.tonic_index.max(0) as usize, self.quality)
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
}

fn dot_infos(state: &AppState) -> Vec<DotInfo> {
    let key = state.key();
    let mode = state.mode();
    let (in_key, off_key) = model::board(&state.instrument, key, mode, state.label_mode());

    let mut dots = Vec::with_capacity(in_key.len() + off_key.len());
    for dot in &in_key {
        dots.push(DotInfo {
            string_index: dot.position.string_index as i32,
            fret: dot.position.fret,
            in_key: true,
            is_root: dot.is_root,
            is_selected: Some(dot.degree.scale_degree) == state.selected,
            label: dot.label.clone().into(),
        });
    }
    for position in &off_key {
        dots.push(DotInfo {
            string_index: position.string_index as i32,
            fret: position.fret,
            in_key: false,
            is_root: false,
            is_selected: false,
            label: SharedString::from(""),
        });
    }
    dots
}

fn slot_infos(state: &AppState) -> Vec<SlotInfo> {
    state
        .chunk
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            let (label, is_rest) = match slot {
                Slot::Note(dr) => (model::degree_label(*dr), false),
                Slot::Rest => (".".to_string(), true),
            };
            SlotInfo {
                index: index as i32,
                label: label.into(),
                is_rest,
                is_selected: index == state.slot,
            }
        })
        .collect()
}

fn slot_positions(state: &AppState) -> Vec<SlotPosition> {
    let fingering = state
        .solved
        .get(state.fingering_index)
        .cloned()
        .unwrap_or_default();
    state
        .chunk
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let position = state
                .slot_notes
                .get(index)
                .and_then(|note| *note)
                .and_then(|note_index| fingering.get(note_index));
            match position {
                Some(p) => SlotPosition {
                    string_index: p.string_index as i32,
                    fret: p.fret,
                },
                None => SlotPosition {
                    string_index: -1,
                    fret: -1,
                },
            }
        })
        .collect()
}

fn tab_lines(state: &AppState) -> Vec<SharedString> {
    let fingering = state
        .solved
        .get(state.fingering_index)
        .cloned()
        .unwrap_or_default();
    (0..state.instrument.string_count())
        .map(|string_index| {
            let mut line = format!(
                "{:>3}|",
                tuning::open_note(&state.instrument.tuning, string_index).to_string()
            );
            for (index, _slot) in state.chunk.iter().enumerate() {
                let played = state
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

fn refresh(ui: &MainWindow, state: &AppState) {
    ui.set_dots(ModelRc::from(Rc::new(VecModel::from(dot_infos(state)))));
    ui.set_slots(ModelRc::from(Rc::new(VecModel::from(slot_infos(state)))));
    ui.set_slot_positions(ModelRc::from(Rc::new(VecModel::from(slot_positions(
        state,
    )))));
    ui.set_slot_count(state.chunk.len() as i32);
    ui.set_tab_lines(ModelRc::from(Rc::new(VecModel::from(tab_lines(state)))));
    ui.set_fingering_count(state.solved.len() as i32);
    ui.set_fingering_index(state.fingering_index as i32);
    ui.set_fingering_label(
        format!(
            "fingering {}/{}",
            state.fingering_index + 1,
            state.solved.len()
        )
        .into(),
    );
    ui.set_chord_label(model::PROGRESSION_LABELS[state.chord_index].into());
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let state = Rc::new(RefCell::new(AppState::new()));

    let tonic_names: Vec<SharedString> = model::tonic_list()
        .iter()
        .map(|tonic| tonic.to_string().into())
        .collect();
    ui.set_tonic_names(ModelRc::from(Rc::new(VecModel::from(tonic_names))));
    ui.set_quality_names(ModelRc::from(Rc::new(VecModel::from(
        model::QUALITY_NAMES
            .iter()
            .map(|name| SharedString::from(*name))
            .collect::<Vec<_>>(),
    ))));
    ui.set_tonic_index(0);
    ui.set_quality_index(0);

    {
        let state = state.clone();
        let instrument = state.borrow().instrument.clone();
        ui.set_string_count(instrument.string_count() as i32);
        ui.set_max_fret(instrument.max_fret());
        ui.set_string_indices(ModelRc::from(Rc::new(VecModel::from(
            (0..instrument.string_count() as i32).collect::<Vec<_>>(),
        ))));
        ui.set_fret_numbers(ModelRc::from(Rc::new(VecModel::from(
            (1..=instrument.max_fret()).collect::<Vec<_>>(),
        ))));
        ui.set_fret_labels(ModelRc::from(Rc::new(VecModel::from(
            (1..=instrument.max_fret())
                .map(|fret| SharedString::from(fret.to_string()))
                .collect::<Vec<_>>(),
        ))));
        ui.set_inlay_frets(ModelRc::from(Rc::new(VecModel::from(
            [3, 5, 7, 9, 15, 17, 19, 21]
                .iter()
                .filter(|fret| **fret <= instrument.max_fret())
                .copied()
                .collect::<Vec<_>>(),
        ))));
        ui.set_inlay_double_frets(ModelRc::from(Rc::new(VecModel::from(
            [12].iter()
                .filter(|fret| **fret <= instrument.max_fret())
                .copied()
                .collect::<Vec<_>>(),
        ))));
    }

    ui.set_tick(0);
    ui.set_bloom_start(0);
    ui.set_play_start(-1);
    ui.set_playing(false);
    refresh(&ui, &state.borrow());

    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_tonic_selected(move |index| {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            if state.tonic_index == index {
                return;
            }
            state.tonic_index = index;
            state.resolve_fingerings();
            refresh(&ui, &state);
            ui.set_bloom_start(ui.get_tick());
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_quality_selected(move |index| {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            let quality = if index == 1 {
                Quality::Minor
            } else {
                Quality::Major
            };
            if state.quality == quality {
                return;
            }
            state.quality = quality;
            state.resolve_fingerings();
            refresh(&ui, &state);
            ui.set_bloom_start(ui.get_tick());
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_dot_clicked(move |string_index, fret| {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            let position = FretboardPosition {
                string_index: string_index.max(0) as usize,
                fret,
            };
            state.selected = degree_at(&state.instrument, state.key(), state.mode(), position)
                .map(|dr| dr.scale_degree);
            refresh(&ui, &state);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_slot_clicked(move |index| {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            if !state.chunk.is_empty() {
                state.slot = index.max(0) as usize;
                refresh(&ui, &state);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_add_slot(move || {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            if state.chunk.len() < model::MAX_SLOTS {
                state.chunk.push(Slot::Rest);
                state.resolve_fingerings();
                refresh(&ui, &state);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_remove_slot(move || {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            if !state.chunk.is_empty() {
                let slot = state.slot;
                state.chunk.remove(slot);
                state.slot = slot.saturating_sub(1);
                state.resolve_fingerings();
                refresh(&ui, &state);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_fingering_prev(move || {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            let len = state.solved.len();
            if len > 0 {
                state.fingering_index = (state.fingering_index + len - 1) % len;
                refresh(&ui, &state);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_fingering_next(move || {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            let len = state.solved.len();
            if len > 0 {
                state.fingering_index = (state.fingering_index + 1) % len;
                refresh(&ui, &state);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_chord_prev(move || {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            state.chord_index =
                (state.chord_index + model::PROGRESSION.len() - 1) % model::PROGRESSION.len();
            state.resolve_fingerings();
            refresh(&ui, &state);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_chord_next(move || {
            let ui = ui_weak.unwrap();
            let mut state = state.borrow_mut();
            state.chord_index = (state.chord_index + 1) % model::PROGRESSION.len();
            state.resolve_fingerings();
            refresh(&ui, &state);
        });
    }
    {
        let ui_weak = ui.as_weak();
        ui.on_play_toggled(move || {
            let ui = ui_weak.unwrap();
            let playing = !ui.get_playing();
            ui.set_playing(playing);
            if playing {
                ui.set_play_start(ui.get_tick());
            } else {
                ui.set_play_start(-1);
            }
        });
    }

    let boot = Instant::now();
    let timer = slint::Timer::default();
    {
        let ui_weak = ui.as_weak();
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(16),
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_tick(boot.elapsed().as_millis() as i32);
                }
            },
        );
    }

    ui.run()?;
    drop(timer);
    Ok(())
}
