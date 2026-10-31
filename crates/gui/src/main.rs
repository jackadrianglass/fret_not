mod board;
mod colors;
mod fretboard_canvas;
mod model;
mod screens;
mod tab_canvas;

use std::ops::{Deref, DerefMut};
use std::time::Duration;

use iced::widget::{Stack, button, column, container, pick_list, row, slider, space, text};
use iced::{Border, Element, Fill, Subscription};

use absolute::spelled_pitch::SpelledPitch;
use fretboard::instrument::Instrument;
use relative::chunk::{Slot, Word};
use relative::key::Quality;

use crate::board::BoardState;
use crate::model::{BASE_FONT_SIZE, LabelMode};
use crate::screens::{ScreenKind, Screens, edit, sequence, settings, tab};

fn main() -> iced::Result {
    iced::application(App::new, update, view)
        .title("fret_not [tab v7]")
        .theme(theme)
        .subscription(subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1400.0, 900.0),
            min_size: Some(iced::Size::new(1000.0, 620.0)),
            resizable: true,
            ..iced::window::Settings::default()
        })
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    TonicPicked(SpelledPitch),
    QualityPicked(Quality),
    TempoChanged(u16),
    ScreenChanged(ScreenKind),
    ChunkSelected(usize),
    ChunkAdded,
    RailToggled,
    PlayToggled,
    Settings(settings::Message),
    Edit(edit::Message),
    Tab(tab::Message),
    Sequence(sequence::Message),
}

pub struct Session {
    pub instrument: Instrument,
    pub tonic: SpelledPitch,
    pub quality: Quality,
}

impl Session {
    pub fn key(&self) -> relative::key::Key {
        model::key_of(self.tonic, self.quality)
    }

    pub fn mode(&self) -> relative::mode::Mode {
        model::mode_of(self.quality)
    }
}

// A chunk in the document: the chunk itself plus its lineage. A variant
// names the root chunk it was derived from, so its family can be grouped
// and named. Lineage is flat by construction — a variant's parent is
// always a root, never another variant.
#[derive(Clone, PartialEq, Debug)]
pub struct ChunkEntry {
    pub chunk: Word,
    pub variant_of: Option<usize>,
}

impl ChunkEntry {
    pub fn root(chunk: Word) -> ChunkEntry {
        ChunkEntry {
            chunk,
            variant_of: None,
        }
    }
}

impl Deref for ChunkEntry {
    type Target = Word;

    fn deref(&self) -> &Word {
        &self.chunk
    }
}

impl DerefMut for ChunkEntry {
    fn deref_mut(&mut self) -> &mut Word {
        &mut self.chunk
    }
}

impl From<Word> for ChunkEntry {
    fn from(chunk: Word) -> ChunkEntry {
        ChunkEntry::root(chunk)
    }
}

pub struct Document {
    pub chunks: Vec<ChunkEntry>,
    pub selected_chunk: usize,
    pub steps: Vec<Step>,
}

impl Document {
    // The lineage slice the naming and family queries read: one optional
    // root pointer per chunk.
    pub fn lineage(&self) -> Vec<Option<usize>> {
        self.chunks.iter().map(|entry| entry.variant_of).collect()
    }

    // The root of [chunk]'s family — the chunk itself for a root. Lineage
    // is flat, so one step reaches it; the loop just keeps that honest.
    pub fn root_of(&self, mut chunk: usize) -> usize {
        while let Some(parent) = self.chunks[chunk].variant_of {
            chunk = parent;
        }
        chunk
    }

    // The chunk's family: its root first, then the root's variants in
    // creation (index) order.
    pub fn family(&self, chunk: usize) -> Vec<usize> {
        let root = self.root_of(chunk);
        let mut family = vec![root];
        family.extend((0..self.chunks.len()).filter(|&i| self.chunks[i].variant_of == Some(root)));
        family
    }
}

pub struct Step {
    pub chunk: usize,
    pub root: i32,
}

pub struct Playback {
    pub playing: bool,
    pub tempo: u16,
}

// The persistent preferences — one owner for everything the settings page
// edits, shaped so a future persistence layer is a leaf change.
pub struct Settings {
    pub theme: iced::Theme,
    pub label_mode: LabelMode,
    pub font_size: f32,
    pub solver_fret_cap: i32,
}

impl Settings {
    // A design size (sized for the BASE_FONT_SIZE base) scaled to the
    // current font size.
    pub fn scale(&self, size: f32) -> f32 {
        size * self.font_size / BASE_FONT_SIZE
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            theme: iced::Theme::KanagawaWave,
            label_mode: LabelMode::Degrees,
            font_size: BASE_FONT_SIZE,
            solver_fret_cap: 6,
        }
    }
}

struct App {
    session: Session,
    document: Document,
    playback: Playback,
    board: BoardState,
    screens: Screens,
    active: ScreenKind,
    rail_open: bool,
    settings: Settings,
}

impl App {
    fn new() -> App {
        let session = Session {
            instrument: model::instrument(),
            tonic: model::TONICS[0],
            quality: Quality::Major,
        };
        let settings = Settings::default();
        let document = Document {
            chunks: vec![
                model::initial_chunk().into(),
                model::second_chunk().into(),
                model::phrase_chunk().into(),
            ],
            // The placeholder phrase chunk — the tab screen's rhythmic demo.
            selected_chunk: 2,
            steps: vec![
                Step { chunk: 0, root: 1 },
                Step { chunk: 0, root: 4 },
                Step { chunk: 0, root: 5 },
                Step { chunk: 1, root: 1 },
                Step { chunk: 0, root: 1 },
            ],
        };
        let board = BoardState::new(
            &session.instrument,
            session.key(),
            session.mode(),
            settings.label_mode,
        );
        let mut app = App {
            session,
            document,
            playback: Playback {
                playing: false,
                tempo: 120,
            },
            board,
            screens: Screens::new(),
            rail_open: true,
            settings,
            active: ScreenKind::Edit,
        };
        app.screens
            .refresh(&app.session, &app.document, &app.settings);
        app
    }

    // The board and the screens' caches are derived from the session and
    // the settings; both are recomputed when they change.
    fn refresh_session(&mut self) {
        self.board = BoardState::new(
            &self.session.instrument,
            self.session.key(),
            self.session.mode(),
            self.settings.label_mode,
        );
        self.screens
            .refresh(&self.session, &self.document, &self.settings);
    }

    fn refresh_document(&mut self) {
        self.screens
            .refresh(&self.session, &self.document, &self.settings);
    }
}

fn theme(state: &App) -> iced::Theme {
    state.settings.theme.clone()
}

// How long the cursor must dwell on an insert point before it expands.
const HOVER_DELAY: Duration = Duration::from_millis(250);

// The tick subscription runs only while an expansion is pending — the
// subscription is re-evaluated after every message batch, so the handler
// spending the pending hover turns the timer off.
fn subscription(app: &App) -> Subscription<Message> {
    if app.active == ScreenKind::Edit && app.screens.edit.hover_pending() {
        iced::time::every(HOVER_DELAY).map(|_| Message::Edit(edit::Message::HoverTick))
    } else {
        Subscription::none()
    }
}

fn update(state: &mut App, message: Message) {
    match message {
        Message::TonicPicked(tonic) => {
            if tonic != state.session.tonic {
                state.session.tonic = tonic;
                state.refresh_session();
            }
        }
        Message::QualityPicked(quality) => {
            if quality != state.session.quality {
                state.session.quality = quality;
                state.refresh_session();
            }
        }
        Message::TempoChanged(tempo) => state.playback.tempo = tempo,
        Message::ScreenChanged(kind) => state.active = kind,
        Message::ChunkSelected(index) => {
            if index < state.document.chunks.len() && index != state.document.selected_chunk {
                state.document.selected_chunk = index;
                state.refresh_document();
            }
        }
        Message::ChunkAdded => {
            state
                .document
                .chunks
                .push(Word::new(vec![Slot::Rest]).into());
            state.document.selected_chunk = state.document.chunks.len() - 1;
            state.refresh_document();
        }
        Message::RailToggled => state.rail_open = !state.rail_open,
        Message::PlayToggled => state.playback.playing = !state.playback.playing,
        Message::Settings(message) => {
            if state
                .screens
                .settings
                .update(&mut state.session, &mut state.settings, message)
            {
                state.refresh_session();
            }
        }
        Message::Edit(message) => {
            if state.screens.edit.update(
                message,
                &state.session,
                &mut state.document,
                &state.settings,
            ) {
                state.refresh_document();
            }
        }
        Message::Tab(message) => {
            if state.screens.tab.update(message) {
                state.refresh_document();
            }
        }
        Message::Sequence(message) => {
            if state.screens.sequence.update(message, &state.document) {
                state.refresh_document();
            }
        }
    }
}

fn chrome(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(colors::chrome_background(theme).into()),
        ..container::Style::default()
    }
}

fn group(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(colors::group_background(theme).into()),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub(crate) fn divider(theme: &iced::Theme) -> iced::widget::rule::Style {
    iced::widget::rule::Style {
        color: colors::divider_color(theme),
        radius: Default::default(),
        fill_mode: iced::widget::rule::FillMode::Full,
        snap: true,
    }
}

pub fn outlined(selected: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |theme, _status| button::Style {
        border: if selected {
            Border {
                radius: 4.0.into(),
                width: 2.0,
                color: colors::accent(theme),
            }
        } else {
            Border::default()
        },
        text_color: if selected {
            colors::accent(theme)
        } else {
            colors::text(theme)
        },
        ..button::Style::default()
    }
}

fn view(state: &App) -> Element<'_, Message> {
    let screen = match state.active {
        ScreenKind::Edit => edit::view(
            &state.screens.edit,
            &state.session,
            &state.document,
            &state.board,
            &state.settings,
        )
        .map(Message::Edit),
        ScreenKind::Tab => tab::view(
            &state.screens.tab,
            &state.session,
            &state.document,
            &state.board,
            &state.settings,
        )
        .map(Message::Tab),
        ScreenKind::Sequence => sequence::view(
            &state.screens.sequence,
            &state.session,
            &state.document,
            &state.board,
            &state.settings,
        )
        .map(Message::Sequence),
        ScreenKind::Settings => settings::view(
            &state.screens.settings,
            &state.session,
            &state.settings,
            &state.board,
        )
        .map(Message::Settings),
    };

    let content = container(screen).width(Fill).height(Fill).padding(12);

    // The chunk rail is editing chrome; the settings page stands alone.
    let mut body = row![].height(Fill);
    if state.rail_open && state.active != ScreenKind::Settings {
        body = body
            .push(library_rail(state))
            .push(iced::widget::rule::vertical(1.0).style(divider));
    }
    body = body.push(content);

    column![
        top_bar(state),
        iced::widget::rule::horizontal(1.0).style(divider),
        body,
    ]
    .into()
}

fn top_bar(state: &App) -> Element<'_, Message> {
    let mut nav = row![].spacing(4);
    if state.active != ScreenKind::Settings {
        nav = nav
            .push(button(if state.rail_open { "«" } else { "»" }).on_press(Message::RailToggled));
    }
    for kind in [
        ScreenKind::Edit,
        ScreenKind::Tab,
        ScreenKind::Sequence,
        ScreenKind::Settings,
    ] {
        let active = state.active == kind;
        nav = nav.push(
            button(text(kind.label()))
                .on_press(Message::ScreenChanged(kind))
                .style(move |theme, _status| button::Style {
                    background: if active {
                        Some(colors::group_background(theme).into())
                    } else {
                        None
                    },
                    text_color: if active {
                        colors::accent(theme)
                    } else {
                        colors::text(theme)
                    },
                    ..button::Style::default()
                }),
        );
    }

    let config = row![
        pick_list(
            model::TONICS.to_vec(),
            Some(state.session.tonic),
            Message::TonicPicked,
        ),
        pick_list(
            vec![Quality::Major, Quality::Minor],
            Some(state.session.quality),
            Message::QualityPicked,
        ),
        slider(40..=240, state.playback.tempo, Message::TempoChanged).width(120),
        text(format!("{} bpm", state.playback.tempo)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let playback = row![
        button(if state.playback.playing { "■" } else { "▶" }).on_press(Message::PlayToggled),
        text(state.screens.sequence.step_label(&state.document)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let base = row![
        nav,
        space::horizontal(),
        container(config).padding(4).style(group),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .width(Fill);

    // Playback floats over the base layer, centered on the window — the
    // stack's overlay sizing is what makes the centering exact rather than
    // "midway between whatever is on either side".
    let centered = container(container(playback).padding(4).style(group))
        .width(Fill)
        .height(Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    container(Stack::with_children(vec![base.into(), centered.into()]).width(Fill))
        .padding(4)
        .style(chrome)
        .into()
}

fn library_rail(state: &App) -> Element<'_, Message> {
    let names = model::chunk_names(&state.document.lineage());
    let mut list = column![].spacing(4);
    for (i, _chunk) in state.document.chunks.iter().enumerate() {
        let selected = i == state.document.selected_chunk;
        list = list.push(
            button(text(names[i].clone()))
                .width(Fill)
                .on_press(Message::ChunkSelected(i))
                .style(outlined(selected)),
        );
    }

    let rail = column![
        list,
        button("+ chunk").width(Fill).on_press(Message::ChunkAdded),
    ]
    .spacing(8)
    .padding(4);

    container(rail).width(160).height(Fill).style(chrome).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switching_screens_keeps_the_document_intact() {
        let mut app = App::new();
        let chunk_count = app.document.chunks.len();
        update(&mut app, Message::ScreenChanged(ScreenKind::Tab));
        assert_eq!(app.active, ScreenKind::Tab);
        assert_eq!(app.document.chunks.len(), chunk_count);
    }

    #[test]
    fn the_settings_page_keeps_the_document_and_the_rail_state() {
        let mut app = App::new();
        let chunk_count = app.document.chunks.len();
        update(&mut app, Message::ScreenChanged(ScreenKind::Settings));
        assert_eq!(app.active, ScreenKind::Settings);
        assert_eq!(app.document.chunks.len(), chunk_count);
        assert!(app.rail_open, "the rail state survives the settings page");
    }

    #[test]
    fn the_tempo_slider_writes_the_playback_tempo() {
        let mut app = App::new();
        update(&mut app, Message::TempoChanged(90));
        assert_eq!(app.playback.tempo, 90);
    }

    #[test]
    fn a_theme_pick_from_settings_reaches_the_app_theme() {
        let mut app = App::new();
        update(
            &mut app,
            Message::Settings(settings::Message::ThemePicked(iced::Theme::Dark)),
        );
        assert_eq!(app.settings.theme, iced::Theme::Dark);
    }

    #[test]
    fn a_label_mode_pick_from_settings_reaches_the_settings() {
        let mut app = App::new();
        update(
            &mut app,
            Message::Settings(settings::Message::LabelModePicked(LabelMode::Notes)),
        );
        assert_eq!(app.settings.label_mode, LabelMode::Notes);
    }

    #[test]
    fn the_default_font_size_reproduces_the_design_sizes() {
        let settings = Settings::default();
        assert_eq!(settings.scale(16.0), 16.0);
        assert_eq!(settings.scale(11.0), 11.0);
        let mut doubled = Settings::default();
        doubled.font_size = 32.0;
        assert_eq!(doubled.scale(16.0), 32.0);
        assert_eq!(doubled.scale(11.0), 22.0);
    }

    #[test]
    fn a_family_is_its_root_then_its_variants_in_creation_order() {
        let document = Document {
            chunks: vec![
                ChunkEntry::root(Word::new(vec![Slot::Rest])),
                ChunkEntry::root(Word::new(vec![Slot::Rest])),
                ChunkEntry {
                    chunk: Word::new(vec![Slot::Rest]),
                    variant_of: Some(0),
                },
                ChunkEntry {
                    chunk: Word::new(vec![Slot::Rest]),
                    variant_of: Some(0),
                },
            ],
            selected_chunk: 2,
            steps: Vec::new(),
        };
        assert_eq!(document.root_of(3), 0, "a variant's root is the root chunk");
        assert_eq!(document.root_of(1), 1, "a root is its own root");
        assert_eq!(
            document.family(3),
            vec![0, 2, 3],
            "the root first, then its variants in creation order"
        );
        assert_eq!(
            document.family(1),
            vec![1],
            "a variant-less root is alone in its family"
        );
    }

    #[test]
    fn a_board_click_rewrites_the_selected_slot() {
        let mut app = App::new();
        // Select the chunk explicitly — the default selection is a placeholder.
        update(&mut app, Message::ChunkSelected(0));
        let tonic_position = fretboard::fretboard_position::FretboardPosition {
            string_index: 0,
            fret: 0,
        };
        update(
            &mut app,
            Message::Edit(edit::Message::Board(board::Event::Clicked(tonic_position))),
        );
        match app.document.chunks[0][0] {
            Slot::Note(dr) => {
                assert_eq!(dr.scale_degree.degree, 3, "open E2 is degree 3 of C major");
                assert_eq!(dr.octave, 0, "stored relative to the chunk's own degree-1");
            }
            Slot::Rest => panic!("expected the slot to hold a note"),
        }
    }
}
