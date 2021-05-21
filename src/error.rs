use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("data is not a uasset")]
    InvalidFile,
    #[error("asset has unsupported legacy version value {0:?}")]
    UnsupportedVersion(i32),
    #[error("asset saved without asset version information")]
    UnversionedAsset,
    #[error("failed to parse data: {0:?}")]
    ParseError(binread::Error),
    #[error("failed to seek in stream: {0:?}")]
    IoError(std::io::Error),
}

impl From<binread::Error> for Error {
    fn from(error: binread::Error) -> Self {
        Error::ParseError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}