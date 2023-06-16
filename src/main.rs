use async_log_watcher::LogWatcherSignal;
use rcon::{AsyncStdStream, Connection, Error};
use toml;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Start TF2 and sleep for 60s
    std::process::Command::new("steam")
        .arg("steam://rungameid/440")
        .spawn()?;
    let config: toml::Value = toml::from_str(include_str!("../config.toml")).unwrap();
    let path = &config["server"]["tf2_path"]
        .as_str()
        .expect("Could not read path");

    // spawn() returns a task handle that must be awaited somewhere to run the log reading loop
    let mut log_watcher = async_log_watcher::LogWatcher::new(path.to_string() + "/tf/console.log");
    let log_watcher_handle = log_watcher.spawn(false);

    // New task or thread or whatever. I'd prefer to not spawn tasks or include a tokio/rt dependency
    // so the caller is responsable to drive the future somewhere.
    tokio::task::spawn(async {
        // This is going to run the log reading loop.
        log_watcher_handle.await.unwrap_or_else(|_err| ());
    });

    let mut connected: bool = false;
    // log_watcher.try_read_message() for non blocking reading
    while let Some(data) = log_watcher.read_message().await {
        for line in std::str::from_utf8(&data).unwrap().split('\n') {
            if line.contains("CTFGCClientSystem::ShutdownGC") {
                std::fs::File::create(path.to_string() + "/tf/console.log")?;
                std::process::exit(1);
            }
            //println!("{}", line);
            if line.contains("Connected to") {
                println!("{}", line);
                std::thread::sleep(std::time::Duration::from_secs(10));
                connected = true;
            }
            if connected {
                if line.contains("[SOAP] Soap DM unloaded.") {
                    let mut conn = create_connection().await;
                    demo(&mut conn, "ds_record").await?;
                }

                if line.contains("[demos.tf]: Demo recording completed") {
                    let mut conn = create_connection().await;
                    demo(&mut conn, "ds_stop").await?;
                }

                if line.contains("test record") {
                    let mut conn = create_connection().await;
                    demo(&mut conn, "ds_record").await?;
                }

                if line.contains("test stop") {
                    let mut conn = create_connection().await;
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

async fn create_connection() -> Connection<AsyncStdStream> {
    let config: toml::Value = toml::from_str(include_str!("../config.toml")).unwrap();
    let address: &str = config["server"]["ip"].as_str().expect("Could not read IP");
    let password = config["server"]["password"]
        .as_str()
        .expect("Could not read password");
    return Connection::<AsyncStdStream>::builder()
        .connect(address, password)
        .await
        .expect("Can't connect");
}

async fn demo(conn: &mut Connection<AsyncStdStream>, cmd: &str) -> Result<(), Error> {
    println!("request: {}", cmd);
    let resp = conn.cmd(cmd).await?;
    println!("response: {}", resp);
    Ok(())
}
