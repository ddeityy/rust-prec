mod throttle;

use log::{error, info, trace};
use rcon::{Connection, Error};
use std::env::var;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::str;
use steamlocate::SteamDir;
use throttle::Throttler;
use tokio::net::{TcpStream, UdpSocket};
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
        let builder = Connection::<TcpStream>::builder()
            .connect("127.0.0.1:27015", rcon_password)
            .await;
        let Ok(mut conn) = builder else {
            error!("Failed to connect to rcon");
            return;
        };
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

    let path = log_path();

    // make sure the file exists
    OpenOptions::new().write(true).create(true).open(&path)?;

    let sock = UdpSocket::bind("0.0.0.0:27016").await?;

    let mut buf = [0; 8192];
    let delay = Duration::from_millis(7500);
    let mut throttler = Throttler::new(delay);
    loop {
        let (len, _addr) = sock.recv_from(&mut buf).await?;

        if let Some(message) = parse_udp_log(&buf[0..len]) {
            let line = message.message;
            trace!("got log line: {line}");
            if let Some(event) = ConsoleEvent::from_chat(line.trim()) {
                if let Some(event) = throttler.debounce(event) {
                    event.send(&rcon_password).await;
                }
            }
        }
    }
}

fn log_path() -> PathBuf {
    let dir = SteamDir::locate().unwrap();
    dir.path()
        .join("steamapps")
        .join("common")
        .join("Team Fortress 2")
        .join("tf")
        .join("console.log")
}

struct LogMessage<'a> {
    #[allow(dead_code)]
    password: Option<&'a [u8]>,
    message: &'a str,
}

fn parse_udp_log(data: &[u8]) -> Option<LogMessage> {
    let header = data.get(0..4)?;
    if header != [255; 4] {
        return None;
    }

    let packet_type = data.get(4)?;
    let (password, data) = if *packet_type == 0x53 {
        (Some(data.get(5..9)?), data.get(9..)?)
    } else {
        (None, data.get(5..)?)
    };
    Some(LogMessage {
        password,
        message: str::from_utf8(data).ok()?.trim(),
    })
}
