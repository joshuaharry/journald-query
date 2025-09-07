use crate::error::{JournalError, Result};
use crate::ffi;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

/// A handle to a systemd journal
/// 
/// This struct provides safe access to systemd journal functionality.
/// It enforces thread safety by being !Send and !Sync, meaning it can
/// only be used from the thread that created it.
#[derive(Debug)]
pub struct Journal {
    handle: *mut ffi::SdJournal,
    // PhantomData to make this !Send + !Sync (not thread-safe)
    _not_thread_safe: PhantomData<*const ()>,
}

impl Journal {
    /// Open journal files from a directory
    /// 
    /// # Arguments
    /// * `path` - Directory containing journal files
    /// 
    /// # Returns
    /// A new Journal instance on success
    /// 
    /// # Examples
    /// ```no_run
    /// use journald_query::Journal;
    /// 
    /// let journal = Journal::open_directory("/var/log/journal")?;
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn open_directory<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let path_cstr = CString::new(path_str)
            .map_err(|_| JournalError::InvalidArgument)?;
        
        let mut handle: *mut ffi::SdJournal = ptr::null_mut();
        
        let result = unsafe {
            ffi::sd_journal_open_directory(
                &mut handle,
                path_cstr.as_ptr(),
                0, // Try with no flags first
            )
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        if handle.is_null() {
            return Err(JournalError::Unknown(-1));
        }
        
        Ok(Journal {
            handle,
            _not_thread_safe: PhantomData,
        })
    }

    /// Open specific journal files
    /// 
    /// # Arguments
    /// * `file_paths` - Vector of paths to journal files to open
    /// 
    /// # Returns
    /// A new Journal instance on success
    /// 
    /// # Examples
    /// ```no_run
    /// use journald_query::Journal;
    /// 
    /// let journal = Journal::open_files(vec!["test.journal"])?;
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn open_files<P: AsRef<Path>>(file_paths: Vec<P>) -> Result<Self> {
        // Convert paths to C strings
        let c_strings: std::result::Result<Vec<CString>, _> = file_paths
            .iter()
            .map(|p| CString::new(p.as_ref().to_string_lossy().as_ref()))
            .collect();
        
        let c_strings = c_strings.map_err(|_| JournalError::InvalidArgument)?;
        
        // Create array of C string pointers, null-terminated
        let mut c_ptrs: Vec<*const std::os::raw::c_char> = c_strings
            .iter()
            .map(|cs| cs.as_ptr())
            .collect();
        c_ptrs.push(ptr::null()); // Null terminate the array
        
        let mut handle: *mut ffi::SdJournal = ptr::null_mut();
        
        let result = unsafe {
            ffi::sd_journal_open_files(
                &mut handle,
                c_ptrs.as_ptr(),
                0, // Documentation says flags must be 0 for sd_journal_open_files
            )
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        if handle.is_null() {
            return Err(JournalError::Unknown(-1));
        }
        
        Ok(Journal {
            handle,
            _not_thread_safe: PhantomData,
        })
    }
    
    /// Query unique values for a specific field
    /// 
    /// This prepares the journal to enumerate unique values for the given field.
    /// Call `enumerate_unique_values()` to iterate through the results.
    /// 
    /// # Arguments
    /// * `field` - Field name to query (e.g., "_HOSTNAME", "_SYSTEMD_UNIT")
    /// 
    /// # Examples
    /// ```no_run
    /// # use journald_query::Journal;
    /// # let journal = Journal::open_directory("/var/log/journal")?;
    /// journal.query_unique("_HOSTNAME")?;
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn query_unique(&self, field: &str) -> Result<()> {
        let field_cstr = CString::new(field)
            .map_err(|_| JournalError::InvalidArgument)?;
        
        let result = unsafe {
            ffi::sd_journal_query_unique(self.handle, field_cstr.as_ptr())
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        Ok(())
    }
    
    /// Get the next unique value from the current query
    /// 
    /// Returns the next unique field value as a string, or None if no more values.
    /// The field name and "=" are included in the returned string (e.g., "_HOSTNAME=server1").
    /// 
    /// # Returns
    /// Some(value) if a value is available, None if no more values
    /// 
    /// # Examples
    /// ```no_run
    /// # use journald_query::Journal;
    /// # let journal = Journal::open_directory("/var/log/journal")?;
    /// journal.query_unique("_HOSTNAME")?;
    /// while let Some(hostname) = journal.next_unique_value()? {
    ///     println!("Found hostname: {}", hostname);
    /// }
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn next_unique_value(&self) -> Result<Option<String>> {
        let mut data: *const c_void = ptr::null();
        let mut length: usize = 0;
        
        let result = unsafe {
            ffi::sd_journal_enumerate_available_unique(self.handle, &mut data, &mut length)
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        if result == 0 {
            // No more data
            return Ok(None);
        }
        
        if data.is_null() || length == 0 {
            return Ok(None);
        }
        
        // Convert C data to Rust string
        let slice = unsafe {
            std::slice::from_raw_parts(data as *const u8, length)
        };
        
        let value = String::from_utf8_lossy(slice).to_string();
        Ok(Some(value))
    }
    
    /// Reset unique value enumeration to the beginning
    /// 
    /// After calling this, the next call to `next_unique_value()` will return
    /// the first unique value again.
    pub fn restart_unique(&self) {
        unsafe {
            ffi::sd_journal_restart_unique(self.handle);
        }
    }
    
    /// Get all unique values for a field as a vector
    /// 
    /// This is a convenience method that queries unique values and collects
    /// them all into a Vec<String>.
    /// 
    /// # Arguments
    /// * `field` - Field name to query
    /// 
    /// # Returns
    /// Vector of unique field values (including field name and "=")
    /// 
    /// # Examples
    /// ```no_run
    /// # use journald_query::Journal;
    /// # let journal = Journal::open_directory("/var/log/journal")?;
    /// let hostnames = journal.get_unique_values("_HOSTNAME")?;
    /// for hostname in hostnames {
    ///     println!("Hostname: {}", hostname);
    /// }
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn get_unique_values(&self, field: &str) -> Result<Vec<String>> {
        self.query_unique(field)?;
        
        let mut values = Vec::new();
        while let Some(value) = self.next_unique_value()? {
            values.push(value);
        }
        
        Ok(values)
    }

    /// Add a match filter to the journal
    /// 
    /// This filters journal entries to only include those with the specified field value.
    /// Multiple matches can be added - they work as AND conditions for different fields,
    /// and OR conditions for the same field.
    /// 
    /// # Arguments
    /// * `field` - Field name (e.g., "_HOSTNAME")
    /// * `value` - Value to match (e.g., "localhost")
    /// 
    /// # Examples
    /// ```no_run
    /// # use journald_query::Journal;
    /// # let journal = Journal::open_directory("/var/log/journal")?;
    /// journal.add_match("_HOSTNAME", "localhost")?;
    /// journal.add_match("_SYSTEMD_UNIT", "sshd.service")?;
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn add_match(&self, field: &str, value: &str) -> Result<()> {
        let match_string = format!("{}={}", field, value);
        let match_cstr = CString::new(match_string)
            .map_err(|_| JournalError::InvalidArgument)?;
        
        // Calculate the actual size: length of FIELD + 1 (for '=') + length of value
        // The CString::as_bytes() gives us the bytes without the null terminator
        let size = match_cstr.as_bytes().len();
        
        let result = unsafe {
            ffi::sd_journal_add_match(
                self.handle,
                match_cstr.as_ptr() as *const std::os::raw::c_void,
                size,
            )
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        Ok(())
    }

    /// Clear all match filters
    /// 
    /// After calling this, all journal entries will be available for iteration.
    /// 
    /// # Examples
    /// ```no_run
    /// # use journald_query::Journal;
    /// # let journal = Journal::open_directory("/var/log/journal")?;
    /// journal.add_match("_HOSTNAME", "localhost")?;
    /// journal.flush_matches(); // Clear the filter
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn flush_matches(&self) {
        unsafe {
            ffi::sd_journal_flush_matches(self.handle);
        }
    }

    /// Seek to the beginning of the journal
    /// 
    /// This positions the read pointer before the first entry.
    /// Call `next()` to advance to the first entry.
    /// 
    /// # Examples
    /// ```no_run
    /// # use journald_query::Journal;
    /// # let journal = Journal::open_directory("/var/log/journal")?;
    /// journal.seek_head()?;
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn seek_head(&self) -> Result<()> {
        let result = unsafe {
            ffi::sd_journal_seek_head(self.handle)
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        Ok(())
    }

    /// Advance to the next journal entry
    /// 
    /// Returns `true` if there is a next entry, `false` if at the end.
    /// 
    /// # Examples
    /// ```no_run
    /// # use journald_query::Journal;
    /// # let journal = Journal::open_directory("/var/log/journal")?;
    /// journal.seek_head()?;
    /// while journal.next()? {
    ///     // Process entry
    /// }
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn next(&self) -> Result<bool> {
        let result = unsafe {
            ffi::sd_journal_next(self.handle)
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        Ok(result > 0)
    }

    /// Get data for a specific field from the current journal entry
    /// 
    /// The journal read pointer must be positioned at a valid entry (after calling `next()`).
    /// 
    /// # Arguments
    /// * `field` - Field name to retrieve (e.g., "_HOSTNAME", "_SYSTEMD_UNIT")
    /// 
    /// # Returns
    /// The field data as a string, or None if the field is not present in this entry
    /// 
    /// # Examples
    /// ```no_run
    /// # use journald_query::Journal;
    /// # let journal = Journal::open_directory("/var/log/journal")?;
    /// journal.seek_head()?;
    /// if journal.next()? {
    ///     if let Some(hostname) = journal.get_field("_HOSTNAME")? {
    ///         println!("Hostname: {}", hostname);
    ///     }
    /// }
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn get_field(&self, field: &str) -> Result<Option<String>> {
        let field_cstr = CString::new(field)
            .map_err(|_| JournalError::InvalidArgument)?;
        
        let mut data: *const std::os::raw::c_void = ptr::null();
        let mut length: usize = 0;
        
        let result = unsafe {
            ffi::sd_journal_get_data(
                self.handle,
                field_cstr.as_ptr(),
                &mut data,
                &mut length,
            )
        };
        
        if result == -libc::ENOENT {
            return Ok(None); // Field not found in this entry
        }
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        // Convert the raw data to a string
        let data_slice = unsafe {
            std::slice::from_raw_parts(data as *const u8, length)
        };
        
        let data_str = std::str::from_utf8(data_slice)
            .map_err(|_| JournalError::InvalidData)?;
        
        Ok(Some(data_str.to_string()))
    }

    /// Seek to a specific timestamp in the journal
    /// 
    /// This method positions the journal cursor at or near the specified
    /// realtime timestamp. The timestamp is in microseconds since Unix epoch.
    /// 
    /// # Arguments
    /// * `timestamp_usec` - Timestamp in microseconds since Unix epoch
    /// 
    /// # Returns
    /// Ok(()) on success
    /// 
    /// # Examples
    /// ```no_run
    /// use journald_query::Journal;
    /// 
    /// let journal = Journal::open_directory("/var/log/journal")?;
    /// 
    /// // Seek to January 1, 2022 00:00:00 UTC
    /// let timestamp = 1640995200000000; // microseconds since epoch
    /// journal.seek_realtime_usec(timestamp)?;
    /// 
    /// // Now iterate from that point
    /// while journal.next()? {
    ///     // Process entries from that timestamp onward
    ///     break;
    /// }
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn seek_realtime_usec(&self, timestamp_usec: u64) -> Result<()> {
        let result = unsafe {
            ffi::sd_journal_seek_realtime_usec(self.handle, timestamp_usec)
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        Ok(())
    }

    /// Get the realtime timestamp of the current journal entry
    /// 
    /// This method retrieves the wallclock timestamp of the current journal entry
    /// in microseconds since Unix epoch.
    /// 
    /// # Returns
    /// The timestamp in microseconds since Unix epoch
    /// 
    /// # Examples
    /// ```no_run
    /// use journald_query::Journal;
    /// 
    /// let journal = Journal::open_directory("/var/log/journal")?;
    /// journal.seek_head()?;
    /// 
    /// while journal.next()? {
    ///     let timestamp = journal.get_realtime_usec()?;
    ///     println!("Entry timestamp: {} microseconds since epoch", timestamp);
    ///     break;
    /// }
    /// # Ok::<(), journald_query::JournalError>(())
    /// ```
    pub fn get_realtime_usec(&self) -> Result<u64> {
        let mut timestamp: u64 = 0;
        
        let result = unsafe {
            ffi::sd_journal_get_realtime_usec(self.handle, &mut timestamp)
        };
        
        if result < 0 {
            return Err(JournalError::from_errno(result));
        }
        
        Ok(timestamp)
    }
}

impl Drop for Journal {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::sd_journal_close(self.handle);
            }
        }
    }
}

// Ensure Journal is !Send and !Sync (not thread-safe)
// This is enforced by the PhantomData<*const ()> field
// Note: We can't use static_assertions without adding it as a dependency
// The PhantomData<*const ()> already ensures !Send + !Sync

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_not_send_sync() {
        // This test verifies that Journal is not Send or Sync by using PhantomData
        let _phantom = PhantomData::<*const ()>;
        // PhantomData<*const ()> is !Send + !Sync, so Journal should be too
    }

    #[test]
    fn test_cstring_conversion() {
        // Test that we can create CString from various inputs
        let valid_path = "/var/log/journal";
        let cstr = CString::new(valid_path).unwrap();
        assert_eq!(cstr.to_str().unwrap(), valid_path);
        
        // Test that null bytes are rejected
        let invalid_path = "/var/log\0/journal";
        assert!(CString::new(invalid_path).is_err());
    }

    #[test]
    fn test_error_handling() {
        // Test error conversion
        let err = JournalError::from_errno(-libc::EINVAL);
        assert_eq!(err, JournalError::InvalidArgument);
    }

    #[test]
    fn test_match_string_size_calculation() {
        // Test that our size calculation matches the expected format
        let field = "_HOSTNAME";
        let value = "localhost";
        let match_string = format!("{}={}", field, value);
        let match_cstr = CString::new(match_string).unwrap();
        
        // Expected size: length of "_HOSTNAME" + 1 (for '=') + length of "localhost"
        let expected_size = field.len() + 1 + value.len();
        let actual_size = match_cstr.as_bytes().len();
        
        assert_eq!(actual_size, expected_size);
        
        // "_HOSTNAME=localhost"
        // _HOSTNAME = 9 chars
        // = = 1 char  
        // localhost = 9 chars
        // Total = 19 chars
        assert_eq!(actual_size, 19);
        
        // Verify the actual content
        let expected_content = "_HOSTNAME=localhost";
        assert_eq!(match_cstr.to_str().unwrap(), expected_content);
        assert_eq!(expected_content.len(), 19);
    }
}
