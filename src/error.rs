use std::fmt;

/// Result type for journal operations
pub type Result<T> = std::result::Result<T, JournalError>;

/// Errors that can occur during journal operations
#[derive(Debug, Clone, PartialEq)]
pub enum JournalError {
    /// Invalid argument provided
    InvalidArgument,
    /// Journal object was created in a different process/thread
    CrossThreadUsage,
    /// Read pointer is not positioned at a valid entry
    NotPositioned,
    /// The requested field/entry does not exist
    NotFound,
    /// Memory allocation failed
    OutOfMemory,
    /// A compressed entry is too large
    BufferTooSmall,
    /// Data field is too large for this architecture
    DataTooLarge,
    /// Journal uses an unsupported compression or feature
    ProtocolNotSupported,
    /// Journal is corrupted
    BadMessage,
    /// I/O error occurred
    IoError,
    /// Unknown error code from systemd
    Unknown(i32),
}

impl JournalError {
    /// Convert a systemd error code to a JournalError
    pub fn from_errno(errno: i32) -> Self {
        match -errno {
            libc::EINVAL => JournalError::InvalidArgument,
            libc::ECHILD => JournalError::CrossThreadUsage,
            libc::EADDRNOTAVAIL => JournalError::NotPositioned,
            libc::ENOENT => JournalError::NotFound,
            libc::ENOMEM => JournalError::OutOfMemory,
            libc::ENOBUFS => JournalError::BufferTooSmall,
            libc::E2BIG => JournalError::DataTooLarge,
            libc::EPROTONOSUPPORT => JournalError::ProtocolNotSupported,
            libc::EBADMSG => JournalError::BadMessage,
            libc::EIO => JournalError::IoError,
            code => JournalError::Unknown(code),
        }
    }
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JournalError::InvalidArgument => write!(f, "Invalid argument provided"),
            JournalError::CrossThreadUsage => write!(f, "Journal object used from different thread"),
            JournalError::NotPositioned => write!(f, "Read pointer not positioned at valid entry"),
            JournalError::NotFound => write!(f, "Requested field or entry not found"),
            JournalError::OutOfMemory => write!(f, "Memory allocation failed"),
            JournalError::BufferTooSmall => write!(f, "Compressed entry too large"),
            JournalError::DataTooLarge => write!(f, "Data field too large for architecture"),
            JournalError::ProtocolNotSupported => write!(f, "Unsupported compression or feature"),
            JournalError::BadMessage => write!(f, "Journal is corrupted"),
            JournalError::IoError => write!(f, "I/O error occurred"),
            JournalError::Unknown(code) => write!(f, "Unknown error code: {}", code),
        }
    }
}

impl std::error::Error for JournalError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        assert_eq!(JournalError::from_errno(-libc::EINVAL), JournalError::InvalidArgument);
        assert_eq!(JournalError::from_errno(-libc::ECHILD), JournalError::CrossThreadUsage);
        assert_eq!(JournalError::from_errno(-libc::ENOENT), JournalError::NotFound);
        assert_eq!(JournalError::from_errno(-999), JournalError::Unknown(999));
    }

    #[test]
    fn test_error_display() {
        let err = JournalError::InvalidArgument;
        assert_eq!(err.to_string(), "Invalid argument provided");
    }
}
