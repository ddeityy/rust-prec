use crate::{bookmark::Bookmarks, killstreak::Killstreaks, player::Player};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use tf_demo_parser::{demo::header::Header, MatchState};
use tf_demo_parser::{Demo as parser_demo, DemoParser};

#[derive(Default)]
pub struct Demo<'a> {
    dir: PathBuf,
    name: &'a str,
    absolute_path: PathBuf,
    relative_path: PathBuf,
    player: Player,
    events_file: PathBuf,
    date: &'a str,
    state: MatchState,
    map: String,
}

#[derive(Default, Debug)]
pub struct Highlights {
    killstreaks: Killstreaks,
    bookmarks: Bookmarks,
}

impl<'a> Demo<'a> {
    pub fn new(path: &'a PathBuf) -> Self {
        let mut demo: Demo = Self::default();
        let (header, state) = parse_demo(&path);

        demo.name = path.file_stem().unwrap().to_str().unwrap();
        demo.date = demo.name.split("_").collect::<Vec<&str>>()[0];
        demo.dir = path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        demo.events_file = demo.dir.join("_events.txt");
        demo.player = Player::new(&state, header.nick);
        demo.map = header.map;
        demo.state = state;
        demo.absolute_path = path.to_path_buf();
        let binding = demo
            .absolute_path
            .to_str()
            .unwrap()
            .split("/")
            .collect::<Vec<_>>();
        let binding = binding.iter().rev().take(3).collect::<Vec<&&str>>();
        let path_vec: Vec<&&&str> = binding.iter().rev().collect();
        for path in path_vec {
            demo.relative_path = demo.relative_path.join(path);
        }
        demo.relative_path = PathBuf::from(
            demo.relative_path
                .to_str()
                .unwrap()
                .split(".")
                .collect::<Vec<&str>>()[0],
        );

        return demo;
    }

    pub fn collect_highlights(&self) -> Highlights {
        let bookmarks: Bookmarks = Bookmarks::new(&self.absolute_path);
        let killstreaks: Killstreaks = Killstreaks::new(&self.player, &self.state);
        return Highlights {
            killstreaks,
            bookmarks,
        };
    }

    pub fn write_highlights(self, highlights: &Highlights) {
        if highlights.bookmarks.bookmarks.len() == 0
            && highlights.killstreaks.killstreaks.len() == 0
        {
            return;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(self.events_file)
            .unwrap();
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents).unwrap();

        let lines: Vec<&str> = contents.split('\n').collect();
        for line in lines {
            if line.trim().is_empty() {
                let header = format!("[{}] {} {}", self.date, self.map, self.player.class);
                let header = format!(
                    "{}{}playdemo demos/{}\n",
                    header,
                    " ".repeat(65 - header.len()),
                    self.relative_path.display(),
                );

                file.write_all(header.as_bytes()).unwrap();

                for killstreak in &highlights.killstreaks.killstreaks {
                    let killstreak_str = format!(
                        "[{}] Killstreak {} {}-{} [{} seconds]\n",
                        self.date,
                        killstreak.kills.kills.len(),
                        killstreak.start_tick - 500,
                        killstreak.end_tick,
                        killstreak.length
                    );
                    file.write_all(killstreak_str.as_bytes()).unwrap();
                }

                for bookmark in &highlights.bookmarks.bookmarks {
                    let bookmark_str =
                        format!("[{}] Bookmark at {}\n", self.date, bookmark.tick - 500);
                    file.write_all(bookmark_str.as_bytes()).unwrap();
                }
                file.write_all(">\n".as_bytes()).unwrap();
            }
        }
    }
}

pub fn parse_demo(path: &PathBuf) -> (Header, MatchState) {
    let mut dem_path = path.clone();
    dem_path.set_extension("dem");
    let file = fs::read(dem_path).unwrap();
    let demo_file = parser_demo::new(&file);
    let parser = DemoParser::new(demo_file.get_stream());
    let (header, state) = parser.parse().unwrap();
    return (header, state);
}

pub fn get_highlights(path: &PathBuf) {
    let demo = Demo::new(&path);
    let highlights = &demo.collect_highlights();
    demo.write_highlights(highlights);
}

// #[test]
// pub fn get_highlights() {
//     let path = PathBuf::from("/home/deity/.steam/steam/steamapps/common/Team Fortress 2/tf/demos/2023/2023-09/2023-09-21_21-13-15.dem");
//     let demo = Demo::new(&path);
//     let highlights = &demo.collect_highlights();
//     demo.write_highlights(highlights);
// }
