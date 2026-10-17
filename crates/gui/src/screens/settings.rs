use iced::widget::{column, container, pick_list, row, rule, scrollable, text, text_input};
use iced::{Alignment, Element, Fill};

use fretboard::fingering::Fingering;

use crate::board::{self, BoardState, Highlight};
use crate::colors;
use crate::model::{InstrumentPreset, LabelMode};
use crate::{Session, Settings};

const FONT_SIZE_MIN: f32 = 8.0;
const FONT_SIZE_MAX: f32 = 32.0;
const FRET_CAP_MIN: i32 = 1;
const FRET_CAP_MAX: i32 = 12;

// The settings page: named sections divided by rules, the shared board
// docked below as a live preview — it re-labels on a Labels change and
// re-sizes on a Font size change.
pub struct SettingsScreen {
    // The number inputs' backing text. iced 0.14 has no number input, so a
    // text input commits on every parseable keystroke; unparseable
    // intermediate text commits nothing.
    font_text: String,
    cap_text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Board(board::Event),
    ThemePicked(iced::Theme),
    LabelModePicked(LabelMode),
    FontSizeInput(String),
    TuningPicked(InstrumentPreset),
    FretCapInput(String),
}

impl SettingsScreen {
    pub fn new() -> SettingsScreen {
        SettingsScreen {
            font_text: String::new(),
            cap_text: String::new(),
        }
    }

    // Re-syncs a text field when what it parses to (clamped) diverges from
    // the committed value — so a clamped commit never fights the typing.
    pub fn refresh(&mut self, settings: &Settings) {
        if self
            .font_text
            .parse::<f32>()
            .ok()
            .map(|size| size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX))
            != Some(settings.font_size)
        {
            self.font_text = settings.font_size.to_string();
        }
        if self
            .cap_text
            .parse::<i32>()
            .ok()
            .map(|cap| cap.clamp(FRET_CAP_MIN, FRET_CAP_MAX))
            != Some(settings.solver_fret_cap)
        {
            self.cap_text = settings.solver_fret_cap.to_string();
        }
    }

    // True when session-derived state (the board, the solver caches) must be
    // recomputed — the caller then runs refresh_session.
    pub fn update(
        &mut self,
        session: &mut Session,
        settings: &mut Settings,
        message: Message,
    ) -> bool {
        match message {
            Message::Board(board::Event::Clicked(_)) => false,
            Message::ThemePicked(theme) => {
                // Every derived color reads the theme at render time, so
                // switching needs no refresh.
                settings.theme = theme;
                false
            }
            Message::LabelModePicked(label_mode) => {
                if label_mode != settings.label_mode {
                    settings.label_mode = label_mode;
                    return true;
                }
                false
            }
            Message::FontSizeInput(text) => {
                self.font_text = text;
                if let Ok(size) = self.font_text.parse::<f32>() {
                    settings.font_size = size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX);
                }
                false
            }
            Message::TuningPicked(preset) => {
                let instrument = preset.instrument();
                if instrument != session.instrument {
                    session.instrument = instrument;
                    return true;
                }
                false
            }
            Message::FretCapInput(text) => {
                self.cap_text = text;
                if let Ok(cap) = self.cap_text.parse::<i32>() {
                    let cap = cap.clamp(FRET_CAP_MIN, FRET_CAP_MAX);
                    if cap != settings.solver_fret_cap {
                        settings.solver_fret_cap = cap;
                        return true;
                    }
                }
                false
            }
        }
    }
}

fn accent(theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(colors::accent(theme)),
    }
}

fn secondary(theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(colors::secondary_text(theme)),
    }
}

fn section<'a>(
    title: &'a str,
    rows: Vec<Element<'a, Message>>,
    settings: &'a Settings,
) -> Element<'a, Message> {
    column![
        text(title).size(settings.scale(18.0)).style(accent),
        column(rows).spacing(8),
        rule::horizontal(1.0).style(crate::divider),
    ]
    .spacing(12)
    .into()
}

fn setting_row<'a>(
    label: &'a str,
    control: Element<'a, Message>,
    settings: &'a Settings,
) -> Element<'a, Message> {
    row![text(label).size(settings.scale(16.0)).width(160), control]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}

fn number_row<'a>(
    value: &'a str,
    placeholder: &'static str,
    suffix: &'static str,
    on_input: impl Fn(String) -> Message + 'a,
    settings: &'a Settings,
) -> Element<'a, Message> {
    row![
        text_input(placeholder, value).on_input(on_input).width(80),
        text(suffix).size(settings.scale(12.0)).style(secondary),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

pub fn view<'a>(
    screen: &'a SettingsScreen,
    session: &'a Session,
    settings: &'a Settings,
    board: &'a BoardState,
) -> Element<'a, Message> {
    let appearance = section(
        "Appearance",
        vec![
            setting_row(
                "Theme",
                pick_list(
                    iced::Theme::ALL.to_vec(),
                    Some(settings.theme.clone()),
                    Message::ThemePicked,
                )
                .into(),
                settings,
            ),
            setting_row(
                "Labels",
                pick_list(
                    vec![LabelMode::Degrees, LabelMode::Notes],
                    Some(settings.label_mode),
                    Message::LabelModePicked,
                )
                .into(),
                settings,
            ),
            setting_row(
                "Font size",
                number_row(
                    &screen.font_text,
                    "16.0",
                    "px",
                    Message::FontSizeInput,
                    settings,
                ),
                settings,
            ),
        ],
        settings,
    );

    let fretboard = section(
        "Fretboard",
        vec![
            setting_row(
                "Tuning",
                pick_list(
                    InstrumentPreset::ALL.to_vec(),
                    Some(InstrumentPreset::of(&session.instrument)),
                    Message::TuningPicked,
                )
                .into(),
                settings,
            ),
            setting_row(
                "Fret cap",
                number_row(
                    &screen.cap_text,
                    "6",
                    "frets",
                    Message::FretCapInput,
                    settings,
                ),
                settings,
            ),
        ],
        settings,
    );

    let sections = column![
        text("Settings").size(settings.scale(24.0)),
        appearance,
        fretboard,
    ]
    .spacing(16)
    .width(640.0);

    let page = container(scrollable(
        container(sections)
            .width(Fill)
            .align_x(iced::alignment::Horizontal::Center),
    ))
    .width(Fill)
    .height(Fill)
    .padding(24);

    let board_area = container(
        board::view(
            board,
            Highlight {
                positions: Fingering::default(),
                selected: None,
            },
            settings,
        )
        .map(Message::Board),
    )
    .height(260);

    column![page, board_area]
        .spacing(12)
        .width(Fill)
        .height(Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;
    use relative::key::Quality;

    fn session() -> Session {
        Session {
            instrument: model::instrument(),
            tonic: model::TONICS[0],
            quality: Quality::Major,
        }
    }

    #[test]
    fn font_size_input_commits_valid_parses_clamped() {
        let mut screen = SettingsScreen::new();
        let mut session = session();
        let mut settings = Settings::default();

        assert!(
            !screen.update(
                &mut session,
                &mut settings,
                Message::FontSizeInput("abc".into())
            ),
            "a font change is read at render time, nothing to refresh"
        );
        assert_eq!(settings.font_size, 16.0, "unparseable text commits nothing");

        screen.update(
            &mut session,
            &mut settings,
            Message::FontSizeInput("20".into()),
        );
        assert_eq!(settings.font_size, 20.0);

        screen.update(
            &mut session,
            &mut settings,
            Message::FontSizeInput("100".into()),
        );
        assert_eq!(settings.font_size, 32.0, "clamped to the maximum");
    }

    #[test]
    fn fret_cap_input_commits_and_stales_the_solver_caches() {
        let mut screen = SettingsScreen::new();
        let mut session = session();
        let mut settings = Settings::default();

        assert!(screen.update(
            &mut session,
            &mut settings,
            Message::FretCapInput("9".into())
        ));
        assert_eq!(settings.solver_fret_cap, 9);

        assert!(
            !screen.update(
                &mut session,
                &mut settings,
                Message::FretCapInput("abc".into())
            ),
            "unparseable text commits nothing and refreshes nothing"
        );
        assert_eq!(settings.solver_fret_cap, 9);

        screen.update(
            &mut session,
            &mut settings,
            Message::FretCapInput("99".into()),
        );
        assert_eq!(settings.solver_fret_cap, 12, "clamped to the maximum");
    }

    #[test]
    fn refresh_resyncs_text_that_diverges_from_the_committed_value() {
        let mut screen = SettingsScreen::new();
        let settings = Settings::default();
        screen.refresh(&settings);
        assert_eq!(screen.font_text, "16");
        assert_eq!(screen.cap_text, "6");
    }

    #[test]
    fn refresh_keeps_text_that_still_parses_to_the_committed_value() {
        let mut screen = SettingsScreen::new();
        let mut session = session();
        let mut settings = Settings::default();

        screen.update(
            &mut session,
            &mut settings,
            Message::FontSizeInput("100".into()),
        );
        screen.refresh(&settings);
        assert_eq!(
            screen.font_text, "100",
            "the clamped commit (32.0) matches the clamped parse, the text stays as typed"
        );
    }

    #[test]
    fn a_tuning_pick_rewrites_the_session_instrument() {
        let mut screen = SettingsScreen::new();
        let mut session = session();
        let mut settings = Settings::default();

        assert!(screen.update(
            &mut session,
            &mut settings,
            Message::TuningPicked(model::InstrumentPreset::DropD)
        ));
        assert_eq!(
            session.instrument,
            model::InstrumentPreset::DropD.instrument()
        );

        assert!(
            !screen.update(
                &mut session,
                &mut settings,
                Message::TuningPicked(model::InstrumentPreset::DropD)
            ),
            "re-picking the current preset changes nothing"
        );
    }

    #[test]
    fn a_label_mode_pick_stales_the_board() {
        let mut screen = SettingsScreen::new();
        let mut session = session();
        let mut settings = Settings::default();

        assert!(screen.update(
            &mut session,
            &mut settings,
            Message::LabelModePicked(LabelMode::Notes)
        ));
        assert_eq!(settings.label_mode, LabelMode::Notes);

        assert!(
            !screen.update(
                &mut session,
                &mut settings,
                Message::LabelModePicked(LabelMode::Notes)
            ),
            "re-picking the current mode changes nothing"
        );
    }

    #[test]
    fn a_board_click_on_the_preview_is_ignored() {
        let mut screen = SettingsScreen::new();
        let mut session = session();
        let mut settings = Settings::default();
        assert!(!screen.update(
            &mut session,
            &mut settings,
            Message::Board(board::Event::Clicked(
                fretboard::fretboard_position::FretboardPosition {
                    string_index: 0,
                    fret: 0,
                }
            ))
        ));
    }
}
