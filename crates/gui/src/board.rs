use iced::widget::canvas;
use iced::{Element, Fill};

use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use relative::key::Key;
use relative::mode::Mode;

use crate::fretboard_canvas::{FretboardCanvas, Visual};
use crate::model::{self, Dot, LabelMode};

#[derive(Debug, Clone)]
pub enum Event {
    Clicked(FretboardPosition),
}

// The merged board presentation, shared by every screen: the full in-key
// field as a dim backdrop, with the played positions emphasized on top of
// it. `selected` rings one position — the edit page points it at the
// selected slot's position.
pub struct Highlight {
    pub positions: Vec<FretboardPosition>,
    pub selected: Option<FretboardPosition>,
}

pub struct BoardState {
    dots: Vec<Dot>,
    string_count: usize,
    max_fret: i32,
}

impl BoardState {
    pub fn new(instrument: &Instrument, key: Key, mode: Mode, label_mode: LabelMode) -> BoardState {
        let (dots, _off_key) = model::board(instrument, key, mode, label_mode);
        BoardState {
            dots,
            string_count: instrument.string_count(),
            max_fret: instrument.max_fret(),
        }
    }

    fn visuals(&self, highlight: &Highlight) -> Vec<Visual> {
        self.dots
            .iter()
            .map(|dot| Visual {
                position: dot.position,
                label: Some(dot.label.clone()),
                root_halo: dot.is_root,
                selected: highlight.selected == Some(dot.position),
                emphasized: highlight.positions.contains(&dot.position),
            })
            .collect()
    }
}

pub fn view<'a>(board: &'a BoardState, highlight: Highlight) -> Element<'a, Event> {
    canvas(FretboardCanvas {
        visuals: board.visuals(&highlight),
        string_count: board.string_count,
        max_fret: board.max_fret,
    })
    .width(Fill)
    .height(Fill)
    .into()
}
