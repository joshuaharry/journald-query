use crate::error::{JournalError, Result};
use crate::ffi::{self, wait_result};
use crate::query::Entry;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;

/// Configuration for tailing journal entries from a specific service
#[derive(Debug, Clone, PartialEq)]
pub struct TailConfig {
    /// Hostname to filter by (_HOSTNAME field)
    pub hostname: String,
    /// Service/unit name to filter by (_SYSTEMD_UNIT field)
    pub service: String,
    /// Path to journal directory 
    pub journal_path: String,
}

impl TailConfig {
    /// Create a new tail configuration
    /// 
    /// # Arguments
    /// * `hostname` - The hostname to filter journal entries by
    /// * `service` - The systemd service/unit name to filter by
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
        }
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
        // First seek to recent time (last 10 seconds) to avoid missing entries
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
        
        // Now seek to tail to get to the end
        let result = unsafe { ffi::sd_journal_seek_tail(self.handle) };
        
        if result < 0 {
            Err(JournalError::from_errno(result))
        } else {
            Ok(())
        }
    }
    
    /// Wait for new journal entries (blocking)
    /// 
    /// This will block until new entries are available or an error occurs.
    /// Uses infinite timeout to wait indefinitely.
    fn wait_for_entries(&mut self) -> Result<()> {
        let result = unsafe {
            ffi::sd_journal_wait(self.handle, u64::MAX) // Infinite timeout
        };
        
        match result {
            r if r == wait_result::SD_JOURNAL_APPEND => Ok(()),
            r if r == wait_result::SD_JOURNAL_INVALIDATE => {
                // Journal was rotated/invalidated, but we can continue
                Ok(())
            }
            r if r == wait_result::SD_JOURNAL_NOP => {
                // No changes, but this shouldn't happen with infinite timeout
                // Try again
                self.wait_for_entries()
            }
            r if r < 0 => Err(JournalError::from_errno(r)),
            _ => Err(JournalError::Unknown(result)),
        }
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
                    // No more entries, wait for new ones
                    match self.tail.wait_for_entries() {
                        Ok(()) => {
                            // New entries available, continue the loop to get them
                            continue;
                        }
                        Err(e) => {
                            // Error waiting, return it
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
