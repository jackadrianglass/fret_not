use std::collections::HashMap;

use absolute::spelled_pitch::SpelledPitch;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Fill};

use fretboard::fretboard::degree_at;
use fretboard::fretboard_position::FretboardPosition;
use relative::chunk::{self, Chunk, Slot};
use relative::degree_reference::DegreeReference;
use relative::key::Quality;

use crate::board::{self, BoardState, Highlight};
use crate::colors;
use crate::model;
use crate::{Document, Session, outlined};

const BOARD_HEIGHT: f32 = 260.0;

// The per-chunk editing context, keyed by chunk index: the register the
// chunk's octaves count from and the fingering being displayed — one
// position per slot, set by the user's clicks and filled from the solve
// once, sticky afterwards.
struct ChunkEdit {
    // The key-anchored register of the chunk's anchor note — its first
    // note. Established by the first note written into the chunk (or by
    // where the solve voices it, until then) and moved when the anchor note
    // itself is rewritten.
    base: Option<i32>,
    // Slot-aligned display positions; None for rests and slots not yet
    // placed. A position only changes when the user touches the slot.
    positions: Vec<Option<FretboardPosition>>,
}

pub struct EditScreen {
    slot: usize,
    edits: HashMap<usize, ChunkEdit>,
    solved: Vec<Vec<FretboardPosition>>,
    slot_notes: Vec<Option<usize>>,
    // The emphasized positions: the selected chunk's displayed fingering,
    // slot-aligned in [edits] and projected here for the view.
    positions: Vec<FretboardPosition>,
    // The chunk + session the caches were derived against; a change resets
    // the cursor and, on a key change, the click history.
    anchored: Option<(usize, SpelledPitch, Quality)>,
}

// The index of the chunk's first note — the anchor note its octaves count
// from and whose position centers the solve window.
fn anchor_index(chunk: &Chunk) -> Option<usize> {
    chunk.iter().position(|slot| matches!(slot, Slot::Note(_)))
}

// The chunk's first note is the octave anchor; after a mutation that can
// change which note is first, restore the invariant — the anchor sits at
// octave 0 — without moving any sounding pitch.
fn reanchor(chunk: &mut Chunk, edit: &mut ChunkEdit) {
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
fn displayed_fret_span(edit: &ChunkEdit, chunk: &Chunk) -> i32 {
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
    SlotSelected(usize),
    AddSlot,
    RemoveSlot,
    ToggleRest,
}

impl EditScreen {
    pub fn new() -> EditScreen {
        EditScreen {
            slot: 0,
            edits: HashMap::new(),
            solved: Vec::new(),
            slot_notes: Vec::new(),
            positions: Vec::new(),
            anchored: None,
        }
    }

    // The selected chunk's octave reference register — the anchor note's.
    fn base(&self, document: &Document) -> Option<i32> {
        self.edits
            .get(&document.selected_chunk)
            .and_then(|edit| edit.base)
    }

    pub fn refresh(&mut self, session: &Session, document: &Document) {
        let anchored = (document.selected_chunk, session.tonic, session.quality);
        if self.anchored != Some(anchored) {
            // A key change invalidates every chunk's click history; a chunk
            // switch keeps the per-chunk edits.
            let (_, tonic, quality) = self.anchored.unwrap_or(anchored);
            if (tonic, quality) != (session.tonic, session.quality) {
                self.edits.clear();
            }
            self.anchored = Some(anchored);
            self.slot = 0;
        }
        let chunk = &document.chunks[document.selected_chunk];
        let edit = self
            .edits
            .entry(document.selected_chunk)
            .or_insert_with(|| ChunkEdit {
                base: None,
                positions: vec![None; chunk.len()],
            });
        // The solve window centers on the anchor note's displayed position.
        let window = anchor_index(chunk)
            .and_then(|i| edit.positions.get(i).copied().flatten())
            .unwrap_or(model::SOLVER_ANCHOR);
        // The fret cap widens to admit the displayed fingering: every
        // consecutive move the chunk already shows must be inside the
        // solver's reach, so the displayed fingering is always among the
        // enumerated.
        let cap = displayed_fret_span(edit, chunk).max(model::SOLVER_FRET_CAP);
        let (solved, slot_notes) = model::solve(
            &session.instrument,
            session.key(),
            session.mode(),
            chunk,
            1,
            window,
            cap,
        );
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
        self.positions = positions;
    }

    // True when the document changed and the other screens' caches are stale.
    pub fn update(&mut self, message: Message, session: &Session, document: &mut Document) -> bool {
        match message {
            Message::Board(board::Event::Clicked(position)) => {
                write_position(self, session, document, position)
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
                if let Some(edit) = self.edits.get_mut(&document.selected_chunk) {
                    edit.positions.push(None);
                }
                true
            }
            Message::RemoveSlot => {
                let chunk = &mut document.chunks[document.selected_chunk];
                if chunk.is_empty() {
                    return false;
                }
                let index = self.slot.min(chunk.len() - 1);
                chunk::remove(chunk, index);
                self.slot = self.slot.saturating_sub(1);
                if let Some(edit) = self.edits.get_mut(&document.selected_chunk) {
                    edit.positions.remove(index);
                    reanchor(chunk, edit);
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
        }
    }
}

// A click writes the selected slot and steps the cursor. degree_at is
// key-anchored; the slot stores it relative to the anchor note — the
// chunk's first note, the register the octaves count from. Writing the
// anchor note moves the reference (the other notes shift to keep their
// sounding pitches); any other note stores its register distance from it.
// The clicked position becomes the slot's display position — it sticks
// until the user touches the slot again.
fn write_position(
    screen: &mut EditScreen,
    session: &Session,
    document: &mut Document,
    position: FretboardPosition,
) -> bool {
    let chunk = &mut document.chunks[document.selected_chunk];
    if chunk.is_empty() {
        return false;
    }
    let index = screen.slot.min(chunk.len() - 1);
    let clicked = degree_at(&session.instrument, session.key(), session.mode(), position);
    screen
        .edits
        .entry(document.selected_chunk)
        .or_insert_with(|| ChunkEdit {
            base: None,
            positions: vec![None; chunk.len()],
        });
    let edit = screen.edits.get_mut(&document.selected_chunk).unwrap();
    let slot = match clicked {
        Some(dr) => {
            // The write lands on the anchor when it writes the chunk's first
            // note — at, before, or into a note-less chunk.
            let writes_anchor = match anchor_index(chunk) {
                Some(anchor) => index <= anchor,
                None => true,
            };
            let slot = match edit.base {
                Some(base) if !writes_anchor => Slot::Note(DegreeReference {
                    octave: dr.octave - base,
                    ..dr
                }),
                Some(base) => {
                    chunk::shift_octaves(chunk, base - dr.octave);
                    edit.base = Some(dr.octave);
                    Slot::Note(DegreeReference { octave: 0, ..dr })
                }
                None => {
                    edit.base = Some(dr.octave);
                    Slot::Note(DegreeReference { octave: 0, ..dr })
                }
            };
            edit.positions[index] = Some(position);
            slot
        }
        None => {
            edit.positions[index] = None;
            Slot::Rest
        }
    };
    chunk::set(chunk, index, slot);
    if index + 1 < chunk.len() {
        screen.slot = index + 1;
    } else if chunk.len() < model::MAX_SLOTS {
        chunk::push(chunk, Slot::Rest);
        edit.positions.push(None);
        screen.slot = index + 1;
    }
    true
}

pub fn view<'a>(
    screen: &'a EditScreen,
    session: &'a Session,
    document: &'a Document,
    board: &'a BoardState,
) -> Element<'a, Message> {
    let chunk = &document.chunks[document.selected_chunk];
    let base = screen.base(document);

    let band = container(slot_band(screen, session, base, chunk))
        .width(Fill)
        .height(Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    let board_area = container(
        board::view(
            board,
            Highlight {
                selected: selected_slot_position(screen, document),
                positions: screen.positions.clone(),
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

fn slot_band<'a>(
    screen: &'a EditScreen,
    session: &'a Session,
    base: Option<i32>,
    chunk: &'a Chunk,
) -> Element<'a, Message> {
    let mut band = row![].spacing(6).align_y(Alignment::Center);

    for (i, slot) in chunk.iter().enumerate() {
        let is_selected = i == screen.slot;
        let cell = match slot {
            Slot::Note(dr) => {
                let name = match base {
                    Some(base) => {
                        model::degree_note_in_register(session.key(), session.mode(), base, *dr)
                            .to_string()
                    }
                    None => String::new(),
                };
                column![
                    text(model::slot_label(*dr)).size(16.0).style(accent),
                    text(name).size(11.0).style(secondary),
                ]
            }
            Slot::Rest => column![
                text(".").size(16.0).style(secondary),
                text("rest").size(11.0).style(secondary)
            ],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;

    fn session() -> Session {
        Session {
            instrument: model::instrument(),
            tonic: model::TONICS[0],
            quality: Quality::Major,
            label_mode: model::LabelMode::Degrees,
        }
    }

    fn rests_document(slots: usize) -> Document {
        Document {
            chunks: vec![Chunk::new(vec![Slot::Rest; slots])],
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
        position: FretboardPosition,
    ) {
        screen.update(
            Message::Board(board::Event::Clicked(position)),
            session,
            document,
        );
        screen.refresh(session, document);
    }

    #[test]
    fn the_same_degree_an_octave_apart_stores_distinct_octaves() {
        let session = session();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        click(&mut screen, &session, &mut document, pos(0, 0));
        click(&mut screen, &session, &mut document, pos(0, 12));

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), None],
            "open low E then its octave: degree 3 at chunk octaves 0 and 1, appended rest behind the cursor"
        );
        assert_eq!(
            screen.positions,
            vec![pos(0, 0), pos(0, 12)],
            "both clicks display where they were clicked"
        );
    }

    #[test]
    fn a_click_below_the_anchor_stores_a_negative_octave() {
        let session = session();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        click(&mut screen, &session, &mut document, pos(0, 12));
        click(&mut screen, &session, &mut document, pos(0, 0));

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), Some(-1), None],
            "the anchor note stays at octave 0, the lower note counts below it"
        );
        assert_eq!(
            screen.edits[&0].base,
            Some(3),
            "the anchor keeps its register"
        );
        assert_eq!(
            screen.positions,
            vec![pos(0, 12), pos(0, 0)],
            "both clicks display where they were clicked"
        );
    }

    #[test]
    fn an_off_key_click_writes_a_rest_and_advances_the_cursor() {
        let session = session();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        click(&mut screen, &session, &mut document, pos(0, 6));

        assert_eq!(screen.slot, 1, "the cursor steps right");
        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![None, None],
            "Bb on the low E string is off-key in C major: the slot becomes a rest"
        );

        click(&mut screen, &session, &mut document, pos(0, 0));

        assert_eq!(
            octave_of(document.chunks[0][1]),
            Some(0),
            "the first in-key note establishes the base at octave 0"
        );
    }

    #[test]
    fn writing_past_the_end_appends_up_to_the_slot_cap() {
        let session = session();
        let mut document = rests_document(model::MAX_SLOTS);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        for _ in 0..(model::MAX_SLOTS + 3) {
            click(&mut screen, &session, &mut document, pos(0, 0));
        }

        assert_eq!(
            document.chunks[0].len(),
            model::MAX_SLOTS,
            "the chunk stops growing at the cap"
        );
        assert_eq!(
            screen.slot,
            model::MAX_SLOTS - 1,
            "the cursor pins at the last slot"
        );
        assert!(
            document.chunks[0]
                .iter()
                .all(|slot| matches!(slot, Slot::Note(_))),
            "every rest was written over"
        );
    }

    #[test]
    fn switching_chunks_resets_the_cursor_and_rederives_the_base() {
        let session = session();
        let mut document = Document {
            chunks: vec![Chunk::new(vec![Slot::Rest]), model::second_chunk()],
            selected_chunk: 0,
            steps: Vec::new(),
        };
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        click(&mut screen, &session, &mut document, pos(0, 0));
        assert_eq!(screen.slot, 1);

        document.selected_chunk = 1;
        screen.refresh(&session, &document);

        assert_eq!(screen.slot, 0, "the cursor resets on a chunk switch");
        assert!(
            screen.base(&document).is_some(),
            "the new chunk's anchor note establishes the base"
        );
    }

    #[test]
    fn authored_clicks_beat_the_solve_where_they_disagree() {
        let session = session();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        click(&mut screen, &session, &mut document, pos(0, 0));
        click(&mut screen, &session, &mut document, pos(4, 6));

        assert_eq!(
            screen.solved.first().map(|fingering| fingering.as_slice()),
            Some(&[pos(0, 0), pos(5, 1)][..]),
            "the solve re-voices slot 1's F4 to the high E string"
        );
        assert_eq!(
            screen.positions,
            vec![pos(0, 0), pos(4, 6)],
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
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        click(&mut screen, &session, &mut document, pos(0, 12));
        click(&mut screen, &session, &mut document, pos(0, 0));
        screen.update(Message::SlotSelected(0), &session, &mut document);
        click(&mut screen, &session, &mut document, pos(0, 0));

        assert_eq!(
            document.chunks[0]
                .iter()
                .map(|slot| octave_of(*slot))
                .collect::<Vec<_>>(),
            vec![Some(0), Some(0), None],
            "the rewritten anchor keeps octave 0, the other note shifts to keep its register"
        );
        assert_eq!(screen.edits[&0].base, Some(2));
    }

    #[test]
    fn removing_the_anchor_rebases_to_the_new_first_note() {
        let session = session();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        click(&mut screen, &session, &mut document, pos(0, 12));
        click(&mut screen, &session, &mut document, pos(0, 0));
        screen.update(Message::SlotSelected(0), &session, &mut document);
        screen.update(Message::RemoveSlot, &session, &mut document);
        screen.refresh(&session, &document);

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
        let mut document = Document {
            chunks: vec![model::second_chunk()],
            selected_chunk: 0,
            steps: Vec::new(),
        };
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        let filled = screen.positions[1];
        click(&mut screen, &session, &mut document, pos(4, 0));

        assert_eq!(
            screen.positions,
            vec![pos(4, 0), filled],
            "the filled position sticks across edits; only the touched slot moves"
        );
        assert_ne!(
            screen.solved.first().map(|fingering| fingering.as_slice()),
            Some(&screen.positions[..]),
            "the solve would have re-voiced slot 1 — the fill must not follow it"
        );
    }

    #[test]
    fn chunk_switches_keep_each_chunks_click_history() {
        let session = session();
        let mut document = Document {
            chunks: vec![Chunk::new(vec![Slot::Rest]), model::second_chunk()],
            selected_chunk: 0,
            steps: Vec::new(),
        };
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);
        click(&mut screen, &session, &mut document, pos(0, 0));

        document.selected_chunk = 1;
        screen.refresh(&session, &document);
        document.selected_chunk = 0;
        screen.refresh(&session, &document);

        assert_eq!(
            screen.positions.first(),
            Some(&pos(0, 0)),
            "the click history survives the round trip"
        );
    }

    #[test]
    fn a_key_change_clears_the_click_history() {
        let session = session();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);
        click(&mut screen, &session, &mut document, pos(0, 0));

        let changed = Session {
            instrument: model::instrument(),
            tonic: model::TONICS[1],
            quality: Quality::Major,
            label_mode: model::LabelMode::Degrees,
        };
        screen.refresh(&changed, &document);

        assert_ne!(
            screen.edits[&0].positions[0],
            Some(pos(0, 0)),
            "display positions are key-dependent, a key change drops them"
        );
    }

    #[test]
    fn slot_edits_keep_the_position_map_aligned() {
        let session = session();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        click(&mut screen, &session, &mut document, pos(0, 0));
        screen.update(Message::ToggleRest, &session, &mut document);
        screen.refresh(&session, &document);
        screen.update(Message::RemoveSlot, &session, &mut document);
        screen.refresh(&session, &document);
        screen.update(Message::AddSlot, &session, &mut document);
        screen.refresh(&session, &document);

        assert_eq!(
            screen.edits[&0].positions,
            vec![Some(pos(0, 0)), None],
            "toggle, remove, and add keep the position map slot-aligned"
        );
    }

    #[test]
    fn a_fingering_wider_than_the_default_cap_is_still_solved() {
        let session = session();
        let mut document = rests_document(1);
        let mut screen = EditScreen::new();
        screen.refresh(&session, &document);

        click(&mut screen, &session, &mut document, pos(0, 0));
        click(&mut screen, &session, &mut document, pos(0, 12));

        assert_eq!(screen.positions, vec![pos(0, 0), pos(0, 12)]);
        assert!(
            screen
                .solved
                .iter()
                .any(|fingering| fingering == &vec![pos(0, 0), pos(0, 12)]),
            "the displayed fingering is among the solved despite the 12-fret move"
        );
    }
}
