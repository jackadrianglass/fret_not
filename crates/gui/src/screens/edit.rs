use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Fill};

use fretboard::fretboard::{degree_at, degree_note};
use fretboard::fretboard_position::FretboardPosition;
use relative::chunk::{self, Chunk, Slot};
use relative::degree_reference::DegreeReference;

use crate::board::{self, BoardState, Highlight};
use crate::colors;
use crate::model;
use crate::{Document, Session, outlined};

const BOARD_HEIGHT: f32 = 260.0;

pub struct EditScreen {
    slot: usize,
    solved: Vec<Vec<FretboardPosition>>,
    slot_notes: Vec<Option<usize>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Board(board::Event),
    SlotSelected(usize),
    AddSlot,
    RemoveSlot,
    ToggleRest,
}

impl EditScreen {
    pub fn new() -> EditScreen {
        EditScreen {
            slot: 0,
            solved: Vec::new(),
            slot_notes: Vec::new(),
        }
    }

    pub fn refresh(&mut self, session: &Session, document: &Document) {
        let chunk = &document.chunks[document.selected_chunk];
        let (solved, slot_notes) =
            model::solve(&session.instrument, session.key(), session.mode(), chunk, 1);
        self.solved = solved;
        self.slot_notes = slot_notes;
    }

    // True when the document changed and the other screens' caches are stale.
    pub fn update(&mut self, message: Message, session: &Session, document: &mut Document) -> bool {
        match message {
            Message::Board(board::Event::Clicked(position)) => {
                let chunk = &mut document.chunks[document.selected_chunk];
                if chunk.is_empty() {
                    return false;
                }
                let index = self.slot.min(chunk.len() - 1);
                let slot = degree_at(&session.instrument, session.key(), session.mode(), position)
                    .map(Slot::Note)
                    .unwrap_or(Slot::Rest);
                chunk::set(chunk, index, slot);
                true
            }
            Message::SlotSelected(index) => {
                let len = document.chunks[document.selected_chunk].len();
                if len > 0 {
                    self.slot = index.min(len - 1);
                }
                false
            }
            Message::AddSlot => {
                let chunk = &mut document.chunks[document.selected_chunk];
                if chunk.len() >= model::MAX_SLOTS {
                    return false;
                }
                chunk::push(chunk, Slot::Rest);
                true
            }
            Message::RemoveSlot => {
                let chunk = &mut document.chunks[document.selected_chunk];
                if chunk.is_empty() {
                    return false;
                }
                chunk::remove(chunk, self.slot.min(chunk.len() - 1));
                self.slot = self.slot.saturating_sub(1);
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
                true
            }
        }
    }
}

pub fn view<'a>(
    screen: &'a EditScreen,
    session: &'a Session,
    document: &'a Document,
    board: &'a BoardState,
) -> Element<'a, Message> {
    let chunk = &document.chunks[document.selected_chunk];
    let fingering = screen.solved.first().cloned().unwrap_or_default();

    let band = container(slot_band(screen, session, chunk))
        .width(Fill)
        .height(Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    let board_area = container(
        board::view(
            board,
            Highlight {
                selected: selected_slot_position(screen, &fingering),
                positions: fingering,
            },
        )
        .map(Message::Board),
    )
    .height(BOARD_HEIGHT);

    column![band, board_area]
        .spacing(12)
        .height(Fill)
        .width(Fill)
        .into()
}

fn selected_slot_position(
    screen: &EditScreen,
    fingering: &[FretboardPosition],
) -> Option<FretboardPosition> {
    screen
        .slot_notes
        .get(screen.slot)
        .and_then(|note| *note)
        .and_then(|index| fingering.get(index))
        .copied()
}

fn secondary(theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(colors::secondary_text(theme)),
    }
}

fn slot_band<'a>(
    screen: &'a EditScreen,
    session: &'a Session,
    chunk: &'a Chunk,
) -> Element<'a, Message> {
    let mut band = row![].spacing(6).align_y(Alignment::Center);

    for (i, slot) in chunk.iter().enumerate() {
        let is_selected = i == screen.slot;
        let cell = match slot {
            Slot::Note(dr) => {
                let note = degree_note(session.key(), session.mode(), *dr);
                column![
                    text(model::degree_label(*dr)).size(16.0),
                    text(note.spelled_pitch.to_string())
                        .size(11.0)
                        .style(secondary),
                ]
            }
            Slot::Rest => {
                column![
                    text(".").size(16.0),
                    text("rest").size(11.0).style(secondary)
                ]
            }
        };
        band = band.push(
            button(cell.padding(6))
                .on_press(Message::SlotSelected(i))
                .style(outlined(is_selected)),
        );
    }

    band.push(button("+").on_press(Message::AddSlot))
        .push(button("-").on_press(Message::RemoveSlot))
        .push(button("rest").on_press(Message::ToggleRest))
        .into()
}
