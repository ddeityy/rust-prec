mod error;
mod throttle;
mod watcher;

use crate::watcher::LogWatcher;
use log::{error, info, trace};
use rcon::Connection;
use std::env::var;
use std::fs::{copy, create_dir_all, remove_file, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use steamlocate::SteamDir;
use throttle::Throttler;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::time::{sleep, Duration};

use bitbuffer::{BitRead, BitReadStream, LittleEndian};
use error::Error;

#[derive(PartialEq, Debug)]
enum ConsoleEvent {
    Record,
    Stop,
}

#[derive(Debug)]
struct EventHandler {
    rcon_password: String,
}

impl EventHandler {
    fn new(rcon_password: String) -> Self {
        EventHandler { rcon_password }
    }

    async fn handle(&mut self, event: ConsoleEvent) {
        let Ok(mut conn) = self.connect().await else {
            error!("Failed to connect to rcon");
            return;
        };

        info!("Sending {:?}", self);
        match conn.cmd(event.command()).await {
            Ok(response) => {
                if let Some((_, relative_path)) =
                    response.trim().split_once("(Demo Support) End recording ")
                {
                    let path = tf_path().join(relative_path);
                    info!("Demo recorded: {}", path.display());
                    spawn(async {
                        sleep(Duration::from_secs(5)).await; // give tf2 some time to finish up
                        if let Err(e) = Self::post_record(path).await {
                            error!("Error while processing recorded demo: {e:?}")
                        }
                    });
                }
            }
            Err(e) => {
                error!("Error while sending rcon event: {e:?}")
            }
        }
    }

    async fn connect(&self) -> Result<Connection<TcpStream>, rcon::Error> {
        Connection::<TcpStream>::builder()
            .connect("127.0.0.1:27015", dbg!(&self.rcon_password))
            .await
    }

    async fn post_record(path: PathBuf) -> Result<(), Error> {
        let name = path
            .file_name()
            .ok_or(Error::InvalidDemoPath)?
            .to_str()
            .ok_or(Error::InvalidDemoPath)?;
        let base_path = path.parent().ok_or(Error::InvalidDemoPath)?;
        // name format is YYYY-MM-DD_HH-MM-SS
        let mut date_parts = name.split('-');
        let year = date_parts.next().ok_or(Error::InvalidDemoPath)?;
        let month = date_parts.next().ok_or(Error::InvalidDemoPath)?;

        let demo_path = base_path.join(format!("{name}.dem"));
        let map = Header::read(&demo_path)?.map;

        let target_dir = base_path.join(format!("{year}/{year}-{month}"));
        create_dir_all(&target_dir)?;

        let files = base_path
            .read_dir()?
            .flatten()
            .filter_map(|entry| {
                let entry_name = entry.file_name();
                let entry_name = entry_name.to_str()?;
                entry_name.starts_with(name).then(|| entry.path())
            })
            .collect::<Vec<PathBuf>>();

        for file in files {
            let extension = file
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            let target = target_dir.join(format!("{name}-{map}.{extension}"));
            info!("moving {} to {}", file.display(), target.display());

            // copy + delete instead of rename to allow for cross-device move
            copy(&file, target)?;
            remove_file(file)?;
        }

        Ok(())
    }
}

impl ConsoleEvent {
    fn from_chat(chat: &str) -> Option<Self> {
        if chat.contains("[SOAP] Soap DM unloaded.") | chat.contains("[P-REC] Recording...") {
            Some(ConsoleEvent::Record)
        } else if chat.contains("[LogsTF] Uploading logs...")
            | chat.contains("[P-REC] Stop record.")
        {
            Some(ConsoleEvent::Stop)
        } else {
            None
        }
    }

    fn command(&self) -> &'static str {
        match self {
            ConsoleEvent::Record => "ds_record",
            ConsoleEvent::Stop => "ds_stop",
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    info!("P-REC started");

    let rcon_password = var("RCON_PASSWORD").unwrap_or_else(|_| "prec".to_string());

    let path = log_path();

    // make sure the file exists
    OpenOptions::new().write(true).create(true).open(&path)?;

    let log_watcher = LogWatcher::new(path);

    let mut handler = EventHandler::new(rcon_password);
    let delay = Duration::from_millis(7500);
    let mut throttler = Throttler::new(delay);

    for line_result in log_watcher {
        let line = line_result?;
        trace!("got log line: {line}");
        if let Some(event) = ConsoleEvent::from_chat(line.trim()) {
            if let Some(event) = throttler.debounce(event) {
                handler.handle(event).await;
            }
        }
    }

    Ok(())
}

fn tf_path() -> PathBuf {
    let dir = SteamDir::locate().unwrap();
    dir.path()
        .join("steamapps")
        .join("common")
        .join("Team Fortress 2")
        .join("tf")
}

fn log_path() -> PathBuf {
    tf_path().join("console.log")
}

#[derive(BitRead, Debug)]
#[allow(dead_code)]
struct Header {
    #[size = 8]
    pub demo_type: String,
    pub version: u32,
    pub protocol: u32,
    #[size = 260]
    pub server: String,
    #[size = 260]
    pub nick: String,
    #[size = 260]
    pub map: String,
    #[size = 260]
    pub game: String,
    pub duration: f32,
    pub ticks: u32,
    pub frames: u32,
    pub signon: u32,
}

impl Header {
    fn read(path: &Path) -> Result<Header, Error> {
        let mut file = File::open(path)?;
        let header_size = <Header as BitRead<LittleEndian>>::bit_size().unwrap() / 8;
        let mut buff = Vec::with_capacity(header_size);
        buff.resize(header_size, 0);
        file.read_exact(&mut buff)?;
        let mut stream = BitReadStream::<LittleEndian>::from(buff.as_slice());
        Ok(stream.read()?)
    }
}
