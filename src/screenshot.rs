use crate::EventHandler;

use log::error;
use std::fs;
use std::io::Error;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use image::ImageFormat;

impl<'a> EventHandler<'a> {
    pub async fn take_status_screenshot(&mut self) {
        match self.connect().await {
            Ok(mut conn) => {
                if let Err(e) = conn.cmd("toggleconsole").await {
                    error!("Error toggling console: {}", e);
                    return;
                }

                thread::sleep(Duration::from_millis(300));

                if let Err(e) = conn.cmd("clear").await {
                    error!("Error clearing console: {}", e);
                    return;
                }

                self.send_status();

                self.take_screenshot();

                thread::sleep(Duration::from_millis(300));

                if let Err(e) = conn.cmd("toggleconsole").await {
                    error!("Error toggling console: {}", e);
                }

                self.convert_screenshot();
            }
            Err(e) => error!("Error connecting to rcon: {}", e),
        }
    }

    pub fn convert_screenshot(&mut self) {
        let screenshots_dir = crate::tf_path().join("screenshots");
        let tga_path = match self.find_last_screenshot(&screenshots_dir) {
            Ok(path) => path,
            Err(err) => {
                error!("Failed to find last screenshot: {}", err);
                return;
            }
        };

        let img = match image::open(tga_path.clone()) {
            Ok(img) => img,
            Err(err) => {
                error!("Failed to open image: {}", err);
                return;
            }
        };

        let png_path = tga_path.with_extension("png");
        match img.save_with_format(png_path, ImageFormat::Png) {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to save image: {}", err);
                return;
            }
        }

        match fs::remove_file(tga_path) {
            Ok(_) => (),
            Err(err) => error!("Failed to remove original image: {}", err),
        }
    }

    pub fn find_last_screenshot(&mut self, dir_path: &PathBuf) -> Result<PathBuf, Error> {
        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(err) => return Err(err),
        };

        let mut newest_file = None;
        let mut newest_time = 0;

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => return Err(err),
            };

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(err) => return Err(err),
            };

            let file_name = entry.file_name();
            let file_path = entry.path();

            if file_name.to_string_lossy().ends_with(".tga") {
                let modified_time = metadata
                    .modified()
                    .map_err(|err| Error::new(ErrorKind::Other, err))?
                    .duration_since(UNIX_EPOCH)
                    .map_err(|err| Error::new(ErrorKind::Other, err))?
                    .as_secs();
                if modified_time > newest_time {
                    newest_time = modified_time;
                    newest_file = Some(file_path);
                }
            }
        }

        newest_file.ok_or(Error::new(ErrorKind::NotFound, "No .tga files found"))
    }
}
