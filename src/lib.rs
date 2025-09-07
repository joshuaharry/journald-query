mod discover;
mod query;
mod tail;

// Core FFI bindings and types
mod ffi;
mod journal;
mod error;

pub use journal::Journal;
pub use error::{JournalError, Result};
pub use discover::{discover_hosts, discover_units, discover_hosts_and_units};
