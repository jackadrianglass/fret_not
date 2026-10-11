mod board;
mod colors;
mod fretboard_canvas;
mod model;
mod screens;

use iced::widget::{Stack, button, column, container, pick_list, row, space, text, toggler};
use iced::{Border, Element, Fill};

use absolute::spelled_pitch::SpelledPitch;
use fretboard::instrument::Instrument;
use relative::chunk::{Chunk, Slot};
use relative::key::Quality;

use crate::board::BoardState;
use crate::model::LabelMode;
use crate::screens::{ScreenKind, Screens, edit, sequence, tab};

fn main() -> iced::Result {
    iced::application(App::new, update, view)
        .title("fret_not")
        .theme(theme)
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
    NotesToggled(bool),
    ScreenChanged(ScreenKind),
    ChunkSelected(usize),
    ChunkAdded,
    RailToggled,
    PlayToggled,
    ThemePicked(iced::Theme),
    Edit(edit::Message),
    Tab(tab::Message),
    Sequence(sequence::Message),
}

pub struct Session {
    pub instrument: Instrument,
    pub tonic: SpelledPitch,
    pub quality: Quality,
    pub label_mode: LabelMode,
}

impl Session {
    pub fn key(&self) -> relative::key::Key {
        model::key_of(self.tonic, self.quality)
    }

    pub fn mode(&self) -> relative::mode::Mode {
        model::mode_of(self.quality)
    }
}

pub struct Document {
    pub chunks: Vec<Chunk>,
    pub selected_chunk: usize,
    pub steps: Vec<Step>,
}

pub struct Step {
    pub chunk: usize,
    pub root: i32,
}

pub struct Playback {
    pub playing: bool,
    pub tempo: u16,
}

struct App {
    session: Session,
    document: Document,
    playback: Playback,
    board: BoardState,
    screens: Screens,
    active: ScreenKind,
    rail_open: bool,
    theme: iced::Theme,
}

impl App {
    fn new() -> App {
        let session = Session {
            instrument: model::instrument(),
            tonic: model::TONICS[0],
            quality: Quality::Major,
            label_mode: LabelMode::Degrees,
        };
        let document = Document {
            chunks: vec![model::initial_chunk(), model::second_chunk()],
            selected_chunk: 0,
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
            session.label_mode,
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
            theme: iced::Theme::KanagawaWave,
            active: ScreenKind::Edit,
        };
        app.screens.refresh(&app.session, &app.document);
        app
    }

    // The board and the screens' caches are derived from the session; both
    // are recomputed when it changes.
    fn refresh_session(&mut self) {
        self.board = BoardState::new(
            &self.session.instrument,
            self.session.key(),
            self.session.mode(),
            self.session.label_mode,
        );
        self.screens.refresh(&self.session, &self.document);
    }

    fn refresh_document(&mut self) {
        self.screens.refresh(&self.session, &self.document);
    }
}

fn theme(state: &App) -> iced::Theme {
    state.theme.clone()
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
        Message::NotesToggled(enabled) => {
            let label_mode = if enabled {
                LabelMode::Notes
            } else {
                LabelMode::Degrees
            };
            if label_mode != state.session.label_mode {
                state.session.label_mode = label_mode;
                state.refresh_session();
            }
        }
        Message::ScreenChanged(kind) => state.active = kind,
        Message::ChunkSelected(index) => {
            if index < state.document.chunks.len() && index != state.document.selected_chunk {
                state.document.selected_chunk = index;
                state.refresh_document();
            }
        }
        Message::ChunkAdded => {
            state.document.chunks.push(Chunk::new(vec![Slot::Rest]));
            state.document.selected_chunk = state.document.chunks.len() - 1;
            state.refresh_document();
        }
        Message::RailToggled => state.rail_open = !state.rail_open,
        Message::PlayToggled => state.playback.playing = !state.playback.playing,
        // Every derived color reads the theme at render time, so switching
        // needs no refresh of the board or the screens' caches.
        Message::ThemePicked(theme) => state.theme = theme,
        Message::Edit(message) => {
            if state
                .screens
                .edit
                .update(message, &state.session, &mut state.document)
            {
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

fn divider(theme: &iced::Theme) -> iced::widget::rule::Style {
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
        )
        .map(Message::Edit),
        ScreenKind::Tab => tab::view(
            &state.screens.tab,
            &state.session,
            &state.document,
            &state.board,
        )
        .map(Message::Tab),
        ScreenKind::Sequence => sequence::view(
            &state.screens.sequence,
            &state.session,
            &state.document,
            &state.board,
        )
        .map(Message::Sequence),
    };

    let content = container(screen).width(Fill).height(Fill).padding(12);

    let mut body = row![].height(Fill);
    if state.rail_open {
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
    nav = nav.push(button(if state.rail_open { "«" } else { "»" }).on_press(Message::RailToggled));
    for kind in [ScreenKind::Edit, ScreenKind::Tab, ScreenKind::Sequence] {
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
        toggler(matches!(state.session.label_mode, LabelMode::Notes))
            .label("note names")
            .on_toggle(Message::NotesToggled),
        text(format!("{} bpm", state.playback.tempo)),
        pick_list(
            iced::Theme::ALL.to_vec(),
            Some(state.theme.clone()),
            Message::ThemePicked,
        ),
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
    let mut list = column![].spacing(4);
    for (i, _chunk) in state.document.chunks.iter().enumerate() {
        let selected = i == state.document.selected_chunk;
        list = list.push(
            button(text(model::chunk_name(i)))
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
        update(&mut app, Message::ScreenChanged(ScreenKind::Tab));
        assert_eq!(app.active, ScreenKind::Tab);
        assert_eq!(app.document.chunks.len(), 2);
    }

    #[test]
    fn a_board_click_rewrites_the_selected_slot() {
        let mut app = App::new();
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
                assert_eq!(dr.scale_degree.degree, 3, "open E2 is degree 3 of C major")
            }
            Slot::Rest => panic!("expected the slot to hold a note"),
        }
    }
}
