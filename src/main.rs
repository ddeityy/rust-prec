use async_log_watcher::LogWatcher;
use debounce::EventDebouncer;
use futures::stream::StreamExt;
use futures_channel::mpsc::{Receiver, Sender};
use log::{error, info};
use rcon::{Connection, Error};
use std::env::var;
use std::fs::OpenOptions;
use std::path::PathBuf;
use steamlocate::SteamDir;
use tokio::net::TcpStream;
use tokio::time::Duration;

#[derive(PartialEq, Debug)]
enum ConsoleEvent {
    Record,
    Stop,
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

    async fn send(&self, rcon_password: &str) {
        let mut conn = create_connection(&rcon_password).await;
        if let Err(e) = conn.cmd(self.command()).await {
            error!("Error while sending rcon event: {e:?}")
        }
        info!("Sending {:?}", self);
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    info!("P-REC started");

    let rcon_password = var("RCON_PASSWORD").unwrap_or_else(|_| "prec".to_string());

    let Some(path) = log_path() else {
        eprintln!("Couldn't locate TF2 on this computer!");
        return Ok(());
    };

    // make sure the file exists
    OpenOptions::new().write(true).create(true).open(&path)?;

    let mut log_watcher = LogWatcher::new(&path);
    let log_watcher_handle = log_watcher.spawn(true);
    tokio::task::spawn(log_watcher_handle);

    let (mut sender, mut receiver): (Sender<ConsoleEvent>, Receiver<ConsoleEvent>) =
        futures_channel::mpsc::channel(1024);

    let delay = Duration::from_millis(100);
    let debouncer = EventDebouncer::new(delay, move |event: ConsoleEvent| {
        sender.try_send(event).expect("receiver was closed")
    });

    tokio::spawn(async move {
        while let Some(event) = receiver.next().await {
            event.send(&rcon_password).await;
        }
    });

    while let Some(data) = log_watcher.read_message().await {
        for line in String::from_utf8(data).unwrap_or_default().split('\n') {
            if let Some(event) = ConsoleEvent::from_chat(line) {
                debouncer.put(event);
            }
        }
    }
    Ok(())
}

fn log_path() -> Option<PathBuf> {
    match SteamDir::locate() {
        Some(dir) => Some(
            dir.path
                .join("steamapps")
                .join("common")
                .join("Team Fortress 2")
                .join("tf")
                .join("console.log"),
        ),
        None => None,
    }
}

async fn create_connection(rcon_password: &str) -> Connection<TcpStream> {
    Connection::<TcpStream>::builder()
        .connect("127.0.0.1:27015", rcon_password)
        .await
        .expect("Can't connect")
}
