use async_log_watcher::LogWatcherSignal;
use rcon::{Connection, Error};
use steamlocate::SteamDir;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Start TF2
    //std::process::Command::new("steam")
    //    .arg("steam://rungameid/440")
    //    .spawn()?;

    let mut path = SteamDir::locate().unwrap();
    match path.app(&440) {
        Some(_) => (),
        None => panic!("Couldn't locate TF2 on this computer!"),
    }

    // spawn() returns a task handle that must be awaited somewhere to run the log reading loop
    let mut log_watcher =
        async_log_watcher::LogWatcher::new(path.path.join("tf").join("console.log"));
    let log_watcher_handle = log_watcher.spawn(false);

    // New task or thread or whatever. I'd prefer to not spawn tasks or include a tokio/rt dependency
    // so the caller is responsable to drive the future somewhere.
    tokio::task::spawn(async {
        // This is going to run the log reading loop.
        log_watcher_handle.await.unwrap_or_else(|_err| ());
    });

    let mut connected: bool = false;
    let mut conn = create_connection().await;
    while let Some(data) = log_watcher.read_message().await {
        for line in std::str::from_utf8(&data).unwrap().split('\n') {
            if line.contains("CTFGCClientSystem::ShutdownGC") {
                std::fs::File::create(path.path.join("tf").join("console.log"))?;
                std::process::exit(1);
            }
            if line.contains("Connected to") {
                println!("{}", line);
                std::thread::sleep(std::time::Duration::from_secs(10));
                connected = true;
            }
            if connected {
                if line.contains("[SOAP] Soap DM unloaded.") | line.contains("[P-REC] Recording...")
                {
                    demo(&mut conn, "ds_record").await?;
                }

                if line.contains("[LogsTF] Uploading logs...")
                    | line.contains("[P-REC] Stop record.")
                {
                    demo(&mut conn, "ds_stop").await?;
                }
            }
        }
    }

    // Close log watcher by sending some command. You can also reload the file or change the file being read
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

async fn demo(conn: &mut Connection<TcpStream>, cmd: &str) -> Result<(), Error> {
    println!("request: {}", cmd);
    let resp = conn.cmd(cmd).await?;
    println!("response: {}", resp);
    Ok(())
}
