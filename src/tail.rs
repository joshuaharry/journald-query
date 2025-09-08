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
    /// How far back in time to start reading entries (default: 10 seconds ago)
    pub start_time_offset: Duration,
}

impl TailConfig {
    /// Create a new tail configuration with default settings
    /// 
    /// Defaults:
    /// - Polling interval: 100ms
    /// - Start time offset: 10 seconds ago
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
            start_time_offset: Duration::from_secs(10), // Default 10 seconds ago
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

    /// Set how far back in time to start reading entries
    /// 
    /// # Arguments
    /// * `offset` - Duration to go back in time from now
    /// 
    /// # Examples
    /// ```
    /// use journald_query::tail::TailConfig;
    /// use std::time::Duration;
    /// 
    /// // Start from 5 minutes ago
    /// let config = TailConfig::new("web-server-01", "nginx.service", "/var/log/journal")
    ///     .with_start_time_offset(Duration::from_secs(300));
    /// 
    /// // Start from 1 hour ago
    /// let config = TailConfig::new("web-server-01", "nginx.service", "/var/log/journal")
    ///     .with_start_time_offset(Duration::from_secs(3600));
    /// ```
    pub fn with_start_time_offset(mut self, offset: Duration) -> Self {
        self.start_time_offset = offset;
        self
    }

    /// Set start time offset in seconds (convenience method)
    /// 
    /// # Arguments
    /// * `seconds` - Number of seconds to go back in time from now
    /// 
    /// # Examples
    /// ```
    /// use journald_query::tail::TailConfig;
    /// 
    /// // Start from 30 seconds ago
    /// let config = TailConfig::new("web-server-01", "nginx.service", "/var/log/journal")
    ///     .with_start_time_offset_secs(30);
    /// 
    /// // Start from 5 minutes ago
    /// let config = TailConfig::new("web-server-01", "nginx.service", "/var/log/journal")
    ///     .with_start_time_offset_secs(300);
    /// ```
    pub fn with_start_time_offset_secs(mut self, seconds: u64) -> Self {
        self.start_time_offset = Duration::from_secs(seconds);
        self
    }

    /// Start tailing from now (no historical entries)
    /// 
    /// This is equivalent to `with_start_time_offset(Duration::ZERO)` but more explicit.
    /// 
    /// # Examples
    /// ```
    /// use journald_query::tail::TailConfig;
    /// 
    /// // Only show new entries from this moment forward
    /// let config = TailConfig::new("web-server-01", "nginx.service", "/var/log/journal")
    ///     .from_now();
    /// ```
    pub fn from_now(mut self) -> Self {
        self.start_time_offset = Duration::ZERO;
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
    /// let mut tail = JournalTail::new(config)?;
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
        // For live tailing, we want to start from configurable time offset
        // Use the configured start_time_offset to determine how far back to go
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        // Convert the offset to microseconds and subtract from now
        let offset_micros = self.config.start_time_offset.as_micros() as u64;
        let start_time = now.saturating_sub(offset_micros);
        
        let result = unsafe { 
            ffi::sd_journal_seek_realtime_usec(self.handle, start_time)
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
    /// You might be tempted to use sd_journal_wait() here. I would recommend against that
    /// for two reasons:
    /// 1. It only captures changes every 250ms - see:
    /// https://github.com/systemd/systemd/issues/17574
    /// 2. It can hang indefinitely for reasons I don't completely understand.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_tail_config_defaults() {
        let config = TailConfig::new("test-host", "test.service", "/test/path");
        
        assert_eq!(config.hostname, "test-host");
        assert_eq!(config.service, "test.service");
        assert_eq!(config.journal_path, "/test/path");
        assert_eq!(config.poll_interval, Duration::from_millis(100));
        assert_eq!(config.start_time_offset, Duration::from_secs(10));
    }

    #[test]
    fn test_tail_config_with_poll_interval() {
        let config = TailConfig::new("host", "service", "/path")
            .with_poll_interval(Duration::from_millis(250));
        
        assert_eq!(config.poll_interval, Duration::from_millis(250));
        // Other fields should remain unchanged
        assert_eq!(config.start_time_offset, Duration::from_secs(10));
    }

    #[test]
    fn test_tail_config_with_poll_interval_ms() {
        let config = TailConfig::new("host", "service", "/path")
            .with_poll_interval_ms(500);
        
        assert_eq!(config.poll_interval, Duration::from_millis(500));
    }

    #[test]
    fn test_tail_config_with_start_time_offset() {
        let config = TailConfig::new("host", "service", "/path")
            .with_start_time_offset(Duration::from_secs(60));
        
        assert_eq!(config.start_time_offset, Duration::from_secs(60));
        // Other fields should remain unchanged
        assert_eq!(config.poll_interval, Duration::from_millis(100));
    }

    #[test]
    fn test_tail_config_with_start_time_offset_secs() {
        let config = TailConfig::new("host", "service", "/path")
            .with_start_time_offset_secs(300);
        
        assert_eq!(config.start_time_offset, Duration::from_secs(300));
    }

    #[test]
    fn test_tail_config_from_now() {
        let config = TailConfig::new("host", "service", "/path")
            .from_now();
        
        assert_eq!(config.start_time_offset, Duration::ZERO);
    }

    #[test]
    fn test_tail_config_method_chaining() {
        let config = TailConfig::new("web-server", "nginx.service", "/var/log/journal")
            .with_poll_interval_ms(50)
            .with_start_time_offset_secs(30);
        
        assert_eq!(config.hostname, "web-server");
        assert_eq!(config.service, "nginx.service");
        assert_eq!(config.journal_path, "/var/log/journal");
        assert_eq!(config.poll_interval, Duration::from_millis(50));
        assert_eq!(config.start_time_offset, Duration::from_secs(30));
    }

    #[test]
    fn test_tail_config_chaining_order_independence() {
        let config1 = TailConfig::new("host", "service", "/path")
            .with_poll_interval_ms(200)
            .with_start_time_offset_secs(60);
        
        let config2 = TailConfig::new("host", "service", "/path")
            .with_start_time_offset_secs(60)
            .with_poll_interval_ms(200);
        
        assert_eq!(config1.poll_interval, config2.poll_interval);
        assert_eq!(config1.start_time_offset, config2.start_time_offset);
    }

    #[test]
    fn test_tail_config_overriding_values() {
        let config = TailConfig::new("host", "service", "/path")
            .with_poll_interval_ms(100)
            .with_poll_interval_ms(200) // Override previous value
            .with_start_time_offset_secs(10)
            .with_start_time_offset_secs(20); // Override previous value
        
        assert_eq!(config.poll_interval, Duration::from_millis(200));
        assert_eq!(config.start_time_offset, Duration::from_secs(20));
    }

    #[test]
    fn test_tail_config_extreme_values() {
        // Test very small values
        let config_small = TailConfig::new("host", "service", "/path")
            .with_poll_interval_ms(1)
            .with_start_time_offset_secs(0);
        
        assert_eq!(config_small.poll_interval, Duration::from_millis(1));
        assert_eq!(config_small.start_time_offset, Duration::ZERO);
        
        // Test large values
        let config_large = TailConfig::new("host", "service", "/path")
            .with_poll_interval_ms(60000) // 1 minute
            .with_start_time_offset_secs(3600); // 1 hour
        
        assert_eq!(config_large.poll_interval, Duration::from_secs(60));
        assert_eq!(config_large.start_time_offset, Duration::from_secs(3600));
    }

    #[test]
    fn test_tail_config_string_conversions() {
        // Test that Into<String> works for all string parameters
        let config = TailConfig::new(
            "hostname".to_string(),
            "service.service".to_string(), 
            "/path/to/journal".to_string()
        );
        
        assert_eq!(config.hostname, "hostname");
        assert_eq!(config.service, "service.service");
        assert_eq!(config.journal_path, "/path/to/journal");
    }

    #[test]
    fn test_tail_config_clone_and_debug() {
        let config = TailConfig::new("host", "service", "/path")
            .with_poll_interval_ms(150)
            .with_start_time_offset_secs(45);
        
        // Test Clone
        let cloned = config.clone();
        assert_eq!(config, cloned);
        
        // Test Debug
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("host"));
        assert!(debug_str.contains("service"));
        assert!(debug_str.contains("/path"));
        assert!(debug_str.contains("150ms"));
        assert!(debug_str.contains("45s"));
    }

    #[test]
    fn test_tail_config_use_cases() {
        // Test common use case configurations
        
        // High-frequency monitoring (dashboards)
        let dashboard_config = TailConfig::new("web-server", "nginx.service", "/var/log/journal")
            .with_poll_interval_ms(50)
            .with_start_time_offset_secs(60);
        
        assert_eq!(dashboard_config.poll_interval, Duration::from_millis(50));
        assert_eq!(dashboard_config.start_time_offset, Duration::from_secs(60));
        
        // Low-frequency monitoring (background services)
        let background_config = TailConfig::new("db-server", "postgres.service", "/var/log/journal")
            .with_poll_interval_ms(500)
            .with_start_time_offset_secs(300);
        
        assert_eq!(background_config.poll_interval, Duration::from_millis(500));
        assert_eq!(background_config.start_time_offset, Duration::from_secs(300));
        
        // Real-time only (alerting)
        let realtime_config = TailConfig::new("alert-server", "monitor.service", "/var/log/journal")
            .from_now()
            .with_poll_interval_ms(25);
        
        assert_eq!(realtime_config.poll_interval, Duration::from_millis(25));
        assert_eq!(realtime_config.start_time_offset, Duration::ZERO);
        
        // Investigation mode (long history)
        let investigation_config = TailConfig::new("problem-server", "failing.service", "/var/log/journal")
            .with_start_time_offset_secs(3600) // 1 hour ago
            .with_poll_interval_ms(100);
        
        assert_eq!(investigation_config.start_time_offset, Duration::from_secs(3600));
        assert_eq!(investigation_config.poll_interval, Duration::from_millis(100));
    }
}
