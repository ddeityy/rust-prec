use async_log_watcher::LogWatcher;
use log::info;
use rcon::{Connection, Error};
use std::env::var;
use std::fs::OpenOptions;
use std::path::PathBuf;
use steamlocate::SteamDir;
use tokio::net::TcpStream;

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

    while let Some(data) = log_watcher.read_message().await {
        for line in String::from_utf8(data).unwrap_or_default().split('\n') {
            if line.contains("[SOAP] Soap DM unloaded.") | line.contains("[P-REC] Recording...") {
                let mut conn = create_connection(&rcon_password).await;
                conn.cmd("ds_record").await?;
                info!("Started recording");
            }

            if line.contains("[LogsTF] Uploading logs...") | line.contains("[P-REC] Stop record.") {
                let mut conn = create_connection(&rcon_password).await;
                conn.cmd("ds_stop").await?;
                info!("Stopped recording");
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
