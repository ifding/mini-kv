use std::io;
use std::string::FromUtf8Error;

use failure::Fail;
use failure::_core::array::TryFromSliceError;

/// Error type for `KvStore`.
#[derive(Fail, Debug)]
pub enum KvError {
    #[fail(display = "{}", _0)]
    IO(#[cause] io::Error),

    /// Error when the key is not a valid utf8 string.
    #[fail(display = "invalid utf8 string: {}", _0)]
    InvalidUtf8String(#[cause] FromUtf8Error),

    #[fail(display = "{}", _0)]
    ReprDecode(#[cause] Box<bincode::ErrorKind>),

    #[fail(display = "{}", _0)]
    SliceDecode(#[cause] TryFromSliceError),

    /// Error when the key is not found.
    #[fail(display = "key not found")]
    KeyNotFound,

    #[fail(display = "Reach the file end")]
    EOF,

    #[fail(display = "invalid data path")]
    InvalidDataPath,
}

/// Result type for `KvStore`.
pub type Result<T> = std::result::Result<T, KvError>;

impl From<io::Error> for KvError {
    fn from(err: io::Error) -> KvError {
        KvError::IO(err)
    }
}

impl From<FromUtf8Error> for KvError {
    fn from(err: FromUtf8Error) -> KvError {
        KvError::InvalidUtf8String(err)
    }
}

impl From<Box<bincode::ErrorKind>> for KvError {
    fn from(err: Box<bincode::ErrorKind>) -> KvError {
        KvError::ReprDecode(err)
    }
}

impl From<TryFromSliceError> for KvError {
    fn from(err: TryFromSliceError) -> KvError {
        KvError::SliceDecode(err)
    }
}