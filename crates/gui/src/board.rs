use iced::widget::canvas;
use iced::{Element, Fill};

use fretboard::fingering::Fingering;
use fretboard::fretboard_position::FretboardPosition;
use fretboard::instrument::Instrument;
use fretboard::tuning;
use relative::key::Key;
use relative::mode::Mode;

use crate::Settings;
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
    // Off-key positions resolved to their spelled degrees — drawn as faint
    // hollow rings until the word plays them, then labeled.
    accidentals: Vec<Dot>,
    string_count: usize,
    max_fret: i32,
    // The open-note SPN label naming each drawn row's string.
    open_notes: Vec<String>,
    label_mode: LabelMode,
}

impl BoardState {
    pub fn new(instrument: &Instrument, key: Key, mode: Mode, label_mode: LabelMode) -> BoardState {
        let (dots, accidentals) = model::board(instrument, key, mode, label_mode);
        let open_notes = (0..instrument.string_count())
            .map(|string_index| tuning::open_note(&instrument.tuning, string_index).to_string())
            .collect();
        BoardState {
            dots,
            accidentals,
            string_count: instrument.string_count(),
            max_fret: instrument.max_fret(),
            open_notes,
            label_mode,
        }
    }

    fn visual(dot: &Dot, accidental: bool, label_mode: LabelMode, highlight: &Highlight) -> Visual {
        Visual {
            position: dot.position,
            label: Some(dot.label.clone()),
            // The subscript octave count rides along on degree labels;
            // note-name labels already carry the octave in SPN.
            octave: match label_mode {
                LabelMode::Degrees => Some(dot.octave),
                LabelMode::Notes => None,
            },
            accidental,
            root_halo: dot.is_root,
            selected: highlight.selected == Some(dot.position),
            emphasized: highlight.positions.contains(&dot.position),
        }
    }

    fn visuals(&self, highlight: &Highlight) -> Vec<Visual> {
        self.dots
            .iter()
            .map(|dot| Self::visual(dot, false, self.label_mode, highlight))
            .chain(
                self.accidentals
                    .iter()
                    .map(|dot| Self::visual(dot, true, self.label_mode, highlight)),
            )
            .collect()
    }
}

pub fn view<'a>(
    board: &'a BoardState,
    highlight: Highlight,
    settings: &'a Settings,
) -> Element<'a, Event> {
    canvas(FretboardCanvas {
        visuals: board.visuals(&highlight),
        string_count: board.string_count,
        max_fret: board.max_fret,
        open_notes: board.open_notes.clone(),
        font_size: settings.font_size,
    })
    .width(Fill)
    .height(Fill)
    .into()
}
