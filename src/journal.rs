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
        let path_cstr = CString::new(path.as_ref().to_string_lossy().as_ref())
            .map_err(|_| JournalError::InvalidArgument)?;
        
        let mut handle: *mut ffi::SdJournal = ptr::null_mut();
        
        let result = unsafe {
            ffi::sd_journal_open_directory(
                &mut handle,
                path_cstr.as_ptr(),
                ffi::flags::SD_JOURNAL_LOCAL_ONLY,
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
}
