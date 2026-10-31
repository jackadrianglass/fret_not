use std::collections::HashMap;

use absolute::spelled_pitch::SpelledPitch;
use iced::widget::{Stack, button, column, container, mouse_area, row, scrollable, text};
use iced::{Alignment, Element, Fill};

use fretboard::chunk_solver;
use fretboard::fingering::{self, Fingering};
use fretboard::fretboard::{degree_at, spelled_degree_at};
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use relative::chunk::{self, Slot, Word};
use relative::degree_reference::DegreeReference;
use relative::key::Quality;

use crate::board::{self, BoardState, Highlight};
use crate::colors;
use crate::model;
use crate::{ChunkEntry, Document, Session, Settings};

const BOARD_HEIGHT: f32 = 260.0;

// The per-chunk editing context, keyed by chunk index: the register the
// chunk's octaves count from and the fingering being displayed — one
// position per slot, set by the user's clicks and filled from the solve
// once, sticky afterwards.
#[derive(Clone)]
struct ChunkEdit {
    // The key-anchored register of the chunk's anchor note — its first
    // note. Established by the first note written into the chunk (or by
    // where the solve voices it, until then) and moved when the anchor note
    // itself is rewritten.
    base: Option<i32>,
    // The solve's fret cap: grows only when the user places a wider move
    // than it admits. Swaps bring in fingerings the cap already
    // enumerated, so it never moves during rotation and the catalog stays
    // fixed.
    cap: i32,
    // Slot-aligned display positions; None for rests and slots not yet
    // placed. A position only changes when the user touches the slot.
    positions: Vec<Option<FretboardPosition>>,
}

pub struct EditScreen {
    slot: usize,
    edits: HashMap<usize, ChunkEdit>,
    solved: Vec<Fingering>,
    slot_notes: Vec<Option<usize>>,
    // The displayed fingering: the edited chunk's positions, slot-aligned
    // in [edits] and projected here for the view.
    positions: Fingering,
    // Which member of [solved] is currently displayed — the stepper's
    // index, re-derived in refresh from [positions].
    fingering_index: Option<usize>,
    // The insert point under the cursor, if any — a band's hover state,
    // tracked per point (chunk, insert index) so the hovered compartment
    // can expand.
    hovered_insert: Option<(usize, usize)>,
    // The insert point whose expansion the hover timer will fire. The
    // compartment expands only after the cursor dwells on it, so passing
    // over a point on the way to a cell never re-flows the band.
    hover_pending: Option<(usize, usize)>,
    // The chunk + session the caches were derived against; a change resets
    // the cursor and, on a key or instrument change, the click history —
    // stored positions are instrument-relative, so a new instrument
    // invalidates them exactly like a new key.
    anchored: Option<(usize, Instrument, SpelledPitch, Quality)>,
    // Set when the cursor was explicitly placed on a freshly edited chunk
    // (a band's cell click); refresh must not reset it to the first slot.
    slot_pinned: bool,
}

// The index of the chunk's first note — the anchor note its octaves count
// from and whose position centers the solve window.
fn anchor_index(chunk: &Word) -> Option<usize> {
    chunk.iter().position(|slot| matches!(slot, Slot::Note(_)))
}

// The chunk's first note is the octave anchor; after a mutation that can
// change which note is first, restore the invariant — the anchor sits at
// octave 0 — without moving any sounding pitch.
fn reanchor(chunk: &mut Word, edit: &mut ChunkEdit) {
    if let Some(Slot::Note(dr)) = anchor_index(chunk).map(|i| chunk[i])
        && dr.octave != 0
    {
        chunk::shift_octaves(chunk, -dr.octave);
        edit.base = edit.base.map(|base| base + dr.octave);
    }
}

// The widest consecutive fret move across the chunk's displayed notes —
// the reach the solver must allow for the displayed fingering to be
// enumerable among its results.
fn displayed_fret_span(edit: &ChunkEdit, chunk: &Word) -> i32 {
    let frets: Vec<i32> = chunk
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| match slot {
            Slot::Note(_) => edit.positions.get(i).copied().flatten().map(|p| p.fret),
            Slot::Rest => None,
        })
        .collect();
    frets
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .max()
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub enum Message {
    Board(board::Event),
    SlotSelected { chunk: usize, slot: usize },
    RemoveSlot { chunk: usize, slot: usize },
    InsertSlot { chunk: usize, slot: usize },
    InsertHovered { chunk: usize, slot: usize },
    InsertUnhovered { chunk: usize, slot: usize },
    ToggleRest,
    VariantCreated,
    FingeringPrev,
    FingeringNext,
    HoverTick,
}

impl EditScreen {
    pub fn new() -> EditScreen {
        EditScreen {
            slot: 0,
            edits: HashMap::new(),
            solved: Vec::new(),
            slot_notes: Vec::new(),
            positions: Fingering::default(),
            fingering_index: None,
            hovered_insert: None,
            hover_pending: None,
            anchored: None,
            slot_pinned: false,
        }
    }

    // Whether an insert point's expansion is waiting on the hover timer —
    // the shell runs the tick subscription only then.
    pub fn hover_pending(&self) -> bool {
        self.hover_pending.is_some()
    }

    // The chunk's octave reference register — the anchor note's.
    fn base_of(&self, chunk: usize) -> Option<i32> {
        self.edits.get(&chunk).and_then(|edit| edit.base)
    }

    pub fn refresh(&mut self, session: &Session, document: &Document, settings: &Settings) {
        let anchored = (
            document.selected_chunk,
            session.instrument.clone(),
            session.tonic,
            session.quality,
        );
        if self.anchored != Some(anchored.clone()) {
            // A key or instrument change invalidates every chunk's click
            // history; a chunk switch keeps the per-chunk edits.
            let stale = match &self.anchored {
                Some((_, instrument, tonic, quality)) => {
                    *instrument != session.instrument
                        || *tonic != session.tonic
                        || *quality != session.quality
                }
                None => false,
            };
            if stale {
                self.edits.clear();
            }
            self.anchored = Some(anchored);
            if !self.slot_pinned {
                self.slot = 0;
            }
            self.slot_pinned = false;
        }
        let chunk = &document.chunks[document.selected_chunk];
        // A hover left over from before a removal can point past its
        // chunk's end.
        let stale_hover = |point: Option<(usize, usize)>| {
            point.is_some_and(|(chunk, slot)| {
                document
                    .chunks
                    .get(chunk)
                    .map(|entry| entry.len())
                    .unwrap_or(0)
                    < slot
            })
        };
        if stale_hover(self.hovered_insert) {
            self.hovered_insert = None;
        }
        if stale_hover(self.hover_pending) {
            self.hover_pending = None;
        }
        let edit = self
            .edits
            .entry(document.selected_chunk)
            .or_insert_with(|| ChunkEdit {
                base: None,
                cap: settings.solver_fret_cap,
                positions: vec![None; chunk.len()],
            });
        // The cap grows only when the user places a wider move than it
        // admits — swapped-in fingerings are always within it, so the
        // rotation catalog stays fixed. The setting is the base: a raised
        // cap lifts every chunk, a lowered one only reaches chunks that
        // have not grown past it.
        edit.cap = edit
            .cap
            .max(settings.solver_fret_cap)
            .max(displayed_fret_span(edit, chunk));
        let mut solved = Vec::new();
        let mut slot_notes = vec![None; chunk.len()];
        if let Some(i) = anchor_index(chunk)
            && let Slot::Note(dr) = chunk[i]
        {
            // The root note's pin: where the user placed it, or — never
            // placed yet — its default starting position, the nearest
            // playable position sounding it. The catalog pins to either
            // from the first refresh, so the stepper count never collapses
            // after a rotation.
            let first = edit.positions.get(i).copied().flatten().or_else(|| {
                chunk_solver::starting_position(
                    session.key(),
                    session.mode(),
                    &session.instrument,
                    dr,
                    model::SOLVER_ANCHOR,
                )
            });
            if let Some(first) = first {
                let (fingerings, notes) = model::solve_pinned(
                    &session.instrument,
                    session.key(),
                    session.mode(),
                    chunk,
                    1,
                    first,
                    edit.cap,
                );
                solved = fingerings;
                slot_notes = notes;
            }
        }
        self.solved = solved;
        self.slot_notes = slot_notes;
        // A note slot without a position takes one from the best current
        // fingering — once. Afterwards it only moves when the user touches
        // the slot.
        for (i, slot) in chunk.iter().enumerate() {
            if !matches!(slot, Slot::Note(_)) || edit.positions.get(i).copied().flatten().is_some()
            {
                continue;
            }
            let filled = self.solved.first().and_then(|fingering| {
                self.slot_notes
                    .get(i)
                    .and_then(|note| *note)
                    .and_then(|note_index| fingering.get(note_index).copied())
            });
            if let Some(position) = filled {
                edit.positions[i] = Some(position);
            }
        }
        // The register octaves count from: the anchor note's displayed
        // register minus its stored octave. Established once per chunk+key
        // so clicks always convert against the same reference.
        if edit.base.is_none()
            && let Some(i) = anchor_index(chunk)
            && let (Slot::Note(dr), Some(position)) = (chunk[i], edit.positions[i])
        {
            edit.base = degree_at(&session.instrument, session.key(), session.mode(), position)
                .map(|clicked| clicked.octave - dr.octave);
        }
        let positions: Vec<FretboardPosition> = chunk
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| match slot {
                Slot::Note(_) => edit.positions.get(i).copied().flatten(),
                Slot::Rest => None,
            })
            .collect();
        self.positions = Fingering::new(positions);
        self.fingering_index = self
            .solved
            .iter()
            .position(|fingering| *fingering == self.positions);
    }

    // True when the document changed and the other screens' caches are stale.
    pub fn update(
        &mut self,
        message: Message,
        session: &Session,
        document: &mut Document,
        settings: &Settings,
    ) -> bool {
        match message {
            Message::Board(board::Event::Clicked(position)) => {
                write_position(self, session, document, settings, position)
            }
            Message::SlotSelected { chunk, slot } => {
                let Some(len) = document.chunks.get(chunk).map(|entry| entry.len()) else {
                    return false;
                };
                // Clicking another band's cell edits that chunk and moves
                // the cursor onto the clicked slot.
                let switched = chunk != document.selected_chunk;
                if switched {
                    document.selected_chunk = chunk;
                    self.slot_pinned = true;
                }
                self.slot = if len > 0 { slot.min(len - 1) } else { 0 };
                switched
            }
            Message::InsertHovered { chunk, slot } => {
                self.hover_pending = Some((chunk, slot));
                false
            }
            Message::InsertUnhovered { chunk, slot } => {
                if self.hover_pending == Some((chunk, slot)) {
                    self.hover_pending = None;
                }
                if self.hovered_insert == Some((chunk, slot)) {
                    self.hovered_insert = None;
                }
                false
            }
            Message::HoverTick => {
                if let Some(point) = self.hover_pending.take() {
                    self.hovered_insert = Some(point);
                }
                false
            }
            Message::InsertSlot { chunk, slot } => {
                let Some(entry) = document.chunks.get_mut(chunk) else {
                    return false;
                };
                if entry.len() >= model::MAX_SLOTS {
                    return false;
                }
                let slot = slot.min(entry.len());
                chunk::insert(entry, slot, Slot::Rest);
                if let Some(edit) = self.edits.get_mut(&chunk) {
                    edit.positions.insert(slot, None);
                }
                if chunk == document.selected_chunk {
                    // The cursor lands on the fresh slot: the next board
                    // click writes into it.
                    self.slot = slot;
                }
                true
            }
            Message::RemoveSlot { chunk, slot } => {
                let Some(entry) = document.chunks.get_mut(chunk) else {
                    return false;
                };
                if entry.is_empty() {
                    return false;
                }
                let slot = slot.min(entry.len() - 1);
                chunk::remove(entry, slot);
                if chunk == document.selected_chunk {
                    // The cursor tracks its slot: a deletion before it shifts
                    // it left; deleting past the end clamps it onto the new
                    // end.
                    if slot < self.slot {
                        self.slot -= 1;
                    }
                    self.slot = self.slot.min(entry.len().saturating_sub(1));
                }
                if let Some(edit) = self.edits.get_mut(&chunk) {
                    edit.positions.remove(slot);
                    reanchor(entry, edit);
                }
                true
            }
            Message::ToggleRest => {
                let chunk = &mut document.chunks[document.selected_chunk];
                if chunk.is_empty() {
                    return false;
                }
                let index = self.slot.min(chunk.len() - 1);
                let slot = match chunk[index] {
                    Slot::Rest => Slot::Note(DegreeReference::natural(1, 0)),
                    Slot::Note(_) => Slot::Rest,
                };
                chunk::set(chunk, index, slot);
                if let Some(edit) = self.edits.get_mut(&document.selected_chunk) {
                    edit.positions[index] = None;
                    reanchor(chunk, edit);
                }
                true
            }
            Message::VariantCreated => {
                let selected = document.selected_chunk;
                let root = document.root_of(selected);
                let source = document.chunks[selected].clone();
                let edit = self.edits.get(&selected).cloned();
                let index = document.chunks.len();
                document.chunks.push(ChunkEntry {
                    chunk: source.chunk,
                    variant_of: Some(root),
                });
                // The variant opens exactly where its source was left: the
                // copied edit state carries its authored positions and
                // octave base.
                if let Some(edit) = edit {
                    self.edits.insert(index, edit);
                }
                document.selected_chunk = index;
                self.slot = 0;
                true
            }
            Message::FingeringPrev => step_fingering(self, document, -1),
            Message::FingeringNext => step_fingering(self, document, 1),
        }
    }
}

// A click writes the selected slot and steps the cursor. Every position
// resolves through spelled_degree_at — in-key or accidental — and
// stores relative to the anchor note: the chunk's first note, the register
// the octaves count from. Writing the anchor note moves the reference (the
// other notes shift to keep their sounding pitches); any other note stores
// its register distance from it. The clicked position becomes the slot's
// display position — it sticks until the user touches the slot again.
// Rests are authored by the rest toggle, not by clicks.
fn write_position(
    screen: &mut EditScreen,
    session: &Session,
    document: &mut Document,
    settings: &Settings,
    position: FretboardPosition,
) -> bool {
    let chunk = &mut document.chunks[document.selected_chunk];
    if chunk.is_empty() {
        return false;
    }
    let index = screen.slot.min(chunk.len() - 1);
    let clicked = spelled_degree_at(&session.instrument, session.key(), session.mode(), position);
    screen
        .edits
        .entry(document.selected_chunk)
        .or_insert_with(|| ChunkEdit {
            base: None,
            cap: settings.solver_fret_cap,
            positions: vec![None; chunk.len()],
        });
    let edit = screen.edits.get_mut(&document.selected_chunk).unwrap();
    let slot = {
        // The write lands on the anchor when it writes the chunk's first
        // note — at, before, or into a note-less chunk.
        let writes_anchor = match anchor_index(chunk) {
            Some(anchor) => index <= anchor,
            None => true,
        };
        let slot = match edit.base {
            Some(base) if !writes_anchor => Slot::Note(DegreeReference {
                octave: clicked.octave - base,
                ..clicked
            }),
            Some(base) => {
                chunk::shift_octaves(chunk, base - clicked.octave);
                edit.base = Some(clicked.octave);
                Slot::Note(DegreeReference {
                    octave: 0,
                    ..clicked
                })
            }
            None => {
                edit.base = Some(clicked.octave);
                Slot::Note(DegreeReference {
                    octave: 0,
                    ..clicked
                })
            }
        };
        edit.positions[index] = Some(position);
        slot
    };
    chunk::set(chunk, index, slot);
    // The cursor steps right but never past the band's end — growing the
    // chunk is the insert points' and the trailing "+"'s job.
    if index + 1 < chunk.len() {
        screen.slot = index + 1;
    }
    true
}

// Steps the displayed fingering to the catalog's neighbor (wrapping) and
// writes it back as the chunk's sticky positions. The root note stays
// pinned — every member voices it at the same position, so the octave
// reference never moves.
fn step_fingering(screen: &mut EditScreen, document: &mut Document, delta: i32) -> bool {
    let Some(index) = screen.fingering_index else {
        return false;
    };
    let len = screen.solved.len();
    let Some(swapped) = screen
        .solved
        .get((index as i32 + delta).rem_euclid(len as i32) as usize)
    else {
        return false;
    };
    let Some(edit) = screen.edits.get_mut(&document.selected_chunk) else {
        return false;
    };
    edit.positions = fingering::slot_positions(swapped, &screen.slot_notes);
    true
}

pub fn view<'a>(
    screen: &'a EditScreen,
    session: &'a Session,
    document: &'a Document,
    board: &'a BoardState,
    settings: &'a Settings,
) -> Element<'a, Message> {
    let family = document.family(document.selected_chunk);
    let names = model::chunk_names(&document.lineage());

    let stepper = row![
        button("prev").on_press(Message::FingeringPrev),
        text(format!(
            "{}/{}",
            screen.fingering_index.map(|i| i + 1).unwrap_or(0),
            screen.solved.len()
        ))
        .size(settings.scale(16.0)),
        button("next").on_press(Message::FingeringNext),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // The rest toggle and the stepper serve the edited chunk, so they sit
    // below the stack rather than beside one band.
    let controls = row![button("rest").on_press(Message::ToggleRest), stepper]
        .spacing(16)
        .align_y(Alignment::Center);

    // The edited chunk's family stacked as slot bands: the root first,
    // then each variant underneath, all visible at once.
    let mut stack = column![].spacing(8);
    for &index in &family {
        stack = stack.push(band_row(
            screen,
            session,
            document,
            index,
            names[index].clone(),
            settings,
        ));
    }
    stack = stack.push(
        button(text("+ add variant").size(settings.scale(14.0)))
            .on_press(Message::VariantCreated)
            .style(trailing_style),
    );

    let board_area = container(
        board::view(
            board,
            Highlight {
                selected: selected_slot_position(screen, document),
                positions: screen.positions.clone(),
            },
            settings,
        )
        .map(Message::Board),
    )
    .height(BOARD_HEIGHT);

    column![
        scrollable(stack).width(Fill).height(Fill),
        controls,
        board_area,
    ]
    .spacing(12)
    .height(Fill)
    .width(Fill)
    .into()
}

// One chunk's band in the stack: its label, then its slot band. The
// edited chunk's band carries the accent border and label; the board,
// the stepper and the rest toggle all follow it.
fn band_row<'a>(
    screen: &'a EditScreen,
    session: &'a Session,
    document: &'a Document,
    index: usize,
    name: String,
    settings: &'a Settings,
) -> Element<'a, Message> {
    let chunk = &document.chunks[index];
    let edited = index == document.selected_chunk;
    let base = screen.base_of(index);
    let label = button(text(name).size(settings.scale(16.0)))
        .on_press(Message::SlotSelected {
            chunk: index,
            slot: 0,
        })
        .style(move |theme, _status| label_style(theme, edited));
    row![
        container(label).width(settings.scale(28.0)),
        slot_band(screen, session, base, chunk, index, edited, settings),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn label_style(theme: &iced::Theme, edited: bool) -> button::Style {
    button::Style {
        text_color: if edited {
            colors::accent(theme)
        } else {
            colors::secondary_text(theme)
        },
        border: if edited {
            iced::Border {
                radius: 4.0.into(),
                width: 2.0,
                color: colors::accent(theme),
            }
        } else {
            iced::Border::default()
        },
        ..button::Style::default()
    }
}

// The selected slot's position on the board — its sticky display position.
fn selected_slot_position(screen: &EditScreen, document: &Document) -> Option<FretboardPosition> {
    let chunk = &document.chunks[document.selected_chunk];
    let index = screen.slot.min(chunk.len().saturating_sub(1));
    if !matches!(chunk.get(index), Some(Slot::Note(_))) {
        return None;
    }
    screen
        .edits
        .get(&document.selected_chunk)
        .and_then(|edit| edit.positions.get(index).copied().flatten())
}

fn secondary(theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(colors::secondary_text(theme)),
    }
}

fn accent(theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(colors::accent(theme)),
    }
}

fn band_outline(theme: &iced::Theme, edited: bool) -> container::Style {
    container::Style {
        border: iced::Border {
            radius: 6.0.into(),
            width: 1.0,
            color: if edited {
                colors::accent(theme)
            } else {
                colors::divider_color(theme)
            },
        },
        ..container::Style::default()
    }
}

fn cell_style(theme: &iced::Theme, status: button::Status, selected: bool) -> button::Style {
    button::Style {
        background: match status {
            button::Status::Hovered => Some(colors::group_background(theme).into()),
            _ => None,
        },
        text_color: colors::text(theme),
        border: iced::Border {
            width: 1.0,
            radius: 4.0.into(),
            color: if selected {
                colors::accent(theme)
            } else {
                colors::cell_border(theme)
            },
        },
        ..button::Style::default()
    }
}

fn ghost(theme: &iced::Theme, status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: match status {
            button::Status::Hovered => colors::accent(theme),
            _ => colors::secondary_text(theme),
        },
        ..button::Style::default()
    }
}

// An insert point's hover state: resting, hovering (the cursor is over it
// and the expansion timer is running), or expanded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertHover {
    Rest,
    Hovering,
    Expanded,
}

fn insert_style(theme: &iced::Theme, hover: InsertHover) -> button::Style {
    button::Style {
        // Hovering tints the bar to mark it interactive; the expansion
        // itself still waits for the dwell.
        background: match hover {
            InsertHover::Rest => Some(colors::divider_color(theme).into()),
            InsertHover::Hovering => Some(colors::cell_border(theme).into()),
            InsertHover::Expanded => None,
        },
        text_color: if hover == InsertHover::Expanded {
            colors::accent(theme)
        } else {
            iced::Color::TRANSPARENT
        },
        ..button::Style::default()
    }
}

fn trailing_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    button::Style {
        background: match status {
            button::Status::Hovered => Some(colors::group_background(theme).into()),
            _ => None,
        },
        text_color: colors::accent(theme),
        ..button::Style::default()
    }
}

// The selected cell's underline: accent when selected, invisible otherwise
// so every cell keeps the same height.
fn underline(selected: bool) -> impl Fn(&iced::Theme) -> iced::widget::rule::Style {
    move |theme| iced::widget::rule::Style {
        color: if selected {
            colors::accent(theme)
        } else {
            iced::Color::TRANSPARENT
        },
        radius: Default::default(),
        fill_mode: iced::widget::rule::FillMode::Full,
        snap: true,
    }
}

// An insert point between cells. The mouse_area tracks hover so the
// compartment can expand under the cursor; the wrapped button keeps the
// click.
fn insert_point<'a>(
    chunk: usize,
    slot: usize,
    hover: InsertHover,
    settings: &'a Settings,
) -> Element<'a, Message> {
    let width = if hover == InsertHover::Expanded {
        settings.scale(22.0)
    } else {
        settings.scale(6.0)
    };
    mouse_area(
        button(container(text("+").size(settings.scale(14.0))).center(Fill))
            .width(width)
            .height(Fill)
            .padding(0)
            .on_press(Message::InsertSlot { chunk, slot })
            .style(move |theme, _status| insert_style(theme, hover)),
    )
    .on_enter(Message::InsertHovered { chunk, slot })
    .on_exit(Message::InsertUnhovered { chunk, slot })
    .into()
}

fn slot_band<'a>(
    screen: &'a EditScreen,
    session: &'a Session,
    base: Option<i32>,
    chunk: &'a Word,
    index: usize,
    edited: bool,
    settings: &'a Settings,
) -> Element<'a, Message> {
    let mut band = row![].align_y(Alignment::Center);

    for (i, slot) in chunk.iter().enumerate() {
        // The cursor underline marks the edited chunk's selected slot only.
        let is_selected = edited && i == screen.slot;
        let cell = match slot {
            Slot::Note(dr) => {
                let name = match base {
                    Some(base) => {
                        model::degree_note_in_register(session.key(), session.mode(), base, *dr)
                            .to_string()
                    }
                    None => String::new(),
                };
                // The underline rule is Fill-wide; pinning the column keeps
                // the cell content-sized instead of leaking Fill up the band.
                column![
                    text(model::slot_label(*dr))
                        .size(settings.scale(16.0))
                        .style(accent),
                    text(name).size(settings.scale(11.0)).style(secondary),
                    iced::widget::rule::horizontal(2.0).style(underline(is_selected)),
                ]
                .spacing(2)
                .width(iced::Length::Shrink)
            }
            Slot::Rest => column![
                text(".").size(settings.scale(16.0)).style(secondary),
                text("rest").size(settings.scale(11.0)).style(secondary),
                iced::widget::rule::horizontal(2.0).style(underline(is_selected)),
            ]
            .spacing(2)
            .width(iced::Length::Shrink),
        };
        // The delete button rides the cell's top-right corner as an overlay
        // layer — the container passes clicks through to the cell below.
        let delete = button(text("×").size(settings.scale(10.0)))
            .padding(2)
            .on_press(Message::RemoveSlot {
                chunk: index,
                slot: i,
            })
            .style(ghost);
        band = band.push(Stack::with_children(vec![
            button(cell.padding(iced::Padding {
                top: 4.0,
                right: 16.0,
                bottom: 2.0,
                left: 8.0,
            }))
            .on_press(Message::SlotSelected {
                chunk: index,
                slot: i,
            })
            .style(move |theme, status| cell_style(theme, status, is_selected))
            .into(),
            container(delete)
                .width(Fill)
                .height(Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Top)
                .into(),
        ]));
        if i + 1 < chunk.len() {
            let hover = if screen.hovered_insert == Some((index, i + 1)) {
                InsertHover::Expanded
            } else if screen.hover_pending == Some((index, i + 1)) {
                InsertHover::Hovering
            } else {
                InsertHover::Rest
            };
            band = band.push(insert_point(index, i + 1, hover, settings));
        }
    }

    band = band.push(
        button(container(text("+").size(settings.scale(14.0))).center(Fill))
            .width(settings.scale(26.0))
            .height(Fill)
            .padding(0)
            .on_press(Message::InsertSlot {
                chunk: index,
                slot: chunk.len(),
            })
            .style(trailing_style),
    );

    // The insert points fill the row's height; pinning the row to Shrink
    // keeps the band content-sized — Row::push would otherwise enclose the
    // row's height with their Fill.
    container(band.height(iced::Length::Shrink))
        .padding(4)
        .style(move |theme| band_outline(theme, edited))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;
    use absolute::alteration::Alteration;
    use relative::scale_degree::ScaleDegree;

    fn session() -> Session {
        Session {
            instrument: model::instrument(),
            tonic: model::TONICS[0],
            quality: Quality::Major,
        }
    }

    fn settings() -> Settings {
        Settings::default()
    }

    fn rests_document(slots: usize) -> Document {
        Document {
            chunks: vec![Word::new(vec![Slot::Rest; slots]).into()],
            selected_chunk: 0,
            steps: Vec::new(),
        }
    }

    fn pos(string_index: usize, fret: i32) -> FretboardPosition {
        FretboardPosition { string_index, fret }
    }

    fn octave_of(slot: Slot) -> Option<i32> {
        match slot {
            Slot::Note(dr) => Some(dr.octave),
            Slot::Rest => None,
        }
    }

    // The shell's loop: update writes, refresh re-derives the caches.
    fn click(
        screen: &mut EditScreen,
        session: &Session,
        document: &mut Document,
        settings: &Settings,
        position: FretboardPosition,
    ) {
        screen.update(
            Message::Board(board::Event::Clicked(position)),
            session,
            document,
            settings,
        );
        screen.refresh(session, document, settings);
    }

    #[test]
    fn the_same_degree_an_octave_apart_stores_distinct_octaves() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1)],
            "open low E then its octave: degree 3 at chunk octaves 0 and 1, no rest appended behind the cursor"
        );
        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(0, 0), pos(0, 12)]),
            "both clicks display where they were clicked"
        );
    }

    #[test]
    fn a_click_below_the_anchor_stores_a_negative_octave() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), Some(-1)],
            "the anchor note stays at octave 0, the lower note counts below it"
        );
        assert_eq!(
            screen.edits[&0].base,
            Some(3),
            "the anchor keeps its register"
        );
        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(0, 12), pos(0, 0)]),
            "both clicks display where they were clicked"
        );
    }

    #[test]
    fn an_off_key_click_writes_the_accidental() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 6));

        assert_eq!(screen.slot, 1, "the cursor steps right");
        assert_eq!(
            document.chunks[0][0],
            Slot::Note(DegreeReference::new(
                ScaleDegree::new(7, Alteration::Flat),
                0
            )),
            "Bb2 on the low E string writes the flat 7 at the anchor register"
        );
        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(0, 6)]),
            "the accidental displays where it was clicked"
        );

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));

        assert_eq!(
            octave_of(document.chunks[0][1]),
            Some(0),
            "an in-key note after the accidental still counts from the same base"
        );
        assert_eq!(
            screen.edits[&0].base,
            Some(2),
            "the accidental's register anchored the chunk"
        );
    }

    #[test]
    fn rests_stay_authored_by_the_toggle() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        assert!(matches!(document.chunks[0][0], Slot::Note(_)));

        screen.update(
            Message::SlotSelected { chunk: 0, slot: 0 },
            &session,
            &mut document,
            &settings,
        );
        screen.update(Message::ToggleRest, &session, &mut document, &settings);
        screen.refresh(&session, &document, &settings);
        assert!(
            matches!(document.chunks[0][0], Slot::Rest),
            "the toggle still converts a note into a rest"
        );
        assert_eq!(
            screen.edits[&0].positions[0], None,
            "the rest clears the slot's display position"
        );
    }

    #[test]
    fn writing_at_the_end_stops_instead_of_appending() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        assert_eq!(screen.slot, 1, "the cursor steps right within the band");
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        assert_eq!(
            document.chunks[0].len(),
            2,
            "writing the last slot grows nothing"
        );
        assert_eq!(screen.slot, 1, "the cursor pins at the band's end");

        click(&mut screen, &session, &mut document, &settings, pos(1, 0));
        assert_eq!(
            document.chunks[0].len(),
            2,
            "rewriting the last slot grows nothing either"
        );
        assert_eq!(
            screen.edits[&0].positions[1],
            Some(pos(1, 0)),
            "the rewrite still lands on the last slot"
        );
    }

    #[test]
    fn switching_chunks_resets_the_cursor_and_rederives_the_base() {
        let session = session();
        let settings = settings();
        let mut document = Document {
            chunks: vec![
                Word::new(vec![Slot::Rest, Slot::Rest]).into(),
                model::second_chunk().into(),
            ],
            selected_chunk: 0,
            steps: Vec::new(),
        };
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        assert_eq!(screen.slot, 1);

        document.selected_chunk = 1;
        screen.refresh(&session, &document, &settings);

        assert_eq!(screen.slot, 0, "the cursor resets on a chunk switch");
        assert!(
            screen.base_of(document.selected_chunk).is_some(),
            "the new chunk's anchor note establishes the base"
        );
    }

    #[test]
    fn authored_clicks_beat_the_solve_where_they_disagree() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(4, 6));

        assert_eq!(
            screen.solved.first(),
            Some(&Fingering::new(vec![pos(0, 0), pos(5, 1)])),
            "the solve re-voices slot 1's F4 to the high E string"
        );
        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(0, 0), pos(4, 6)]),
            "the authored click is displayed where it was clicked"
        );
        assert_eq!(
            screen.edits[&0].base,
            Some(2),
            "the anchor note's click fixes the octave reference"
        );
    }

    #[test]
    fn rewriting_the_anchor_note_moves_the_octave_reference() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        screen.update(
            Message::SlotSelected { chunk: 0, slot: 0 },
            &session,
            &mut document,
            &settings,
        );
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), Some(0)],
            "the rewritten anchor keeps octave 0, the other note shifts to keep its register"
        );
        assert_eq!(screen.edits[&0].base, Some(2));
    }

    #[test]
    fn removing_the_anchor_rebases_to_the_new_first_note() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(3);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        screen.update(
            Message::SlotSelected { chunk: 0, slot: 0 },
            &session,
            &mut document,
            &settings,
        );
        screen.update(
            Message::RemoveSlot { chunk: 0, slot: 0 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), None],
            "the note below the removed anchor becomes the new octave reference"
        );
        assert_eq!(screen.edits[&0].base, Some(2));
    }

    #[test]
    fn unplaced_slots_fill_from_the_solve_once_and_stick() {
        let session = session();
        let settings = settings();
        let mut document = Document {
            chunks: vec![model::second_chunk().into()],
            selected_chunk: 0,
            steps: Vec::new(),
        };
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        let filled = screen.positions[1];
        click(&mut screen, &session, &mut document, &settings, pos(4, 0));

        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(4, 0), filled]),
            "the filled position sticks across edits; only the touched slot moves"
        );
        assert_ne!(
            screen.solved.first(),
            Some(&screen.positions),
            "the solve would have re-voiced slot 1 — the fill must not follow it"
        );
    }

    #[test]
    fn a_never_placed_chunk_pins_to_its_starting_position() {
        let session = session();
        let settings = settings();
        let document = Document {
            chunks: vec![model::second_chunk().into()],
            selected_chunk: 0,
            steps: Vec::new(),
        };
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        let first = screen
            .positions
            .first()
            .copied()
            .expect("the chunk fills on the first refresh");
        assert!(
            screen
                .solved
                .iter()
                .all(|fingering| fingering.first() == Some(&first)),
            "the catalog pins to the starting position from the first refresh"
        );
        assert_eq!(
            screen.fingering_index,
            Some(0),
            "the fill is the catalog's best fingering"
        );

        let catalog = screen.solved.clone();
        screen.refresh(&session, &document, &settings);
        assert_eq!(
            screen.solved, catalog,
            "no pivot: a redundant refresh keeps the catalog"
        );
    }

    #[test]
    fn chunk_switches_keep_each_chunks_click_history() {
        let session = session();
        let settings = settings();
        let mut document = Document {
            chunks: vec![
                Word::new(vec![Slot::Rest]).into(),
                model::second_chunk().into(),
            ],
            selected_chunk: 0,
            steps: Vec::new(),
        };
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));

        document.selected_chunk = 1;
        screen.refresh(&session, &document, &settings);
        document.selected_chunk = 0;
        screen.refresh(&session, &document, &settings);

        assert_eq!(
            screen.positions.first(),
            Some(&pos(0, 0)),
            "the click history survives the round trip"
        );
    }

    #[test]
    fn a_key_change_clears_the_click_history() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));

        let changed = Session {
            instrument: model::instrument(),
            tonic: model::TONICS[1],
            quality: Quality::Major,
        };
        screen.refresh(&changed, &document, &settings);

        assert_ne!(
            screen.edits[&0].positions[0],
            Some(pos(0, 0)),
            "display positions are key-dependent, a key change drops them"
        );
    }

    #[test]
    fn an_instrument_change_clears_the_click_history() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));

        let changed = Session {
            instrument: model::InstrumentPreset::DropD.instrument(),
            tonic: model::TONICS[0],
            quality: Quality::Major,
        };
        screen.refresh(&changed, &document, &settings);

        assert_ne!(
            screen.edits[&0].positions[0],
            Some(pos(0, 0)),
            "display positions are instrument-relative, an instrument change drops them"
        );
    }

    #[test]
    fn slot_edits_keep_the_position_map_aligned() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(4);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        screen.update(
            Message::SlotSelected { chunk: 0, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        screen.update(Message::ToggleRest, &session, &mut document, &settings);
        screen.refresh(&session, &document, &settings);
        screen.update(
            Message::RemoveSlot { chunk: 0, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));

        assert_eq!(
            screen.edits[&0].positions,
            vec![Some(pos(0, 0)), Some(pos(0, 0)), None],
            "toggle and remove keep the position map slot-aligned"
        );
    }

    #[test]
    fn deleting_a_slot_before_the_cursor_shifts_the_cursor_left() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(4);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        click(&mut screen, &session, &mut document, &settings, pos(1, 0));
        screen.update(
            Message::SlotSelected { chunk: 0, slot: 2 },
            &session,
            &mut document,
            &settings,
        );
        screen.update(
            Message::RemoveSlot { chunk: 0, slot: 0 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), Some(-1), None],
            "removing the anchor re-references the octaves to the new first note"
        );
        assert_eq!(
            screen.edits[&0].positions,
            vec![Some(pos(0, 12)), Some(pos(1, 0)), None],
            "the remaining slots keep their sticky positions"
        );
        assert_eq!(
            screen.slot, 1,
            "a deletion before the cursor shifts it left, onto the same slot"
        );
    }

    #[test]
    fn deleting_past_the_cursor_clamps_it_onto_the_new_end() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(3);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        screen.update(
            Message::RemoveSlot { chunk: 0, slot: 2 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(
            screen.slot, 1,
            "deleting the trailing rest clamps the cursor onto the new last slot"
        );
        assert_eq!(document.chunks[0].len(), 2);

        screen.update(
            Message::RemoveSlot { chunk: 0, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);
        screen.update(
            Message::RemoveSlot { chunk: 0, slot: 0 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert!(document.chunks[0].is_empty(), "the last slot deletes too");
        assert_eq!(screen.edits[&0].positions, vec![]);
        assert!(
            !screen.update(
                Message::RemoveSlot { chunk: 0, slot: 0 },
                &session,
                &mut document,
                &settings
            ),
            "deleting from an empty chunk is a no-op"
        );
    }

    #[test]
    fn inserting_between_cells_adds_a_rest_at_that_position() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(3);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        screen.update(
            Message::InsertSlot { chunk: 0, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), None, Some(1), None],
            "the new slot is a rest between the two notes"
        );
        assert_eq!(
            screen.edits[&0].positions,
            vec![Some(pos(0, 0)), None, Some(pos(0, 12)), None],
            "the position map shifts with the tail"
        );
        assert_eq!(screen.slot, 1, "the cursor lands on the fresh slot");
    }

    #[test]
    fn the_trailing_add_appends_a_rest_and_lands_the_cursor() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        let len = document.chunks[0].len();
        screen.update(
            Message::InsertSlot {
                chunk: 0,
                slot: len,
            },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(document.chunks[0].len(), len + 1, "appended");
        assert!(
            matches!(document.chunks[0][len], Slot::Rest),
            "the appended slot is a rest"
        );
        assert_eq!(screen.slot, len, "the cursor lands on the appended slot");
    }

    #[test]
    fn inserting_at_the_front_keeps_the_anchor_note() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(3);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        screen.update(
            Message::InsertSlot { chunk: 0, slot: 0 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![None, Some(0), Some(1), None],
            "a rest inserted before the anchor leaves the octave reference alone"
        );
        assert_eq!(screen.edits[&0].base, Some(2));
    }

    #[test]
    fn insertion_is_bounded_by_the_slot_cap() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(model::MAX_SLOTS);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        assert!(
            !screen.update(
                Message::InsertSlot { chunk: 0, slot: 0 },
                &session,
                &mut document,
                &settings
            ),
            "a full chunk rejects insertion"
        );
        assert_eq!(document.chunks[0].len(), model::MAX_SLOTS);
    }

    #[test]
    fn hovering_an_insert_point_tracks_it_until_the_cursor_leaves() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        assert!(
            !screen.update(
                Message::InsertHovered { chunk: 0, slot: 1 },
                &session,
                &mut document,
                &settings
            ),
            "hovering changes no document"
        );
        assert_eq!(
            screen.hovered_insert, None,
            "the compartment does not expand before the dwell"
        );
        assert!(screen.hover_pending(), "the expansion is scheduled");

        screen.update(Message::HoverTick, &session, &mut document, &settings);
        assert_eq!(screen.hovered_insert, Some((0, 1)));

        screen.update(
            Message::InsertUnhovered { chunk: 0, slot: 2 },
            &session,
            &mut document,
            &settings,
        );
        assert_eq!(
            screen.hovered_insert,
            Some((0, 1)),
            "another point's exit does not clear the hover"
        );

        screen.update(
            Message::InsertUnhovered { chunk: 0, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        assert_eq!(screen.hovered_insert, None);
    }

    #[test]
    fn a_hover_past_the_end_clears_on_refresh() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        screen.update(
            Message::InsertHovered { chunk: 0, slot: 2 },
            &session,
            &mut document,
            &settings,
        );
        screen.update(Message::HoverTick, &session, &mut document, &settings);
        assert_eq!(
            screen.hovered_insert,
            Some((0, 2)),
            "the trailing insert point"
        );

        screen.update(
            Message::RemoveSlot { chunk: 0, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(
            screen.hovered_insert, None,
            "the hovered point no longer exists after the removal"
        );
    }

    #[test]
    fn leaving_before_the_dwell_cancels_the_expansion() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        screen.update(
            Message::InsertHovered { chunk: 0, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        screen.update(
            Message::InsertUnhovered { chunk: 0, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        assert!(!screen.hover_pending(), "leaving cancels the dwell");
        assert_eq!(screen.hovered_insert, None);

        screen.update(Message::HoverTick, &session, &mut document, &settings);
        assert_eq!(
            screen.hovered_insert, None,
            "the timer does not expand a point the cursor left"
        );
    }

    #[test]
    fn a_pending_hover_past_the_end_clears_on_refresh() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        screen.update(
            Message::InsertHovered { chunk: 0, slot: 2 },
            &session,
            &mut document,
            &settings,
        );
        assert!(screen.hover_pending());

        screen.update(
            Message::RemoveSlot { chunk: 0, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(
            screen.hover_pending, None,
            "the pending expansion no longer exists after the removal"
        );
    }

    #[test]
    fn a_fingering_wider_than_the_default_cap_is_still_solved() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));

        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(0, 0), pos(0, 12)])
        );
        assert!(
            screen
                .solved
                .iter()
                .any(|fingering| *fingering == Fingering::new(vec![pos(0, 0), pos(0, 12)])),
            "the displayed fingering is among the solved despite the 12-fret move"
        );
    }

    #[test]
    fn fingering_steps_swap_the_displayed_positions() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(4, 6));

        screen.update(Message::FingeringPrev, &session, &mut document, &settings);
        screen.refresh(&session, &document, &settings);
        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(0, 0), pos(5, 1)]),
            "the solve's top-ranked neighbor becomes the displayed fingering"
        );

        screen.update(Message::FingeringNext, &session, &mut document, &settings);
        screen.refresh(&session, &document, &settings);
        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(0, 0), pos(4, 6)]),
            "stepping back returns to the authored fingering"
        );
    }

    #[test]
    fn rotation_keeps_the_root_note_pinned() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        assert_eq!(
            screen.solved,
            vec![Fingering::new(vec![pos(0, 0)])],
            "a pinned single note has exactly one fingering"
        );

        screen.update(Message::FingeringNext, &session, &mut document, &settings);
        screen.refresh(&session, &document, &settings);
        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(0, 0)]),
            "stepping cannot move the pinned root note"
        );
        assert_eq!(
            screen.edits[&0].base,
            Some(2),
            "the octave reference stays where the root was placed"
        );
    }

    #[test]
    fn every_fingering_keeps_the_root_note_where_it_was_placed() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(4, 0));
        click(&mut screen, &session, &mut document, &settings, pos(4, 8));

        assert!(
            screen
                .solved
                .iter()
                .all(|fingering| fingering.first() == Some(&pos(4, 0))),
            "the pinned root note leads every member of the catalog"
        );
        assert!(
            screen.solved.len() >= 2,
            "the B string open root has multiple voicings of the G4 around it"
        );
    }

    #[test]
    fn rotation_does_not_reshape_the_catalog() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        let catalog = screen.solved.clone();
        assert!(catalog.len() >= 2, "the wide move widens the catalog");

        screen.update(Message::FingeringPrev, &session, &mut document, &settings);
        screen.refresh(&session, &document, &settings);
        assert_eq!(
            screen.solved, catalog,
            "stepping does not re-resolve or re-rank the catalog"
        );
    }

    #[test]
    fn creating_a_variant_copies_the_chunk_and_its_edit_state() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(2);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));

        assert!(
            screen.update(Message::VariantCreated, &session, &mut document, &settings),
            "creating a variant changes the document"
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(document.chunks.len(), 2);
        assert_eq!(
            document.chunks[1].variant_of,
            Some(0),
            "the variant is parented to the root"
        );
        assert_eq!(
            document.chunks[1].chunk, document.chunks[0].chunk,
            "the variant copies the chunk's slots"
        );
        assert_eq!(
            screen.edits[&1].positions, screen.edits[&0].positions,
            "the authored positions carry over"
        );
        assert_eq!(
            screen.edits[&1].base, screen.edits[&0].base,
            "the octave base carries over"
        );
        assert_eq!(
            document.selected_chunk, 1,
            "the variant becomes the edited chunk"
        );
        assert_eq!(screen.slot, 0, "the cursor resets onto the variant");
        assert_eq!(
            screen.positions,
            Fingering::new(vec![pos(0, 0), pos(0, 12)]),
            "the variant opens displaying exactly where its source was"
        );
    }

    #[test]
    fn a_variant_made_from_a_variant_still_parents_to_the_root() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));

        screen.update(Message::VariantCreated, &session, &mut document, &settings);
        screen.refresh(&session, &document, &settings);
        screen.update(Message::VariantCreated, &session, &mut document, &settings);
        screen.refresh(&session, &document, &settings);

        assert_eq!(document.chunks.len(), 3);
        assert_eq!(document.chunks[1].variant_of, Some(0));
        assert_eq!(
            document.chunks[2].variant_of,
            Some(0),
            "a variant of a variant is still parented to the root — lineage is flat"
        );
        assert_eq!(
            document.family(2),
            vec![0, 1, 2],
            "every variant stacks under the one root"
        );
        assert_eq!(
            model::chunk_names(&document.lineage()),
            vec!["A", "A1", "A2"],
            "variants are numbered under their root"
        );
    }

    #[test]
    fn clicking_another_bands_cell_edits_that_chunk_at_that_slot() {
        let session = session();
        let settings = settings();
        let mut document = Document {
            chunks: vec![
                Word::new(vec![Slot::Rest, Slot::Rest, Slot::Rest]).into(),
                model::second_chunk().into(),
            ],
            selected_chunk: 0,
            steps: Vec::new(),
        };
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        let switched = screen.update(
            Message::SlotSelected { chunk: 1, slot: 1 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert!(switched, "switching the edited chunk is a document change");
        assert_eq!(document.selected_chunk, 1);
        assert_eq!(screen.slot, 1, "the cursor lands on the clicked cell");
    }

    #[test]
    fn clicking_the_edited_bands_cell_only_moves_the_cursor() {
        let session = session();
        let settings = settings();
        let mut document = rests_document(3);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);

        let switched = screen.update(
            Message::SlotSelected { chunk: 0, slot: 2 },
            &session,
            &mut document,
            &settings,
        );

        assert!(!switched, "no document change, no refresh needed");
        assert_eq!(document.selected_chunk, 0);
        assert_eq!(screen.slot, 2);
    }

    #[test]
    fn edits_to_another_band_leave_the_edited_chunk_and_cursor_alone() {
        let session = session();
        let settings = settings();
        let mut document = Document {
            chunks: vec![
                Word::new(vec![Slot::Rest, Slot::Rest, Slot::Rest]).into(),
                model::second_chunk().into(),
            ],
            selected_chunk: 0,
            steps: Vec::new(),
        };
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document, &settings);
        click(&mut screen, &session, &mut document, &settings, pos(0, 0));
        click(&mut screen, &session, &mut document, &settings, pos(0, 12));
        assert_eq!(screen.slot, 2, "the cursor sits on the trailing rest");

        screen.update(
            Message::RemoveSlot { chunk: 1, slot: 0 },
            &session,
            &mut document,
            &settings,
        );
        screen.refresh(&session, &document, &settings);

        assert_eq!(
            document.chunks[1].len(),
            1,
            "the other band's slot is removed"
        );
        assert_eq!(document.selected_chunk, 0, "the edited chunk is unchanged");
        assert_eq!(screen.slot, 2, "the cursor is unchanged");
        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), None],
            "the edited chunk's slots are unchanged"
        );
    }
}
