use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ProtoError {
    Crypto,
    Serde(serde_json::Error),
    Replay,
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProtoError::Crypto => write!(f, "decryption failed: invalid key or tampered message"),
            ProtoError::Serde(e) => write!(f, "invalid message format: {e}"),
            ProtoError::Replay => write!(f, "replayed message: counter not greater than last seen"),
        }
    }
}

impl error::Error for ProtoError {}

impl From<serde_json::Error> for ProtoError {
    fn from(e: serde_json::Error) -> Self {
        ProtoError::Serde(e)
    }
}
