use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Document, Step, WordEntry};

// On-disk shape of the document file. The version lets future format
// changes reject (and quarantine) files they don't understand. Field
// order matters: TOML requires scalar keys before arrays of tables.
const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct StoredDocument {
    version: u32,
    selected_word: usize,
    words: Vec<WordEntry>,
    steps: Vec<Step>,
}

impl From<&Document> for StoredDocument {
    fn from(document: &Document) -> StoredDocument {
        StoredDocument {
            version: VERSION,
            selected_word: document.selected_word,
            words: document.words.clone(),
            steps: document.steps.clone(),
        }
    }
}

impl From<StoredDocument> for Document {
    fn from(stored: StoredDocument) -> Document {
        let mut document = Document {
            words: stored.words,
            selected_word: stored.selected_word,
            steps: stored.steps,
        };
        // A hand-edited file may point past the end; clamp instead of
        // letting a stale selection panic a screen.
        if !document.words.is_empty() {
            document.selected_word = document.selected_word.min(document.words.len() - 1);
        }
        document
    }
}

// The OS data dir: $XDG_DATA_HOME or ~/.local/share on Linux,
// ~/Library/Application Support on macOS, %APPDATA% on Windows.
pub fn default_path() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("fret_not").join("document.toml"))
}

// None means "no usable file" — missing, unreadable, wrong version —
// and the caller keeps the seed document. The file work is sync but
// small; the async wrapper keeps it off the update thread via Task.
pub async fn load() -> Option<Document> {
    load_from(&default_path()?)
}

pub async fn save(document: Document) -> io::Result<()> {
    let path =
        default_path().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no OS data dir"))?;
    save_to(&path, &document)
}

fn load_from(path: &Path) -> Option<Document> {
    let text = std::fs::read_to_string(path).ok()?;
    match toml::from_str::<StoredDocument>(&text) {
        Ok(stored) if stored.version == VERSION && !stored.words.is_empty() => {
            Some(Document::from(stored))
        }
        _ => {
            // Quarantine the unreadable file instead of deleting it.
            let _ = std::fs::rename(path, path.with_extension("bak"));
            None
        }
    }
}

fn save_to(path: &Path, document: &Document) -> io::Result<()> {
    let text = toml::to_string_pretty(&StoredDocument::from(document))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Write-then-rename so a crash mid-write can't leave a torn file.
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use absolute::alteration::Alteration;
    use relative::degree_reference::DegreeReference;
    use relative::scale_degree::ScaleDegree;
    use relative::word::{Slot, Word};

    fn note(degree: i32, alteration: Alteration, octave: i32) -> Slot {
        Slot::Note(DegreeReference::new(
            ScaleDegree::new(degree, alteration),
            octave,
        ))
    }

    fn fixture() -> Document {
        Document {
            words: vec![
                WordEntry::root(Word::new(vec![
                    note(1, Alteration::Natural, 0),
                    Slot::Rest,
                    note(3, Alteration::Flat, -1),
                ])),
                WordEntry {
                    word: Word::new(vec![]),
                    variant_of: Some(0),
                },
                WordEntry::root(Word::new(vec![note(7, Alteration::DoubleSharp, 2)])),
            ],
            selected_word: 1,
            steps: vec![Step { word: 0, root: 1 }, Step { word: 2, root: 5 }],
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "fret_not_store_{}_{}.toml",
            std::process::id(),
            name
        ))
    }

    #[test]
    fn a_document_round_trips_through_toml() {
        let path = temp_path("round_trip");
        let document = fixture();
        assert!(save_to(&path, &document).is_ok());
        assert_eq!(load_from(&path), Some(document));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn the_file_carries_the_version_and_readable_tables() {
        let path = temp_path("shape");
        assert!(save_to(&path, &fixture()).is_ok());
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("version = 1"), "the version header: {text}");
        assert!(text.contains("[[words]]"), "words as tables: {text}");
        assert!(text.contains("[[steps]]"), "steps as tables: {text}");
        assert!(
            text.contains("double_sharp"),
            "alterations as words: {text}"
        );
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn a_missing_file_loads_as_none() {
        assert_eq!(load_from(&temp_path("missing")), None);
    }

    #[test]
    fn a_corrupt_file_is_quarantined_and_loads_as_none() {
        let path = temp_path("corrupt");
        std::fs::write(&path, "this is not toml {{{").unwrap();
        assert_eq!(load_from(&path), None);
        assert!(
            path.with_extension("bak").exists(),
            "the bad file is kept as .bak"
        );
        assert!(!path.exists());
        std::fs::remove_file(path.with_extension("bak")).ok();
    }

    #[test]
    fn a_wrong_version_is_quarantined_and_loads_as_none() {
        let path = temp_path("version");
        std::fs::write(
            &path,
            "version = 99\nselected_word = 0\nwords = []\nsteps = []\n",
        )
        .unwrap();
        assert_eq!(load_from(&path), None);
        assert!(path.with_extension("bak").exists());
        std::fs::remove_file(path.with_extension("bak")).ok();
    }

    #[test]
    fn an_out_of_range_selection_is_clamped_on_load() {
        let path = temp_path("clamp");
        let mut document = fixture();
        document.selected_word = 42;
        assert!(save_to(&path, &document).is_ok());
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.selected_word, 2, "clamped to the last word");
        std::fs::remove_file(&path).ok();
    }
}
