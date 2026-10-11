pub mod edit;
pub mod sequence;
pub mod tab;

use crate::{Document, Session};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenKind {
    Edit,
    Tab,
    Sequence,
}

impl ScreenKind {
    pub fn label(&self) -> &'static str {
        match self {
            ScreenKind::Edit => "edit",
            ScreenKind::Tab => "tab",
            ScreenKind::Sequence => "sequence",
        }
    }
}

pub struct Screens {
    pub edit: edit::EditScreen,
    pub tab: tab::TabScreen,
    pub sequence: sequence::SequenceScreen,
}

impl Screens {
    pub fn new() -> Screens {
        Screens {
            edit: edit::EditScreen::new(),
            tab: tab::TabScreen::new(),
            sequence: sequence::SequenceScreen::new(),
        }
    }

    // Fingerings are cached in update; any document or session change makes
    // them stale, so the shell calls this after every such mutation.
    pub fn refresh(&mut self, session: &Session, document: &Document) {
        self.edit.refresh(session, document);
        self.tab.refresh(session, document);
        self.sequence.refresh(session, document);
    }
}
