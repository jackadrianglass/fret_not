use iced::widget::{button, canvas, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill};

use fretboard::fingering::Fingering;
use fretboard::tuning;

use crate::board::{self, BoardState, Highlight};
use crate::model;
use crate::tab_canvas::{self, TabCanvas};
use crate::{Document, Session, Settings};

// One bar per sixteen slots — slots are sixteenth notes, so a bar reads as 4/4.
const COLUMNS_PER_BAR: usize = 16;

pub struct TabScreen {
    chord_index: usize,
    fingering_index: usize,
    solved: Vec<Fingering>,
    slot_notes: Vec<Option<usize>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Board(board::Event),
    TabCanvas(tab_canvas::Event),
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

    pub fn refresh(&mut self, session: &Session, document: &Document, settings: &Settings) {
        let chunk = &document.chunks[document.selected_chunk];
        let (solved, slot_notes) = model::solve(
            &session.instrument,
            session.key(),
            session.mode(),
            chunk,
            model::PROGRESSION[self.chord_index],
            model::SOLVER_ANCHOR,
            settings.solver_fret_cap,
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
            Message::TabCanvas(_) => false,
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

// The staff's column count: whole bars only. A trailing partial bar is
// padded with rest columns up to the bar grid, so every bar is the same
// size and the grid stays aligned.
fn padded_columns(slots: usize, columns_per_bar: usize) -> usize {
    let per_bar = columns_per_bar.max(1);
    slots.div_ceil(per_bar) * per_bar
}

// The fret grid the tab paints: one row per string, one column per slot,
// None where the string does not sound in that slot — a rest, or a note
// played on another string. The grid spans [columns] columns, which may
// outrun the slots when a partial bar is padded.
fn fret_grid(
    fingering: &Fingering,
    slot_notes: &[Option<usize>],
    string_count: usize,
    columns: usize,
) -> Vec<Vec<Option<i32>>> {
    let mut grid = vec![vec![None; columns]; string_count];
    for (slot, note) in slot_notes.iter().enumerate() {
        if let Some(position) = note.and_then(|index| fingering.get(index).copied())
            && let Some(row) = grid.get_mut(position.string_index)
        {
            row[slot] = Some(position.fret);
        }
    }
    grid
}

pub fn view<'a>(
    screen: &'a TabScreen,
    session: &'a Session,
    document: &'a Document,
    board: &'a BoardState,
    settings: &'a Settings,
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

    let string_count = session.instrument.string_count();
    let open_notes = (0..string_count)
        .map(|string_index| tuning::open_note(&session.instrument.tuning, string_index).to_string())
        .collect();
    let columns = padded_columns(chunk.len(), COLUMNS_PER_BAR);
    let mut rests: Vec<bool> = screen
        .slot_notes
        .iter()
        .map(|slot| slot.is_none())
        .collect();
    rests.resize(columns, true);
    let staff = TabCanvas {
        string_count,
        open_notes,
        columns,
        frets: fret_grid(&fingering, &screen.slot_notes, string_count, columns),
        rests,
        columns_per_bar: COLUMNS_PER_BAR,
        font_size: settings.font_size,
    };
    let staff_width = staff.content_width();
    let staff_element: Element<'_, tab_canvas::Event> =
        canvas(staff).width(staff_width).height(Fill).into();
    let tab_area = container(scrollable(staff_element.map(Message::TabCanvas)).horizontal())
        .height(Fill)
        .padding(20);

    let board_area = container(
        board::view(
            board,
            Highlight {
                positions: fingering,
                selected: None,
            },
            settings,
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

#[cfg(test)]
mod tests {
    use super::*;
    use fretboard::fretboard_position::FretboardPosition;

    fn pos(string_index: usize, fret: i32) -> FretboardPosition {
        FretboardPosition { string_index, fret }
    }

    #[test]
    fn the_staff_pads_a_partial_bar_to_whole_bars() {
        assert_eq!(
            padded_columns(12, 8),
            16,
            "a bar and a half pads to two bars"
        );
        assert_eq!(padded_columns(8, 8), 8, "a full bar stays as it is");
        assert_eq!(padded_columns(4, 8), 8, "a half bar pads to one bar");
        assert_eq!(
            padded_columns(12, 16),
            16,
            "sixteenths: three beats pad to a full bar"
        );
        assert_eq!(padded_columns(16, 16), 16, "a full bar of sixteenths stays");
        assert_eq!(padded_columns(0, 8), 0, "no slots, no staff");
    }

    #[test]
    fn notes_land_on_their_string_row_and_rests_leave_columns_empty() {
        let fingering = Fingering::new(vec![pos(2, 5), pos(0, 12)]);
        let slot_notes = vec![Some(0), None, Some(1)];
        let grid = fret_grid(&fingering, &slot_notes, 6, slot_notes.len());
        assert_eq!(grid.len(), 6, "one row per string");
        assert_eq!(grid[2][0], Some(5), "the first note on its string's row");
        assert_eq!(
            grid[0][2],
            Some(12),
            "a two-digit fret in its slot's column"
        );
        assert!(
            grid.iter().all(|row| row[1].is_none()),
            "a rest slot leaves every string empty"
        );
        assert_eq!(
            grid.iter().flatten().filter(|cell| cell.is_some()).count(),
            2,
            "only the played cells carry frets"
        );
    }

    #[test]
    fn positions_beyond_the_string_count_paint_nothing() {
        let fingering = Fingering::new(vec![pos(9, 3)]);
        let grid = fret_grid(&fingering, &[Some(0)], 6, 1);
        assert!(
            grid.iter().flatten().all(|cell| cell.is_none()),
            "a position off the instrument's strings paints nothing"
        );
    }

    #[test]
    fn padding_columns_stay_empty() {
        let fingering = Fingering::new(vec![pos(2, 5)]);
        let grid = fret_grid(&fingering, &[Some(0)], 6, 8);
        assert_eq!(grid[0].len(), 8, "the grid spans the padded bars");
        assert_eq!(grid[2][0], Some(5));
        assert!(
            grid.iter()
                .all(|row| row[1..].iter().all(|cell| cell.is_none())),
            "padding columns carry no frets"
        );
    }
}
