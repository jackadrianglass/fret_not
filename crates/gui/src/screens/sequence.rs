use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Font};

use fretboard::fretboard_position::FretboardPosition;
use relative::chunk::Slot;

use crate::board::{self, BoardState, Highlight};
use crate::colors;
use crate::model;
use crate::{Document, Session, outlined};

pub struct SequenceScreen {
    step: usize,
    solved: Vec<Vec<FretboardPosition>>,
    slot_notes: Vec<Option<usize>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Board(board::Event),
    StepSelected(usize),
}

impl SequenceScreen {
    pub fn new() -> SequenceScreen {
        SequenceScreen {
            step: 0,
            solved: Vec::new(),
            slot_notes: Vec::new(),
        }
    }

    pub fn refresh(&mut self, session: &Session, document: &Document) {
        let Some(step) = document.steps.get(self.step) else {
            return;
        };
        let chunk = &document.chunks[step.chunk];
        let (solved, slot_notes) = model::solve(
            &session.instrument,
            session.key(),
            session.mode(),
            chunk,
            step.root,
        );
        self.solved = solved;
        self.slot_notes = slot_notes;
    }

    // True when the selected step changed and the cached fingering is stale.
    pub fn update(&mut self, message: Message, document: &Document) -> bool {
        match message {
            Message::Board(board::Event::Clicked(_)) => false,
            Message::StepSelected(index) => {
                let changed = index < document.steps.len() && index != self.step;
                if changed {
                    self.step = index;
                }
                changed
            }
        }
    }

    pub fn step_label(&self, document: &Document) -> String {
        match document.steps.get(self.step) {
            Some(step) => format!(
                "step {} / {} — chunk {} over {}",
                self.step + 1,
                document.steps.len(),
                model::chunk_name(step.chunk),
                model::roman(step.root)
            ),
            None => "no steps".to_string(),
        }
    }
}

fn secondary(theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(colors::secondary_text(theme)),
    }
}

pub fn view<'a>(
    screen: &'a SequenceScreen,
    _session: &'a Session,
    document: &'a Document,
    board: &'a BoardState,
) -> Element<'a, Message> {
    let mut cards = row![].spacing(6).align_y(Alignment::Center);
    for (i, step) in document.steps.iter().enumerate() {
        let chunk = &document.chunks[step.chunk];
        let glyphs = chunk
            .iter()
            .map(|slot| match slot {
                Slot::Note(dr) => model::degree_label(*dr),
                Slot::Rest => ".".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        let card = column![
            text(model::roman(step.root)).size(12.0).style(secondary),
            text(model::chunk_name(step.chunk)).size(18.0),
            text(glyphs)
                .size(10.0)
                .font(Font::MONOSPACE)
                .style(secondary),
        ];
        cards = cards.push(
            button(container(card).padding(8))
                .on_press(Message::StepSelected(i))
                .style(outlined(i == screen.step)),
        );
    }

    let timeline = container(scrollable(cards).horizontal()).height(Fill);
    let fingering = screen.solved.first().cloned().unwrap_or_default();
    let board_area = container(
        board::view(
            board,
            Highlight {
                positions: fingering,
                selected: None,
            },
        )
        .map(Message::Board),
    )
    .height(260);

    column![timeline, board_area]
        .spacing(12)
        .height(Fill)
        .width(Fill)
        .into()
}
