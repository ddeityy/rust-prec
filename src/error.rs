use bitbuffer::BitError;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Failed to parse demo header: {0:#}")]
    Header(#[from] BitError),
    #[error(transparent)]
    Rcon(#[from] rcon::Error),
    #[error("Demo path doesn't match expected format")]
    InvalidDemoPath,
    #[error("Highlight extraction failed")]
    Highlight(#[from] Box<dyn std::error::Error>),
}
