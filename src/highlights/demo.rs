use crate::{bookmark::Bookmarks, killstreak::Killstreaks, player::Player};
use log::error;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tf_demo_parser::{demo::header::Header, MatchState};
use tf_demo_parser::{Demo as parser_demo, DemoParser};

#[derive(Default, Clone, Debug)]
pub struct Demo {
    dir: PathBuf,
    absolute_path: PathBuf,
    relative_path: PathBuf,
    player: Player,
    events_file: PathBuf,
    date: String,
    state: Arc<MatchState>,
    map: String,
}

#[derive(Default, Debug, Clone)]
pub struct Highlights {
    killstreaks: Killstreaks,
    bookmarks: Bookmarks,
}

impl<'a> Demo {
    pub fn new(path: &'a PathBuf) -> Self {
        let mut demo: Demo = Self::default();
        let (header, state) = match parse_demo(&path) {
            Ok(val) => val,
            Err(e) => {
                error!("Failed to read parse content: {}", e);
                return Self::default();
            }
        };

        let mut dir = PathBuf::from("/");
        for part in path.to_str().unwrap().split("/") {
            if part == "demos" {
                dir.push(&part);
                break;
            }
            dir.push(&part);
        }
        demo.dir = dir;
        let creation_date = get_creation_date(&path).unwrap();
        demo.date = creation_date.to_string();
        demo.events_file = demo.dir.join("_events.txt");
        demo.player = Player::new(&state, header.nick);
        demo.map = header.map;
        demo.state = Arc::new(state);
        demo.absolute_path = path.to_path_buf();

        let mut rel_path = PathBuf::new();
        let mut dem: bool = false;
        for part in path.to_str().unwrap().split("/") {
            if part == "demos" || dem {
                dem = true;
                rel_path.push(&part);
            }
        }
        demo.relative_path = rel_path.clone();

        return demo;
    }

    pub fn collect_highlights(&self) -> Result<Highlights, Box<dyn std::error::Error>> {
        let bookmarks: Bookmarks = Bookmarks::new(&self.absolute_path)?;
        let killstreaks: Killstreaks = Killstreaks::new(&self.player, &self.state);
        return Ok(Highlights {
            killstreaks,
            bookmarks,
        });
    }

    pub fn write_highlights(self, highlights: &Highlights) {
        if highlights.bookmarks.bookmarks.is_empty()
            && highlights.killstreaks.killstreaks.is_empty()
        {
            return;
        }

        let mut file = match OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&self.events_file)
        {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open events file: {}", e);
                return;
            }
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to read existing content: {}", e);
                return;
            }
        };

        let mut lines: Vec<&str> = contents.split('\n').collect();

        let parts: Vec<&str> = self
            .absolute_path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split('-')
            .collect();

        // remove map name
        let demo_name = parts[..parts.len() - 1].join("-");

        // remove whatever ds_log put in _events.txt
        lines.retain(|line| !line.contains(&demo_name));

        file = match OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(&self.events_file)
        {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open events file: {}", e);
                return;
            }
        };

        match file.write_all(lines.join("\n").as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to write separator: {}", e);
                return;
            }
        };

        for line in lines {
            if line.is_empty() {
                let header = format!("[{}] {} {}", self.date, self.map, self.player.class);
                let header = format!(
                    "{}{}playdemo {}\n",
                    header,
                    " ".repeat(65 - header.len()),
                    self.relative_path.display(),
                );

                match file.write_all(header.as_bytes()) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Failed to write header: {}", e);
                        return;
                    }
                };

                for killstreak in &highlights.killstreaks.killstreaks {
                    let killstreak_str = format!(
                        "[{}] Killstreak {} {}-{} [{:.2} seconds]\n",
                        self.date,
                        killstreak.kills.kills.len(),
                        killstreak.start_tick - 500,
                        killstreak.end_tick,
                        killstreak.length
                    );
                    match file.write_all(killstreak_str.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Failed to write killstreak: {}", e);
                            return;
                        }
                    };
                }

                for bookmark in &highlights.bookmarks.bookmarks {
                    let bookmark_str =
                        format!("[{}] Bookmark at {}\n", self.date, bookmark.tick - 500);
                    match file.write_all(bookmark_str.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Failed to write bookmark: {}", e);
                            return;
                        }
                    };
                }
                match file.write_all(">\n".as_bytes()) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Failed to write separator: {}", e);
                        return;
                    }
                };
            }
        }
    }
}

pub fn parse_demo(path: &PathBuf) -> Result<(Header, MatchState), Box<dyn std::error::Error>> {
    let mut dem_path = path.clone();
    dem_path.set_extension("dem");
    let file = fs::read(dem_path)?;
    let demo_file = parser_demo::new(&file);
    let parser = DemoParser::new(demo_file.get_stream());
    let (header, state) = parser.parse()?;
    return Ok((header, state));
}

pub fn get_highlights(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let demo_path = PathBuf::from(&path.to_str().unwrap().trim_end_matches(".dem"));
    let demo = Demo::new(&demo_path);
    let highlights = &demo.collect_highlights()?;
    demo.write_highlights(highlights);
    Ok(())
}

fn get_creation_date(file_path: &PathBuf) -> Option<String> {
    let metadata = fs::metadata(file_path).ok()?;
    let creation_time = metadata.created().ok()?;
    let unix_epoch = std::time::UNIX_EPOCH;
    let duration = creation_time.duration_since(unix_epoch).ok()?;
    let seconds = duration.as_secs();
    let days = seconds / 86400; // 86400 seconds in a day
    let years = days / 365; // 365 days in a year (ignoring leap years)
    let months = (days % 365) / 30; // 30 days in a month (ignoring varying month lengths)
    let days = days % 30;
    Some(format!(
        "{:04}-{:02}-{:02}",
        years + 1970,
        months + 1,
        days + 1
    ))
}

#[test]
pub fn test_get_highlights() {
    let path = PathBuf::from("/home/deity/.steam/steam/steamapps/common/Team Fortress 2/tf/demos/2024/2024-08/2024-08-23_22-40-25-cp_process_f12.dem");
    let demo = Demo::new(&path);
    let highlights = &demo.collect_highlights().unwrap();
    demo.write_highlights(highlights);
}
