use crate::EventHandler;
use log::error;
use rdev::{simulate, EventType, Key, SimulateError};
use std::{env::var, path::PathBuf, thread, time::Duration};
use xcap::Window;

pub async fn take_status_screenshot(path: &PathBuf) {
    let rcon_password = var("RCON_PASSWORD").unwrap_or_else(|_| "prec".to_string());
    let toggle_console = "toggleconsole";

    let handler = EventHandler::new(rcon_password);

    match handler.connect().await {
        Ok(mut conn) => match conn.cmd(&toggle_console).await {
            Ok(_) => {
                thread::sleep(Duration::from_millis(500));

                match conn.cmd("clear").await {
                    Ok(_) => {
                        send_status();
                        thread::sleep(Duration::from_millis(500));

                        take_screenshot(path);

                        match conn.cmd(&toggle_console).await {
                            Ok(_) => (),
                            Err(e) => error!("Error toggling console: {}", e),
                        }
                    }
                    Err(e) => error!("Error clearing console: {}", e),
                }
            }
            Err(e) => error!("Error toggling console: {}", e),
        },
        Err(e) => error!("Error connecting to rcon: {}", e),
    }
}

fn take_screenshot(path: &PathBuf) {
    match Window::all() {
        Ok(windows) => {
            for window in windows {
                if window.is_minimized() {
                    continue;
                }

                if window.app_name().contains("tf_") {
                    let mut p = path.clone();
                    p.set_extension("png");
                    match window.capture_image() {
                        Ok(image) => match image.save(&p) {
                            Ok(_) => (),
                            Err(e) => error!("Error saving image: {}", e),
                        },
                        Err(e) => error!("Error capturing image: {}", e),
                    }
                }
            }
        }
        Err(e) => error!("Error getting all windows: {}", e),
    }
}

fn send(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            error!("Error sending keyboard press {:?}", event_type);
        }
    }
}

fn send_status() {
    send(&EventType::KeyPress(Key::KeyS));
    send(&EventType::KeyRelease(Key::KeyS));

    send(&EventType::KeyPress(Key::KeyT));
    send(&EventType::KeyRelease(Key::KeyT));

    send(&EventType::KeyPress(Key::KeyA));
    send(&EventType::KeyRelease(Key::KeyA));

    send(&EventType::KeyPress(Key::KeyT));
    send(&EventType::KeyRelease(Key::KeyT));

    send(&EventType::KeyPress(Key::KeyU));
    send(&EventType::KeyRelease(Key::KeyU));

    send(&EventType::KeyPress(Key::KeyS));
    send(&EventType::KeyRelease(Key::KeyS));

    send(&EventType::KeyPress(Key::Return));
    send(&EventType::KeyRelease(Key::Return));
}
