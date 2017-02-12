use std::any::Any;
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::sync::mpsc::{RecvError, SendError};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum UnsupportedFeature {
    Hierarchical,
    Lossless,
    ArithmeticEntropyCoding,
    SamplePrecision(u8),
    ComponentCount(u8),
    // DNL/zero height
    DNL,
    SubsamplingRatio,
}

#[derive(Debug)]
pub enum Error {
    Format(String),
    Unsupported(UnsupportedFeature),
    Io(IoError),
    Internal(Box<StdError>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Format(ref desc)      => write!(f, "invalid JPEG format: {}", desc),
            Error::Unsupported(ref feat) => write!(f, "unsupported JPEG feature: {:?}", feat),
            Error::Io(ref err)           => err.fmt(f),
            Error::Internal(ref err)     => err.fmt(f),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Format(_)         => "invalid JPEG format",
            Error::Unsupported(_)    => "unsupported JPEG feature",
            Error::Io(ref err)       => err.description(),
            Error::Internal(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Internal(ref err) => Some(&**err),
            _ => None,
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl From<RecvError> for Error {
    fn from(err: RecvError) -> Error {
        Error::Internal(Box::new(err))
    }
}

impl<T: Any + Send> From<SendError<T>> for Error {
    fn from(err: SendError<T>) -> Error {
        Error::Internal(Box::new(err))
    }
}
