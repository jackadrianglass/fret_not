mod board;
mod colors;
mod fretboard_canvas;
mod model;
mod screens;
mod store;
mod tab_canvas;

use std::ops::{Deref, DerefMut};
use std::time::Duration;

use iced::widget::{Stack, button, column, container, pick_list, row, slider, space, text};
use iced::{Border, Element, Fill, Subscription, Task};
use serde::{Deserialize, Serialize};

use absolute::spelled_pitch::SpelledPitch;
use fretboard::instrument::Instrument;
use relative::key::Quality;
use relative::word::{Slot, Word};

use crate::board::BoardState;
use crate::model::{BASE_FONT_SIZE, LabelMode};
use crate::screens::{ScreenKind, Screens, edit, phrase, settings, tab};

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
    WordSelected(usize),
    WordAdded,
    RailToggled,
    PlayToggled,
    Settings(settings::Message),
    Edit(edit::Message),
    Tab(tab::Message),
    Phrase(phrase::Message),
    DocumentLoaded(Option<Document>),
    DocumentSaved(Result<(), String>),
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

// A word in the document: the word itself plus its lineage. A variant
// names the root word it was derived from, so its family can be grouped
// and named. Lineage is flat by construction — a variant's parent is
// always a root, never another variant.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WordEntry {
    pub word: Word,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_of: Option<usize>,
}

impl WordEntry {
    pub fn root(word: Word) -> WordEntry {
        WordEntry {
            word,
            variant_of: None,
        }
    }
}

impl Deref for WordEntry {
    type Target = Word;

    fn deref(&self) -> &Word {
        &self.word
    }
}

impl DerefMut for WordEntry {
    fn deref_mut(&mut self) -> &mut Word {
        &mut self.word
    }
}

impl From<Word> for WordEntry {
    fn from(word: Word) -> WordEntry {
        WordEntry::root(word)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub words: Vec<WordEntry>,
    pub selected_word: usize,
    pub steps: Vec<Step>,
}

impl Document {
    // The lineage slice the naming and family queries read: one optional
    // root pointer per word.
    pub fn lineage(&self) -> Vec<Option<usize>> {
        self.words.iter().map(|entry| entry.variant_of).collect()
    }

    // The root of [word]'s family — the word itself for a root. Lineage
    // is flat, so one step reaches it; the loop just keeps that honest.
    pub fn root_of(&self, mut word: usize) -> usize {
        while let Some(parent) = self.words[word].variant_of {
            word = parent;
        }
        word
    }

    // The word's family: its root first, then the root's variants in
    // creation (index) order.
    pub fn family(&self, word: usize) -> Vec<usize> {
        let root = self.root_of(word);
        let mut family = vec![root];
        family.extend((0..self.words.len()).filter(|&i| self.words[i].variant_of == Some(root)));
        family
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub word: usize,
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
    fn new() -> (App, Task<Message>) {
        let session = Session {
            instrument: model::instrument(),
            tonic: model::TONICS[0],
            quality: Quality::Major,
        };
        let settings = Settings::default();
        let document = Document {
            words: vec![
                model::initial_word().into(),
                model::second_word().into(),
                model::phrase_word().into(),
            ],
            // The placeholder phrase word — the tab screen's rhythmic demo.
            selected_word: 2,
            steps: vec![
                Step { word: 0, root: 1 },
                Step { word: 0, root: 4 },
                Step { word: 0, root: 5 },
                Step { word: 1, root: 1 },
                Step { word: 0, root: 1 },
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
        (app, Task::perform(store::load(), Message::DocumentLoaded))
    }

    // Persist the document after a mutation; the write runs off the
    // update thread as a Task.
    fn save_document(&self) -> Task<Message> {
        Task::perform(store::save(self.document.clone()), |result| {
            Message::DocumentSaved(result.map_err(|e| e.to_string()))
        })
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

fn update(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::TonicPicked(tonic) => {
            if tonic != state.session.tonic {
                state.session.tonic = tonic;
                state.refresh_session();
            }
            Task::none()
        }
        Message::QualityPicked(quality) => {
            if quality != state.session.quality {
                state.session.quality = quality;
                state.refresh_session();
            }
            Task::none()
        }
        Message::TempoChanged(tempo) => {
            state.playback.tempo = tempo;
            Task::none()
        }
        Message::ScreenChanged(kind) => {
            state.active = kind;
            Task::none()
        }
        Message::WordSelected(index) => {
            if index < state.document.words.len() && index != state.document.selected_word {
                state.document.selected_word = index;
                state.refresh_document();
                return state.save_document();
            }
            Task::none()
        }
        Message::WordAdded => {
            state
                .document
                .words
                .push(Word::new(vec![Slot::Rest]).into());
            state.document.selected_word = state.document.words.len() - 1;
            state.refresh_document();
            state.save_document()
        }
        Message::RailToggled => {
            state.rail_open = !state.rail_open;
            Task::none()
        }
        Message::PlayToggled => {
            state.playback.playing = !state.playback.playing;
            Task::none()
        }
        Message::Settings(message) => {
            if state
                .screens
                .settings
                .update(&mut state.session, &mut state.settings, message)
            {
                state.refresh_session();
            }
            Task::none()
        }
        Message::Edit(message) => {
            if state.screens.edit.update(
                message,
                &state.session,
                &mut state.document,
                &state.settings,
            ) {
                state.refresh_document();
                return state.save_document();
            }
            Task::none()
        }
        Message::Tab(message) => {
            state.screens.tab.update(message);
            Task::none()
        }
        Message::Phrase(message) => {
            if state.screens.phrase.update(message, &state.document) {
                state.refresh_document();
                return state.save_document();
            }
            Task::none()
        }
        Message::DocumentLoaded(document) => {
            if let Some(document) = document {
                state.document = document;
                state.refresh_document();
            }
            Task::none()
        }
        Message::DocumentSaved(result) => {
            // Nothing surfaces save failures yet; the payload stays so a
            // status line can read it later.
            let _ = result;
            Task::none()
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
        ScreenKind::Phrase => phrase::view(
            &state.screens.phrase,
            &state.session,
            &state.document,
            &state.board,
            &state.settings,
        )
        .map(Message::Phrase),
        ScreenKind::Settings => settings::view(
            &state.screens.settings,
            &state.session,
            &state.settings,
            &state.board,
        )
        .map(Message::Settings),
    };

    let content = container(screen).width(Fill).height(Fill).padding(12);

    // The word rail is editing chrome; the settings page stands alone.
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
        ScreenKind::Phrase,
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
        text(state.screens.phrase.step_label(&state.document)),
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
    let names = model::word_names(&state.document.lineage());
    let mut list = column![].spacing(4);
    for (i, _word) in state.document.words.iter().enumerate() {
        let selected = i == state.document.selected_word;
        list = list.push(
            button(text(names[i].clone()))
                .width(Fill)
                .on_press(Message::WordSelected(i))
                .style(outlined(selected)),
        );
    }

    let rail = column![
        list,
        button("+ word").width(Fill).on_press(Message::WordAdded),
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
        let (mut app, _boot) = App::new();
        let word_count = app.document.words.len();
        let _ = update(&mut app, Message::ScreenChanged(ScreenKind::Tab));
        assert_eq!(app.active, ScreenKind::Tab);
        assert_eq!(app.document.words.len(), word_count);
    }

    #[test]
    fn the_settings_page_keeps_the_document_and_the_rail_state() {
        let (mut app, _boot) = App::new();
        let word_count = app.document.words.len();
        let _ = update(&mut app, Message::ScreenChanged(ScreenKind::Settings));
        assert_eq!(app.active, ScreenKind::Settings);
        assert_eq!(app.document.words.len(), word_count);
        assert!(app.rail_open, "the rail state survives the settings page");
    }

    #[test]
    fn the_tempo_slider_writes_the_playback_tempo() {
        let (mut app, _boot) = App::new();
        let _ = update(&mut app, Message::TempoChanged(90));
        assert_eq!(app.playback.tempo, 90);
    }

    #[test]
    fn a_theme_pick_from_settings_reaches_the_app_theme() {
        let (mut app, _boot) = App::new();
        let _ = update(
            &mut app,
            Message::Settings(settings::Message::ThemePicked(iced::Theme::Dark)),
        );
        assert_eq!(app.settings.theme, iced::Theme::Dark);
    }

    #[test]
    fn a_label_mode_pick_from_settings_reaches_the_settings() {
        let (mut app, _boot) = App::new();
        let _ = update(
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
            words: vec![
                WordEntry::root(Word::new(vec![Slot::Rest])),
                WordEntry::root(Word::new(vec![Slot::Rest])),
                WordEntry {
                    word: Word::new(vec![Slot::Rest]),
                    variant_of: Some(0),
                },
                WordEntry {
                    word: Word::new(vec![Slot::Rest]),
                    variant_of: Some(0),
                },
            ],
            selected_word: 2,
            steps: Vec::new(),
        };
        assert_eq!(document.root_of(3), 0, "a variant's root is the root word");
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
        let (mut app, _boot) = App::new();
        // Select the word explicitly — the default selection is a placeholder.
        let _ = update(&mut app, Message::WordSelected(0));
        let tonic_position = fretboard::fretboard_position::FretboardPosition {
            string_index: 0,
            fret: 0,
        };
        let _ = update(
            &mut app,
            Message::Edit(edit::Message::Board(board::Event::Clicked(tonic_position))),
        );
        match app.document.words[0][0] {
            Slot::Note(dr) => {
                assert_eq!(dr.scale_degree.degree, 3, "open E2 is degree 3 of C major");
                assert_eq!(dr.octave, 0, "stored relative to the word's own degree-1");
            }
            Slot::Rest => panic!("expected the slot to hold a note"),
        }
    }
}
