use async_log_watcher::LogWatcher;
use debounce::EventDebouncer;
use futures_channel::mpsc::{Receiver, Sender};
use log::info;
use rcon::{Connection, Error};
use std::env::var;
use std::fs::OpenOptions;
use std::path::PathBuf;
use steamlocate::SteamDir;
use tokio::net::TcpStream;
use tokio::time::Duration;

#[derive(PartialEq)]
enum ConsoleEvent {
    Record,
    Stop,
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

    let (sender, mut receiver): (Sender<ConsoleEvent>, Receiver<ConsoleEvent>) =
        futures_channel::mpsc::channel(1024);

    let delay = Duration::from_millis(100);
    let debouncer = EventDebouncer::new(delay, move |event: ConsoleEvent| {
        match_event(event, sender.clone())
    });

    tokio::spawn(async move {
        while let Some(event) = receiver.try_next().unwrap_or_default() {
            match event {
                ConsoleEvent::Record => record(&rcon_password).await,
                ConsoleEvent::Stop => stop(&rcon_password).await,
            }
        }
    });

    while let Some(data) = log_watcher.read_message().await {
        for line in String::from_utf8(data).unwrap_or_default().split('\n') {
            if line.contains("[SOAP] Soap DM unloaded.") | line.contains("[P-REC] Recording...") {
                debouncer.put(ConsoleEvent::Record);
            }

            if line.contains("[LogsTF] Uploading logs...") | line.contains("[P-REC] Stop record.") {
                debouncer.put(ConsoleEvent::Stop);
            }
        }
    }
    Ok(())
}

async fn record(rcon_password: &str) {
    let mut conn = create_connection(&rcon_password).await;
    conn.cmd("ds_record").await.unwrap();
    info!("Started recording");
}

async fn stop(rcon_password: &str) {
    let mut conn = create_connection(&rcon_password).await;
    conn.cmd("ds_stop").await.unwrap();
    info!("Stopped recording");
}

fn match_event(event: ConsoleEvent, mut sender: Sender<ConsoleEvent>) {
    match event {
        ConsoleEvent::Record => sender.try_send(event).unwrap_or_default(),
        ConsoleEvent::Stop => sender.try_send(event).unwrap_or_default(),
    }
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
