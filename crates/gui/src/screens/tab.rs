use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Fill, Font};

use fretboard::fretboard_position::FretboardPosition;
use fretboard::tuning;

use crate::board::{self, BoardState, Highlight};
use crate::model;
use crate::{Document, Session};

pub struct TabScreen {
    chord_index: usize,
    fingering_index: usize,
    solved: Vec<Vec<FretboardPosition>>,
    slot_notes: Vec<Option<usize>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Board(board::Event),
    FingeringPrev,
    FingeringNext,
    ChordPrev,
    ChordNext,
}

impl TabScreen {
    pub fn new() -> TabScreen {
        TabScreen {
            chord_index: 0,
            fingering_index: 0,
            solved: Vec::new(),
            slot_notes: Vec::new(),
        }
    }

    pub fn refresh(&mut self, session: &Session, document: &Document) {
        let chunk = &document.chunks[document.selected_chunk];
        let (solved, slot_notes) = model::solve(
            &session.instrument,
            session.key(),
            session.mode(),
            chunk,
            model::PROGRESSION[self.chord_index],
        );
        self.solved = solved;
        self.slot_notes = slot_notes;
        self.fingering_index = 0;
    }

    // True when the chord changed and the cached fingerings are stale.
    pub fn update(&mut self, message: Message) -> bool {
        match message {
            Message::Board(board::Event::Clicked(position)) => {
                if let Some(index) = self
                    .solved
                    .iter()
                    .position(|fingering| fingering.contains(&position))
                {
                    self.fingering_index = index;
                }
                false
            }
            Message::FingeringPrev => {
                let len = self.solved.len();
                if len > 0 {
                    self.fingering_index = (self.fingering_index + len - 1) % len;
                }
                false
            }
            Message::FingeringNext => {
                let len = self.solved.len();
                if len > 0 {
                    self.fingering_index = (self.fingering_index + 1) % len;
                }
                false
            }
            Message::ChordPrev => {
                self.chord_index =
                    (self.chord_index + model::PROGRESSION.len() - 1) % model::PROGRESSION.len();
                true
            }
            Message::ChordNext => {
                self.chord_index = (self.chord_index + 1) % model::PROGRESSION.len();
                true
            }
        }
    }
}

pub fn view<'a>(
    screen: &'a TabScreen,
    session: &'a Session,
    document: &'a Document,
    board: &'a BoardState,
) -> Element<'a, Message> {
    let chunk = &document.chunks[document.selected_chunk];
    let fingering = screen
        .solved
        .get(screen.fingering_index)
        .cloned()
        .unwrap_or_default();

    let controls = row![
        button("prev").on_press(Message::FingeringPrev),
        button("next").on_press(Message::FingeringNext),
        text(format!(
            "fingering {}/{}",
            screen.fingering_index + 1,
            screen.solved.len()
        )),
        button("< chord").on_press(Message::ChordPrev),
        text(model::PROGRESSION_LABELS[screen.chord_index]),
        button("chord >").on_press(Message::ChordNext),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let lines: Vec<String> = (0..session.instrument.string_count())
        .map(|string_index| {
            let mut line = format!(
                "{:>3}|",
                tuning::open_note(&session.instrument.tuning, string_index)
            );
            for i in 0..chunk.len() {
                let played = screen
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

    let tab = column(
        lines
            .into_iter()
            .map(|line| text(line).font(Font::MONOSPACE).size(16.0).into())
            .collect::<Vec<Element<'_, Message>>>(),
    );

    let tab_area = container(tab).height(Fill).padding(20);
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

    column![controls, tab_area, board_area]
        .spacing(12)
        .height(Fill)
        .width(Fill)
        .into()
}
