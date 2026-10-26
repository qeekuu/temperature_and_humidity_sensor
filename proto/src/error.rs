use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ProtoError {
    Crypto,
    Serde(serde_json::Error),
    Replay,
    Io(std::io::Error),
    InvalidKeyLength(usize),
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProtoError::Crypto => write!(f, "decryption failed: invalid key or tampered message"),
            ProtoError::Serde(e) => write!(f, "invalid message format: {e}"),
            ProtoError::Replay => write!(f, "replayed message: counter not greater than last seen"),
            ProtoError::Io(e) => write!(f, "io error: {e}"),
            ProtoError::InvalidKeyLength(len) => {
                write!(f, "invalid key length, expected 32 bytes got: {len}")
            }
        }
    }
}

impl error::Error for ProtoError {}

impl From<serde_json::Error> for ProtoError {
    fn from(e: serde_json::Error) -> Self {
        ProtoError::Serde(e)
    }
}

impl From<std::io::Error> for ProtoError {
    fn from(e: std::io::Error) -> Self {
        ProtoError::Io(e)
    }
}
