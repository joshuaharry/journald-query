use std::os::raw::{c_char, c_int, c_void};

/// Opaque handle to a systemd journal
#[repr(C)]
pub struct SdJournal {
    _private: [u8; 0],
}

// FFI bindings to systemd journal functions
unsafe extern "C" {
    pub fn sd_journal_open_directory(
        ret: *mut *mut SdJournal,
        path: *const c_char,
        flags: c_int,
    ) -> c_int;

    pub fn sd_journal_close(j: *mut SdJournal);

    pub fn sd_journal_query_unique(j: *mut SdJournal, field: *const c_char) -> c_int;

    pub fn sd_journal_enumerate_available_unique(
        j: *mut SdJournal,
        data: *mut *const c_void,
        length: *mut usize,
    ) -> c_int;

    pub fn sd_journal_restart_unique(j: *mut SdJournal);
}

/// Journal open flags
pub mod flags {
    use std::os::raw::c_int;

    /// Only include files generated on the local machine
    pub const SD_JOURNAL_LOCAL_ONLY: c_int = 1;
    /// Only include volatile journal files
    #[allow(dead_code)]
    pub const SD_JOURNAL_RUNTIME_ONLY: c_int = 2;
    /// Include system journal files
    #[allow(dead_code)]
    pub const SD_JOURNAL_SYSTEM: c_int = 4;
    /// Include current user journal files
    #[allow(dead_code)]
    pub const SD_JOURNAL_CURRENT_USER: c_int = 8;
    /// Treat path as OS root
    #[allow(dead_code)]
    pub const SD_JOURNAL_OS_ROOT: c_int = 16;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    // These tests verify the FFI bindings compile and have correct signatures
    // They don't actually call the functions since we need libsystemd linked

    #[test]
    fn test_ffi_signatures() {
        // This test ensures our FFI signatures are correct by attempting to use them
        // in ways that would fail compilation if the types were wrong
        
        let mut journal_ptr: *mut SdJournal = ptr::null_mut();
        let journal_ptr_ptr: *mut *mut SdJournal = &mut journal_ptr;
        let path_ptr: *const c_char = ptr::null();
        let flags: c_int = 0;
        
        // These function pointers verify the signatures are correct
        let _open_fn: unsafe extern "C" fn(*mut *mut SdJournal, *const c_char, c_int) -> c_int = 
            sd_journal_open_directory;
        let _close_fn: unsafe extern "C" fn(*mut SdJournal) = sd_journal_close;
        let _query_fn: unsafe extern "C" fn(*mut SdJournal, *const c_char) -> c_int = 
            sd_journal_query_unique;
        let _enumerate_fn: unsafe extern "C" fn(*mut SdJournal, *mut *const c_void, *mut usize) -> c_int = 
            sd_journal_enumerate_available_unique;
        let _restart_fn: unsafe extern "C" fn(*mut SdJournal) = sd_journal_restart_unique;

        // Verify we can create the types we need
        let _: *mut *mut SdJournal = journal_ptr_ptr;
        let _: *const c_char = path_ptr;
        let _: c_int = flags;
    }

    #[test]
    fn test_flags() {
        // Verify flags have expected values (these are from systemd source)
        assert_eq!(flags::SD_JOURNAL_LOCAL_ONLY, 1);
        assert_eq!(flags::SD_JOURNAL_RUNTIME_ONLY, 2);
        assert_eq!(flags::SD_JOURNAL_SYSTEM, 4);
        assert_eq!(flags::SD_JOURNAL_CURRENT_USER, 8);
        assert_eq!(flags::SD_JOURNAL_OS_ROOT, 16);
    }
}
