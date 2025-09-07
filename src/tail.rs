use crate::error::{JournalError, Result};
use crate::ffi;
use crate::query::Entry;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;
use std::time::Duration;

/// Configuration for tailing journal entries from a specific service
#[derive(Debug, Clone, PartialEq)]
pub struct TailConfig {
    /// Hostname to filter by (_HOSTNAME field)
    pub hostname: String,
    /// Service/unit name to filter by (_SYSTEMD_UNIT field)
    pub service: String,
    /// Path to journal directory 
    pub journal_path: String,
    /// Polling interval for checking new entries (default: 100ms)
    pub poll_interval: Duration,
}

impl TailConfig {
    /// Create a new tail configuration with default polling interval (100ms)
    /// 
    /// # Arguments
    /// * `hostname` - The hostname to filter journal entries by
    /// * `service` - The systemd service/unit name to filter by
    /// * `journal_path` - Path to the journal directory
    /// 
    /// # Examples
    /// ```
    /// use journald_query::tail::TailConfig;
    /// 
    /// let config = TailConfig::new("web-server-01", "nginx.service", "/var/log/journal");
    /// ```
    pub fn new<H: Into<String>, S: Into<String>, P: Into<String>>(hostname: H, service: S, journal_path: P) -> Self {
        Self {
            hostname: hostname.into(),
            service: service.into(),
            journal_path: journal_path.into(),
            poll_interval: Duration::from_millis(100), // Default 100ms polling
        }
    }

    /// Set a custom polling interval
    /// 
    /// # Arguments
    /// * `interval` - Duration between polls for new journal entries
    /// 
    /// # Examples
    /// ```
    /// use journald_query::tail::TailConfig;
    /// use std::time::Duration;
    /// 
    /// let config = TailConfig::new("web-server-01", "nginx.service", "/var/log/journal")
    ///     .with_poll_interval(Duration::from_millis(50)); // 50ms for higher responsiveness
    /// ```
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Set polling interval in milliseconds (convenience method)
    /// 
    /// # Arguments
    /// * `millis` - Polling interval in milliseconds
    /// 
    /// # Examples
    /// ```
    /// use journald_query::tail::TailConfig;
    /// 
    /// let config = TailConfig::new("web-server-01", "nginx.service", "/var/log/journal")
    ///     .with_poll_interval_ms(250); // 250ms polling
    /// ```
    pub fn with_poll_interval_ms(mut self, millis: u64) -> Self {
        self.poll_interval = Duration::from_millis(millis);
        self
    }
}

/// A live tail of journal entries for a specific hostname and service
/// 
/// This struct provides an iterator interface for streaming journal entries
/// in real-time. It positions itself at the end of the journal and waits
/// for new entries matching the specified hostname and service.
#[derive(Debug)]
pub struct JournalTail {
    handle: *mut ffi::SdJournal,
    config: TailConfig,
    // PhantomData to make this !Send + !Sync (not thread-safe)
    _not_thread_safe: PhantomData<*const ()>,
}

impl JournalTail {
    /// Create a new journal tail for the specified hostname and service
    /// 
    /// This will:
    /// 1. Open the journal (system journal or specified path)
    /// 2. Add filters for the hostname and service
    /// 3. Seek to the tail (end) of the journal
    /// 
    /// # Arguments
    /// * `config` - Configuration specifying hostname, service, and optional journal path
    /// 
    /// # Returns
    /// A new JournalTail instance ready for iteration
    /// 
    /// # Examples
    /// ```no_run
    /// use journald_query::tail::{TailConfig, JournalTail};
    /// 
    /// let config = TailConfig::new("web-server-01", "nginx.service", "/var/log/journal");
    /// let tail = JournalTail::new(config)?;
    /// 
    /// for entry in tail.iter() {
    ///     match entry {
    ///         Ok(entry) => println!("{}: {}", entry.timestamp_utc, entry.message),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn new(config: TailConfig) -> Result<Self> {
        // Open the journal
        let handle = Self::open_journal(&config)?;
        
        // Create the tail instance
        let mut tail = Self {
            handle,
            config,
            _not_thread_safe: PhantomData,
        };
        
        // Set up filters and position
        tail.setup_filters()?;
        tail.seek_to_tail()?;
        
        Ok(tail)
    }
    
    /// Get an iterator over journal entries
    /// 
    /// The iterator will block on each call to `next()` until a new entry
    /// matching the filters becomes available.
    pub fn iter(&mut self) -> JournalIterator<'_> {
        JournalIterator { tail: self }
    }
    
    // Private helper methods
    
    fn open_journal(config: &TailConfig) -> Result<*mut ffi::SdJournal> {
        let mut handle: *mut ffi::SdJournal = ptr::null_mut();


        
        let result = {
            // Open from specified directory
            let path_cstr = CString::new(config.journal_path.as_str())
                .map_err(|_| JournalError::InvalidArgument)?;
            
            unsafe {
                ffi::sd_journal_open_directory(
                    &mut handle,
                    path_cstr.as_ptr(),
                    0,
                )
            }
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        if handle.is_null() {
            return Err(JournalError::IoError);
        }
        
        Ok(handle)
    }
    
    fn setup_filters(&mut self) -> Result<()> {
        // Add hostname filter: _HOSTNAME=hostname
        let hostname_match = format!("_HOSTNAME={}", self.config.hostname);
        self.add_match(&hostname_match)?;
        
        // Add service filter: _SYSTEMD_UNIT=service
        let service_match = format!("_SYSTEMD_UNIT={}", self.config.service);
        self.add_match(&service_match)?;
        
        Ok(())
    }
    
    fn add_match(&mut self, match_str: &str) -> Result<()> {
        let match_cstr = CString::new(match_str)
            .map_err(|_| JournalError::InvalidArgument)?;
        let match_bytes = match_cstr.as_bytes();
        
        let result = unsafe {
            ffi::sd_journal_add_match(
                self.handle,
                match_bytes.as_ptr() as *const c_void,
                match_bytes.len(),
            )
        };
        
        if result < 0 {
            Err(JournalError::from_errno(result))
        } else {
            Ok(())
        }
    }
    
    fn seek_to_tail(&mut self) -> Result<()> {
        // For live tailing, we want to start from recent entries, not the very end
        // Seek to 10 seconds ago to catch recent entries and then move forward
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        let ten_seconds_ago = now - (10 * 1_000_000); // 10 seconds in microseconds
        
        let result = unsafe { 
            ffi::sd_journal_seek_realtime_usec(self.handle, ten_seconds_ago)
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        // Move to the first entry at or after this time
        let result = unsafe { ffi::sd_journal_next(self.handle) };
        
        if result < 0 {
            Err(JournalError::from_errno(result))
        } else {
            // Position is now set, ready for iteration
            Ok(())
        }
    }
    
    /// Wait for new journal entries using polling approach
    /// 
    /// This uses a simple polling approach with sleep instead of sd_journal_wait()
    /// which hangs indefinitely. The polling interval is configurable via TailConfig.
    fn wait_for_entries_polling(&mut self) -> Result<()> {
        // Use the configured polling interval
        let poll_interval = self.config.poll_interval;
        
        // Simple approach: just sleep and let the caller try again
        // This avoids complex journal position management
        std::thread::sleep(poll_interval);
        Ok(())
    }
    
    /// Get the current journal entry
    fn get_current_entry(&self) -> Result<Entry> {
        // This reuses the existing logic from journal.rs
        // We need to extract the entry data from the current journal position
        
        let hostname = self.get_field_data("_HOSTNAME").ok();
        let unit = self.get_field_data("_SYSTEMD_UNIT").ok();
        let message = self.get_field_data("MESSAGE")
            .unwrap_or_else(|_| String::new());
        
        // Get timestamp
        let mut timestamp: u64 = 0;
        let result = unsafe {
            ffi::sd_journal_get_realtime_usec(self.handle, &mut timestamp)
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        Ok(Entry {
            hostname,
            unit,
            timestamp_utc: timestamp,
            message,
        })
    }
    
    /// Get data for a specific field from the current journal entry
    fn get_field_data(&self, field: &str) -> Result<String> {
        let field_cstr = CString::new(field)
            .map_err(|_| JournalError::InvalidArgument)?;
        
        let mut data: *const c_void = ptr::null();
        let mut length: usize = 0;
        
        let result = unsafe {
            ffi::sd_journal_get_data(
                self.handle,
                field_cstr.as_ptr(),
                &mut data,
                &mut length,
            )
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        if data.is_null() || length == 0 {
            return Err(JournalError::NotFound);
        }
        
        // Convert the data to a string
        let data_slice = unsafe {
            std::slice::from_raw_parts(data as *const u8, length)
        };
        
        let data_str = std::str::from_utf8(data_slice)
            .map_err(|_| JournalError::InvalidData)?;
        
        // The data is in format "FIELD=value", we want just the value
        if let Some(equals_pos) = data_str.find('=') {
            Ok(data_str[equals_pos + 1..].to_string())
        } else {
            Ok(data_str.to_string())
        }
    }
}

impl Drop for JournalTail {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::sd_journal_close(self.handle);
            }
        }
    }
}

/// Iterator over journal entries for live tailing
/// 
/// This iterator will block on each call to `next()` until a new journal entry
/// matching the configured filters becomes available.
pub struct JournalIterator<'a> {
    tail: &'a mut JournalTail,
}

impl<'a> Iterator for JournalIterator<'a> {
    type Item = Result<Entry>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to get the next entry
            let next_result = unsafe {
                ffi::sd_journal_next(self.tail.handle)
            };
            
            match next_result {
                1 => {
                    // Found an entry
                    return Some(self.tail.get_current_entry());
                }
                0 => {
                    // No more entries, wait for new ones using polling approach
                    match self.tail.wait_for_entries_polling() {
                        Ok(()) => {
                            // New entries should be available, continue the loop to get them
                            continue;
                        }
                        Err(e) => {
                            // Error waiting for entries, return it
                            return Some(Err(e));
                        }
                    }
                }
                r if r < 0 => {
                    // Error getting next entry
                    return Some(Err(JournalError::from_errno(r)));
                }
                _ => {
                    // Unexpected result
                    return Some(Err(JournalError::Unknown(next_result)));
                }
            }
        }
    }
}
