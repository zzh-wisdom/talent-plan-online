use std::io;
use std::num::ParseIntError;

/// Error type for storage
#[derive(Debug)]
pub struct StorageError {
    // For the sake of simplicity, only the wrong information is saved.
    message: String,
}

impl From<io::Error> for StorageError {
    fn from(err: io::Error) -> StorageError {
        StorageError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> StorageError {
        StorageError {
            message: err.to_string(),
        }
    }
}

impl From<ParseIntError> for StorageError {
    fn from(err: ParseIntError) -> StorageError {
        StorageError {
            message: err.to_string(),
        }
    }
}

impl ToString for StorageError {
    fn to_string(&self) -> String {
        self.message.clone()
    }
}

/// Result type for storage
pub type Result<T> = std::result::Result<T, StorageError>;
