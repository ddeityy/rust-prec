use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::{self};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    name: String,
    value: String,
    tick: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Root {
    events: Vec<Event>,
}

#[derive(Default, Debug, Clone)]
pub struct Bookmark {
    pub tick: i64,
}

#[derive(Default, Debug, Clone)]
pub struct Bookmarks {
    pub bookmarks: Vec<Bookmark>,
}

impl Bookmarks {
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut bookmarks = Self::default();
        bookmarks.get_bookmarks(path)?;
        Ok(bookmarks)
    }

    pub fn get_bookmarks(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut json_path = path.to_path_buf();
        json_path.set_extension("json");
        let mut file = File::open(&json_path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let root: Value = serde_json::from_str(&contents)?;

        if let Some(events) = root.get("events") {
            let bookmark_ticks: Vec<Bookmark> = events
                .as_array()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Expected events to be an array",
                    )
                })?
                .iter()
                .filter_map(|event| {
                    if event["name"].as_str()?.starts_with("Bookmark") {
                        Some(Bookmark {
                            tick: event["tick"].as_i64()?,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            if !bookmark_ticks.is_empty() {
                for bookmark in bookmark_ticks {
                    self.bookmarks.push(bookmark);
                }
            }
        } else {
            return Err("JSON does not contain 'events' key.".into());
        }

        Ok(())
    }
}
