mod discover;
mod query;
mod tail;

// Core FFI bindings and types
mod ffi;
mod journal;
mod error;

pub use journal::Journal;
pub use error::{JournalError, Result};
pub use discover::{discover_services, Host, Hosts};
pub use query::{query_journal, Query, Entry};
pub use tail::{TailConfig, JournalTail, JournalIterator};
