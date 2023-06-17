use async_log_watcher::LogWatcherSignal;
use rcon::{Connection, Error};
use std::{fs::File, path::PathBuf};
use steamlocate::SteamDir;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("P-REC started");
    let steam_dir = SteamDir::locate();
    let path: PathBuf;
    match steam_dir.unwrap().app(&440) {
        Some(app) => path = app.path.join("tf").join("console.log"),
        None => panic!("Couldn't locate TF2 on this computer!"),
    }

    let mut log_watcher = async_log_watcher::LogWatcher::new(&path);
    let log_watcher_handle = log_watcher.spawn(false);

    tokio::task::spawn(async {
        log_watcher_handle.await.unwrap_or_else(|_err| ());
    });

    while let Some(data) = log_watcher.read_message().await {
        for line in std::str::from_utf8(&data).unwrap().split('\n') {
            if line.contains("[SOAP] Soap DM unloaded.") | line.contains("[P-REC] Recording...") {
                let mut conn = create_connection().await;
                send(&mut conn, "ds_record").await?;
                File::create(&path)?;
            }

            if line.contains("[LogsTF] Uploading logs...") | line.contains("[P-REC] Stop record.") {
                let mut conn = create_connection().await;
                send(&mut conn, "ds_stop").await?;
                File::create(&path)?;
            }

            if line.contains("test") {
                let mut conn = create_connection().await;
                send(&mut conn, "say test").await?;
                File::create(&path)?;
            }
        }
    }

    log_watcher
        .send_signal(LogWatcherSignal::Close)
        .await
        .unwrap();
    Ok(())
}

async fn create_connection() -> Connection<TcpStream> {
    return Connection::<TcpStream>::builder()
        .connect("127.0.0.1:27015", "prec")
        .await
        .expect("Can't connect");
}

async fn send(conn: &mut Connection<TcpStream>, cmd: &str) -> Result<(), Error> {
    conn.cmd(cmd).await?;
    Ok(())
}
