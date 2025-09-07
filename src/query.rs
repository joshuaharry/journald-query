use std::path::Path;
use crate::journal::Journal;
use crate::error::JournalError;

/// Represents a query for filtering journal entries.
///
/// Fields:
/// - `hostname`: Optional hostname to filter by (`_HOSTNAME` field).
/// - `unit`: Optional systemd unit to filter by (`_SYSTEMD_UNIT` field).
/// - `start_time_utc`: Start of the time range (inclusive), in microseconds since Unix epoch (UTC).
/// - `end_time_utc`: End of the time range (inclusive), in microseconds since Unix epoch (UTC).
/// - `message_contains`: Optional substring to match within the `MESSAGE` field.
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub hostname: Option<String>,
    pub unit: Option<String>,
    pub start_time_utc: u64,
    pub end_time_utc: u64,
    pub message_contains: Option<String>,
}

/// Represents a single journal entry returned by a query.
///
/// Fields:
/// - `hostname`: Hostname from the `_HOSTNAME` field, if present.
/// - `unit`: Systemd unit from the `_SYSTEMD_UNIT` field, if present.
/// - `timestamp_utc`: Timestamp of the entry in microseconds since Unix epoch (UTC).
/// - `message`: The log message (`MESSAGE` field).
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub hostname: Option<String>,
    pub unit: Option<String>,
    pub timestamp_utc: u64,
    pub message: String,
}

impl Query {
    /// Create a new query with time range
    pub fn new(start_time_utc: u64, end_time_utc: u64) -> Self {
        Query {
            hostname: None,
            unit: None,
            start_time_utc,
            end_time_utc,
            message_contains: None,
        }
    }

    /// Filter by hostname
    pub fn hostname<S: Into<String>>(mut self, hostname: S) -> Self {
        self.hostname = Some(hostname.into());
        self
    }

    /// Filter by systemd unit
    pub fn unit<S: Into<String>>(mut self, unit: S) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Filter by message content (substring match)
    pub fn message_contains<S: Into<String>>(mut self, message: S) -> Self {
        self.message_contains = Some(message.into());
        self
    }
}

/// Query journal entries with the given filters
/// 
/// This function applies the specified filters and returns all matching journal entries
/// within the time range, sorted by timestamp.
/// 
/// # Arguments
/// * `journal_dir` - Directory containing journal files
/// * `query` - Query parameters including time range and optional filters
/// 
/// # Returns
/// A vector of matching entries sorted by timestamp
/// 
/// # Examples
/// ```no_run
/// use journald_query::{Query, query_journal};
/// use std::path::Path;
/// 
/// // Query all entries from last hour
/// let now = 1640995200000000; // Example timestamp
/// let hour_ago = now - (60 * 60 * 1_000_000); // 1 hour in microseconds
/// let query = Query::new(hour_ago, now)
///     .hostname("web-server")
///     .unit("nginx.service");
/// 
/// let entries = query_journal(Path::new("/var/log/journal"), query)?;
/// for entry in entries {
///     println!("{}: {}", entry.timestamp_utc, entry.message);
/// }
/// # Ok::<(), journald_query::JournalError>(())
/// ```
pub fn query_journal(journal_dir: &Path, query: Query) -> Result<Vec<Entry>, JournalError> {
    let journal = Journal::open_directory(journal_dir)?;
    
    // Clear any existing matches
    journal.flush_matches();
    
    // Add hostname filter if specified
    if let Some(hostname) = &query.hostname {
        journal.add_match("_HOSTNAME", hostname)?;
    }
    
    // Add unit filter if specified
    if let Some(unit) = &query.unit {
        journal.add_match("_SYSTEMD_UNIT", unit)?;
    }
    
    // Seek to the start time
    journal.seek_realtime_usec(query.start_time_utc)?;
    
    let mut entries = Vec::new();
    
    // Iterate through entries
    while journal.next()? {
        // Get timestamp and check if we've exceeded end time
        let timestamp = journal.get_realtime_usec()?;
        if timestamp > query.end_time_utc {
            break;
        }
        
        // Get entry fields and strip field name prefixes
        let hostname = journal.get_field("_HOSTNAME")?
            .and_then(|raw| raw.strip_prefix("_HOSTNAME=").map(|s| s.to_string()));
        let unit = journal.get_field("_SYSTEMD_UNIT")?
            .and_then(|raw| raw.strip_prefix("_SYSTEMD_UNIT=").map(|s| s.to_string()));
        let message = journal.get_field("MESSAGE")?
            .and_then(|raw| raw.strip_prefix("MESSAGE=").map(|s| s.to_string()))
            .unwrap_or_else(|| "(no message)".to_string());
        
        // Apply message filter if specified
        if let Some(filter_text) = &query.message_contains {
            if !message.contains(filter_text) {
                continue;
            }
        }
        
        // Create entry
        let entry = Entry {
            hostname,
            unit,
            timestamp_utc: timestamp,
            message,
        };
        
        entries.push(entry);
    }
    
    // Entries should already be in chronological order from journal iteration
    Ok(entries)
}