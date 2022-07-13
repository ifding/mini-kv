use std::io;
use std::string::FromUtf8Error;
use std::array::TryFromSliceError;

use thiserror::Error;

/// Error type for kvs.
#[derive(Error, Debug)]
pub enum KvsError {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("SliceDecode: {0}")]
    SliceDecode(#[from] TryFromSliceError),

    #[error("ReprDecode: {0}")]
    ReprDecode(#[from] Box<bincode::ErrorKind>),

    #[error("StringDecode: {0}")]
    StringDecode(#[from] FromUtf8Error),

    #[error("Key not found")]
    KeyNotFound,

    #[error("Reach the file end")]
    EOF,

    #[error("failed to read the file")]
    InvalidDataPath,
}

/// Result type for kvs.
pub type Result<T> = std::result::Result<T, KvsError>;
