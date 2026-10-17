pub mod edit;
pub mod sequence;
pub mod settings;
pub mod tab;

use crate::{Document, Session, Settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenKind {
    Edit,
    Tab,
    Sequence,
    Settings,
}

impl ScreenKind {
    pub fn label(&self) -> &'static str {
        match self {
            ScreenKind::Edit => "edit",
            ScreenKind::Tab => "tab",
            ScreenKind::Sequence => "sequence",
            ScreenKind::Settings => "settings",
        }
    }
}

pub struct Screens {
    pub edit: edit::EditScreen,
    pub tab: tab::TabScreen,
    pub sequence: sequence::SequenceScreen,
    pub settings: settings::SettingsScreen,
}

impl Screens {
    pub fn new() -> Screens {
        Screens {
            edit: edit::EditScreen::new(),
            tab: tab::TabScreen::new(),
            sequence: sequence::SequenceScreen::new(),
            settings: settings::SettingsScreen::new(),
        }
    }

    // Fingerings are cached in update; any document, session, or settings
    // change makes them stale, so the shell calls this after every such
    // mutation.
    pub fn refresh(&mut self, session: &Session, document: &Document, settings: &Settings) {
        self.edit.refresh(session, document, settings);
        self.tab.refresh(session, document, settings);
        self.sequence.refresh(session, document, settings);
        self.settings.refresh(settings);
    }
}
