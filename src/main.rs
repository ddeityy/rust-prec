use async_log_watcher::LogWatcherSignal;
use rcon::{AsyncStdStream, Connection, Error};
use toml;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config: toml::Value = toml::from_str(include_str!("../config.toml")).unwrap();

    let address = config["server"]["ip"].as_str().expect("Could not read IP");
    let path = config["server"]["tf2_path"]
        .as_str()
        .expect("Could not read path");
    let password = config["server"]["password"]
        .as_str()
        .expect("Could not read password");

    let mut conn = <Connection<AsyncStdStream>>::builder()
        .connect(address, password)
        .await?;
    let mut log_watcher = async_log_watcher::LogWatcher::new(path);

    // spawn() returns a task handle that must be awaited somewhere to run the log reading loop
    let log_watcher_handle = log_watcher.spawn(false);

    // New task or thread or whatever. I'd prefer to not spawn tasks or include a tokio/rt dependency
    // so the caller is responsable to drive the future somewhere.
    tokio::task::spawn(async {
        // This is going to run the log reading loop.
        log_watcher_handle.await.unwrap_or_else(|_err| ());
    });

    // log_watcher.try_read_message() for non blocking reading
    while let Some(data) = log_watcher.read_message().await {
        for line in std::str::from_utf8(&data).unwrap().split('\n') {
            println!("{}", line);
            match line {
                "[SOAP] Soap DM unloaded." => demo(&mut conn, "ds_record").await?,
                "[demos.tf]: Demo recording completed" => demo(&mut conn, "ds_stop").await?,
                _ => (),
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

async fn demo(conn: &mut Connection<AsyncStdStream>, cmd: &str) -> Result<(), Error> {
    println!("request: {}", cmd);
    let resp = conn.cmd(cmd).await?;
    println!("response: {}", resp);
    Ok(())
}
