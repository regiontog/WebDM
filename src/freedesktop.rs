use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum EntryType {
    Application,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Entry {
    pub(crate) typ: EntryType,
    pub(crate) name: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) exec: Option<String>,
    pub(crate) icon: Option<String>,
    pub(crate) comment: Option<String>,
}

#[derive(Debug)]
pub(crate) enum EntryError {
    FileError(FileError),
    MissingRequiredField,
}

impl Entry {
    pub(crate) fn parse(path: &Path) -> Result<Self, EntryError> {
        let mut desktop = DesktopFile::new(path).map_err(EntryError::FileError)?;

        Ok(Entry {
            typ: match desktop
                .0
                .remove("Type")
                .ok_or(EntryError::MissingRequiredField)?
                .as_ref()
            {
                "Application" => EntryType::Application,
                _ => EntryType::Unknown,
            },
            name: desktop
                .0
                .remove("Name")
                .ok_or(EntryError::MissingRequiredField)?,
            path: desktop.0.remove("Path").map(Into::into),
            exec: desktop.0.remove("Exec"),
            icon: desktop.0.remove("Icon"),
            comment: desktop.0.remove("Comment"),
        })
    }
}

struct DesktopFile(HashMap<String, String>);

#[derive(Debug)]
pub(crate) enum FileError {
    Io(std::io::Error),
    BadFormat,
}

impl DesktopFile {
    fn new(path: &Path) -> Result<Self, FileError> {
        let f = File::open(path).map_err(FileError::Io)?;
        let reader = BufReader::new(f);
        let mut lines = reader.lines();

        match lines.next() {
            Some(Ok(ref s)) if s == "[Desktop Entry]" => {}
            _ => println!("Possibly bad desktop entry file {}", path.display()),
        }

        let mut desktop = DesktopFile(HashMap::new());

        for line in lines {
            let line = line.map_err(FileError::Io)?;

            if line.trim_start().starts_with('#') {
                continue;
            }

            if line.trim_start().starts_with('[') {
                break;
            }

            let mut line = line.splitn(2, '=');
            let key = line.next().ok_or(FileError::BadFormat)?.trim_end();
            let value = line.next().ok_or(FileError::BadFormat)?.trim_start();
            desktop.0.insert(key.to_owned(), value.to_owned());
        }

        Ok(desktop)
    }
}
