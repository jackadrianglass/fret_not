use iced::widget::canvas;
use iced::{Element, Fill};

use fretboard::fingering::Fingering;
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::tuning;
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
// it. [positions] is the fingering being displayed; `selected` rings one
// position — the edit page points it at the selected slot's position.
pub struct Highlight {
    pub positions: Fingering,
    pub selected: Option<FretboardPosition>,
}

pub struct BoardState {
    dots: Vec<Dot>,
    string_count: usize,
    max_fret: i32,
    // The open-note SPN label naming each drawn row's string.
    open_notes: Vec<String>,
    label_mode: LabelMode,
}

impl BoardState {
    pub fn new(instrument: &Instrument, key: Key, mode: Mode, label_mode: LabelMode) -> BoardState {
        let (dots, _off_key) = model::board(instrument, key, mode, label_mode);
        let open_notes = (0..instrument.string_count())
            .map(|string_index| tuning::open_note(&instrument.tuning, string_index).to_string())
            .collect();
        BoardState {
            dots,
            string_count: instrument.string_count(),
            max_fret: instrument.max_fret(),
            open_notes,
            label_mode,
        }
    }

    fn visuals(&self, highlight: &Highlight) -> Vec<Visual> {
        self.dots
            .iter()
            .map(|dot| Visual {
                position: dot.position,
                label: Some(dot.label.clone()),
                // The subscript octave count rides along on degree labels;
                // note-name labels already carry the octave in SPN.
                octave: match self.label_mode {
                    LabelMode::Degrees => Some(dot.octave),
                    LabelMode::Notes => None,
                },
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
        open_notes: board.open_notes.clone(),
    })
    .width(Fill)
    .height(Fill)
    .into()
}
