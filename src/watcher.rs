// adapter from https://github.com/aravindavk/logwatcher/blob/master/src/lib.rs

use log::{debug, info};
use std::fs::File;
use std::io;
use std::io::SeekFrom;
use std::io::{BufRead, BufReader, Seek};

use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

const DELAY: Duration = Duration::from_secs(1);

pub struct LogWatcher {
    path: PathBuf,
    reader: Option<LineReader>,
}

impl LogWatcher {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        LogWatcher {
            path: path.into(),
            reader: None,
        }
    }

    fn get_reader(&mut self) -> Result<&mut LineReader, io::Error> {
        if self.reader.is_none() {
            self.reader = Some(LineReader::new(self.path.clone())?);
        }

        Ok(self.reader.as_mut().unwrap())
    }
}

impl Iterator for LogWatcher {
    type Item = Result<String, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let reader = match self.get_reader() {
                Ok(reader) => reader,
                Err(e) => return Some(Err(e)),
            };

            match reader.next() {
                Some(line) => return Some(line),
                None => {
                    info!("{} rotated, reopening", self.path.display());
                    self.reader = None;
                }
            }

            sleep(DELAY);
        }
    }
}

struct LineReader {
    path: PathBuf,
    reader: BufReader<File>,
    ino: u64,
    pos: u64,
}

impl LineReader {
    pub fn new(path: PathBuf) -> Result<LineReader, io::Error> {
        debug!("opening {}", path.display());
        let file = File::open(&path)?;
        let meta = file.metadata()?;
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(LineReader {
                path,
                ino: meta.ino(),
                pos: meta.len(),
                reader: BufReader::new(file),
            })
        }
        #[cfg(target_os = "windows")]
        {
            Ok(LineReader {
                path,
                ino: 0,
                pos: meta.len(),
                reader: BufReader::new(file),
            })
        }
    }

    fn current_ino(&self) -> Result<u64, io::Error> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(self.path.metadata()?.ino())
        }
        #[cfg(target_os = "windows")]
        {
            Ok(0)
        }
    }
}

impl Iterator for LineReader {
    type Item = Result<String, io::Error>;

    /// Read lines from the file, blocking until a new lines is available.
    ///
    /// If the file gets rotated (the original filename points to a new file),
    /// `None` is returned.
    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        loop {
            // discard the buffer to ensure we get any new lines
            self.reader.seek(SeekFrom::Start(self.pos)).unwrap();

            match self.reader.read_line(&mut line) {
                Ok(0) => {
                    #[cfg(target_os = "linux")]
                    {
                        let current_ino = match self.current_ino() {
                            Ok(ino) => ino,
                            Err(e) => return Some(Err(e)),
                        };
                        if current_ino != self.ino {
                            return None;
                        } else {
                            sleep(DELAY);
                        }
                    }
                    #[cfg(target_os = "windows")]
                    {
                        sleep(DELAY);
                    }
                }
                Ok(bytes) => {
                    self.pos += bytes as u64;
                    return Some(Ok(line));
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
