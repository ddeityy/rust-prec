use crate::EventHandler;

use log::error;

use std::thread;
use std::time::Duration;

impl<'a> EventHandler<'a> {
    pub async fn take_status_screenshot(&mut self) {
        match self.connect().await {
            Ok(mut conn) => match conn.cmd("toggleconsole").await {
                Ok(_) => {
                    thread::sleep(Duration::from_millis(500));

                    match conn.cmd("clear").await {
                        Ok(_) => {
                            self.send_status();

                            thread::sleep(Duration::from_millis(300));

                            self.take_screenshot();

                            match conn.cmd("toggleconsole").await {
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
}
// fn take_screenshot(path: &PathBuf) {
//     match Window::all() {
//         Ok(windows) => {
//             for window in windows {
//                 if window.is_minimized() {
//                     continue;
//                 }

//                 if window.app_name().contains("tf_") {
//                     let mut p = path.clone();
//                     p.set_extension("png");
//                     match window.capture_image() {
//                         Ok(image) => match image.save(&p) {
//                             Ok(_) => (),
//                             Err(e) => error!("Error saving image: {}", e),
//                         },
//                         Err(e) => error!("Error capturing image: {}", e),
//                     }
//                 }
//             }
//         }
//         Err(e) => error!("Error getting all windows: {}", e),
//     }
// }

// fn send_2() {
//     let mut enigo = match Enigo::new(&Settings::default()) {
//         Ok(enigo) => enigo,
//         Err(e) => {
//             error!("Failed to initialize Enigo: {}", e);
//             return;
//         }
//     };

//     match enigo.text("status") {
//         Ok(_) => (),
//         Err(e) => {
//             error!("Failed to send text 'status': {}", e);
//             return;
//         }
//     }

//     match enigo.key(Key::Return, Press) {
//         Ok(_) => (),
//         Err(e) => {
//             error!("Failed to press Return key: {}", e);
//             return;
//         }
//     }

//     match enigo.key(Key::Return, Release) {
//         Ok(_) => (),
//         Err(e) => {
//             error!("Failed to release Return key: {}", e);
//             return;
//         }
//     }
// }
