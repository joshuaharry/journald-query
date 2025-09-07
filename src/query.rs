use std::path::Path;
use crate::journal::Journal;
use crate::error::JournalError;

#[derive(Debug, Clone)]
pub struct Query {
    hostname: String,
    unit: String,
    start_time_utc: u64,
    end_time_utc: u64,
    message: String,
}

pub fn query_journal(journal_dir: &Path, query: Query) -> Result<Vec<String>, JournalError> {
    let journal = Journal::open_directory(journal_dir)?;
}