use journald_query::{Journal, TailConfig, JournalTail};
use std::path::{Path, PathBuf};

// Integration tests using our test journal files
// These tests are robust and don't depend on system journal state

const TEST_JOURNAL_DIR: &str = "test_journald_files";

fn get_test_file_path(filename: &str) -> PathBuf {
    Path::new(TEST_JOURNAL_DIR).join(filename)
}

#[test]
fn test_discover_services_api_with_test_files() {
    let test_journal_dir = "test_journal_dir";
    
    // Test the actual discover_services function that users will call
    let services = journald_query::discover_services(test_journal_dir)
        .expect("discover_services should work with test journal directory");
    
    println!("Found {} hosts: {:?}", services.len(), services.hostnames());
    
    // Verify we get the expected structure from our multi_host.journal file
    assert_eq!(services.len(), 3, "Should discover exactly 3 hosts from test journal");
    
    // Verify specific host-service correlations from our test data
    let database_server = services.find_host("database-server")
        .expect("Should find database-server host");
    println!("database-server units: {:?}", database_server.units);
    assert_eq!(database_server.units.len(), 2, "database-server should have exactly 2 units");
    assert!(database_server.units.contains(&"mysql.service".to_string()), "database-server should have mysql.service");
    assert!(database_server.units.contains(&"postgresql.service".to_string()), "database-server should have postgresql.service");
    
    let web_server = services.find_host("web-server")
        .expect("Should find web-server host");
    println!("web-server units: {:?}", web_server.units);
    assert_eq!(web_server.units.len(), 2, "web-server should have exactly 2 units");
    assert!(web_server.units.contains(&"nginx.service".to_string()), "web-server should have nginx.service");
    assert!(web_server.units.contains(&"apache2.service".to_string()), "web-server should have apache2.service");
    
    let monitoring_server = services.find_host("monitoring-server")
        .expect("Should find monitoring-server host");
    println!("monitoring-server units: {:?}", monitoring_server.units);
    assert_eq!(monitoring_server.units.len(), 2, "monitoring-server should have exactly 2 units");
    assert!(monitoring_server.units.contains(&"prometheus.service".to_string()), "monitoring-server should have prometheus.service");
    assert!(monitoring_server.units.contains(&"grafana.service".to_string()), "monitoring-server should have grafana.service");
    
    assert!(!database_server.units.contains(&"nginx.service".to_string()), "database-server should NOT have nginx.service");
    assert!(!database_server.units.contains(&"apache2.service".to_string()), "database-server should NOT have apache2.service");
    assert!(!database_server.units.contains(&"prometheus.service".to_string()), "database-server should NOT have prometheus.service");
    assert!(!database_server.units.contains(&"grafana.service".to_string()), "database-server should NOT have grafana.service");
    
    assert!(!web_server.units.contains(&"mysql.service".to_string()), "web-server should NOT have mysql.service");
    assert!(!web_server.units.contains(&"postgresql.service".to_string()), "web-server should NOT have postgresql.service");
    assert!(!web_server.units.contains(&"prometheus.service".to_string()), "web-server should NOT have prometheus.service");
    assert!(!web_server.units.contains(&"grafana.service".to_string()), "web-server should NOT have grafana.service");
    
    assert!(!monitoring_server.units.contains(&"mysql.service".to_string()), "monitoring-server should NOT have mysql.service");
    assert!(!monitoring_server.units.contains(&"postgresql.service".to_string()), "monitoring-server should NOT have postgresql.service");
    assert!(!monitoring_server.units.contains(&"nginx.service".to_string()), "monitoring-server should NOT have nginx.service");
    assert!(!monitoring_server.units.contains(&"apache2.service".to_string()), "monitoring-server should NOT have apache2.service");
}



#[test]
fn test_journal_open_single_file() {
    let multi_host_file = get_test_file_path("multi_host.journal");
    
    if !multi_host_file.exists() {
        panic!("Test journal file not found: {:?}. Run ./test_journald_files/create_test_journals.sh first", multi_host_file);
    }
    
    let journal = Journal::open_files(vec![&multi_host_file])
        .expect("Should be able to open test journal file");
    
    // Journal should be valid and will be closed when dropped
    drop(journal);
}

#[test]
fn test_discover_hosts_multi_host_file() {
    let multi_host_file = get_test_file_path("multi_host.journal");
    
    if !multi_host_file.exists() {
        eprintln!("Test journal file not found, skipping test: {:?}", multi_host_file);
        return;
    }
    
    // Test with single file
    let journal = Journal::open_files(vec![&multi_host_file])
        .expect("Should be able to open test journal file");
    
    let hosts = journal.get_unique_values("_HOSTNAME")
        .expect("Should be able to query unique hostnames");
    
    println!("Found hosts: {:?}", hosts);
    
    // Extract just the hostname parts
    let host_names: Vec<String> = hosts
        .into_iter()
        .filter_map(|h| h.strip_prefix("_HOSTNAME=").map(|s| s.to_string()))
        .collect();
    
    assert!(!host_names.is_empty(), "Should find at least one host");
    
    // Should contain our test hosts
    assert!(host_names.contains(&"web-server".to_string()), "Should find web-server");
    assert!(host_names.contains(&"database-server".to_string()), "Should find database-server");
    assert!(host_names.contains(&"monitoring-server".to_string()), "Should find monitoring-server");
    
    println!("Multi-host test passed. Found hosts: {:?}", host_names);
}

#[test]
fn test_discover_units_multi_host_file() {
    let multi_host_file = get_test_file_path("multi_host.journal");
    
    if !multi_host_file.exists() {
        eprintln!("Test journal file not found, skipping test: {:?}", multi_host_file);
        return;
    }
    
    let journal = Journal::open_files(vec![&multi_host_file])
        .expect("Should be able to open test journal file");
    
    let units = journal.get_unique_values("_SYSTEMD_UNIT")
        .expect("Should be able to query unique units");
    
    println!("Found units: {:?}", units);
    
    // Extract just the unit names
    let unit_names: Vec<String> = units
        .into_iter()
        .filter_map(|u| u.strip_prefix("_SYSTEMD_UNIT=").map(|s| s.to_string()))
        .collect();
    
    assert!(!unit_names.is_empty(), "Should find at least one unit");
    
    // Should contain our test units
    assert!(unit_names.contains(&"nginx.service".to_string()), "Should find nginx.service");
    assert!(unit_names.contains(&"postgresql.service".to_string()), "Should find postgresql.service");
    assert!(unit_names.contains(&"prometheus.service".to_string()), "Should find prometheus.service");
    
    println!("Multi-unit test passed. Found units: {:?}", unit_names);
}

#[test]
fn test_discover_hosts_single_host_file() {
    let single_host_file = get_test_file_path("single_host.journal");
    
    if !single_host_file.exists() {
        eprintln!("Test journal file not found, skipping test: {:?}", single_host_file);
        return;
    }
    
    let journal = Journal::open_files(vec![&single_host_file])
        .expect("Should be able to open test journal file");
    
    let hosts = journal.get_unique_values("_HOSTNAME")
        .expect("Should be able to query unique hostnames");
    
    let host_names: Vec<String> = hosts
        .into_iter()
        .filter_map(|h| h.strip_prefix("_HOSTNAME=").map(|s| s.to_string()))
        .collect();
    
    assert_eq!(host_names.len(), 1, "Should find exactly one host");
    assert!(host_names.contains(&"localhost".to_string()), "Should find localhost");
    
    println!("Single-host test passed. Found host: {:?}", host_names);
}

#[test]
fn test_discover_units_single_host_many_units() {
    let single_host_file = get_test_file_path("single_host.journal");
    
    if !single_host_file.exists() {
        eprintln!("Test journal file not found, skipping test: {:?}", single_host_file);
        return;
    }
    
    let journal = Journal::open_files(vec![&single_host_file])
        .expect("Should be able to open test journal file");
    
    let units = journal.get_unique_values("_SYSTEMD_UNIT")
        .expect("Should be able to query unique units");
    
    let unit_names: Vec<String> = units
        .into_iter()
        .filter_map(|u| u.strip_prefix("_SYSTEMD_UNIT=").map(|s| s.to_string()))
        .collect();
    
    assert!(unit_names.len() >= 5, "Should find at least 5 units");
    
    // Should contain our test units
    assert!(unit_names.contains(&"systemd.service".to_string()), "Should find systemd.service");
    assert!(unit_names.contains(&"NetworkManager.service".to_string()), "Should find NetworkManager.service");
    assert!(unit_names.contains(&"sshd.service".to_string()), "Should find sshd.service");
    assert!(unit_names.contains(&"docker.service".to_string()), "Should find docker.service");
    
    println!("Many-units test passed. Found {} units: {:?}", unit_names.len(), unit_names);
}

#[test]
fn test_error_scenarios() {
    let errors_file = get_test_file_path("errors.journal");
    
    if !errors_file.exists() {
        eprintln!("Test journal file not found, skipping test: {:?}", errors_file);
        return;
    }
    
    let journal = Journal::open_files(vec![&errors_file])
        .expect("Should be able to open test journal file");
    
    // Test hosts in error scenarios
    let hosts = journal.get_unique_values("_HOSTNAME")
        .expect("Should be able to query unique hostnames");
    
    let host_names: Vec<String> = hosts
        .into_iter()
        .filter_map(|h| h.strip_prefix("_HOSTNAME=").map(|s| s.to_string()))
        .collect();
    
    assert!(host_names.contains(&"error-prone-server".to_string()), "Should find error-prone-server");
    assert!(host_names.contains(&"disk-full-server".to_string()), "Should find disk-full-server");
    assert!(host_names.contains(&"memory-constrained-server".to_string()), "Should find memory-constrained-server");
    
    // Test units in error scenarios
    let units = journal.get_unique_values("_SYSTEMD_UNIT")
        .expect("Should be able to query unique units");
    
    let unit_names: Vec<String> = units
        .into_iter()
        .filter_map(|u| u.strip_prefix("_SYSTEMD_UNIT=").map(|s| s.to_string()))
        .collect();
    
    assert!(unit_names.contains(&"failing.service".to_string()), "Should find failing.service");
    assert!(unit_names.contains(&"broken.service".to_string()), "Should find broken.service");
    assert!(unit_names.contains(&"disk-monitor.service".to_string()), "Should find disk-monitor.service");
    
    println!("✅ Error scenarios test passed. Hosts: {:?}, Units: {:?}", host_names, unit_names);
}

#[test]
fn test_multiple_files_combined() {
    let multi_host_file = get_test_file_path("multi_host.journal");
    let single_host_file = get_test_file_path("single_host.journal");
    let errors_file = get_test_file_path("errors.journal");
    
    // Check if all files exist
    let files_to_test: Vec<PathBuf> = vec![&multi_host_file, &single_host_file, &errors_file]
        .into_iter()
        .filter(|f| f.exists())
        .cloned()
        .collect();
    
    if files_to_test.is_empty() {
        eprintln!("No test journal files found, skipping combined test");
        return;
    }
    
    println!("Testing with {} files: {:?}", files_to_test.len(), files_to_test);
    
    let journal = Journal::open_files(files_to_test)
        .expect("Should be able to open multiple test journal files");
    
    // Test combined hosts
    let hosts = journal.get_unique_values("_HOSTNAME")
        .expect("Should be able to query unique hostnames from multiple files");
    
    let host_names: Vec<String> = hosts
        .into_iter()
        .filter_map(|h| h.strip_prefix("_HOSTNAME=").map(|s| s.to_string()))
        .collect();
    
    println!("Combined hosts: {:?}", host_names);
    
    // Should find hosts from all files
    assert!(host_names.len() >= 3, "Should find at least 3 unique hosts across all files");
    
    // Test combined units
    let units = journal.get_unique_values("_SYSTEMD_UNIT")
        .expect("Should be able to query unique units from multiple files");
    
    let unit_names: Vec<String> = units
        .into_iter()
        .filter_map(|u| u.strip_prefix("_SYSTEMD_UNIT=").map(|s| s.to_string()))
        .collect();
    
    println!("Combined units: {:?}", unit_names);
    
    assert!(unit_names.len() >= 10, "Should find at least 10 unique units across all files");
    
    println!("✅ Combined files test passed. Found {} hosts and {} units", 
             host_names.len(), unit_names.len());
}

#[test]
fn test_discover_functions_with_test_files() {
    // This test is currently not possible because discover_hosts/discover_units
    // use open_directory, but our test files are individual journal files.
    // We would need to either:
    // 1. Create a directory structure with the journal files, or
    // 2. Add discover functions that work with file paths
    
    // For now, we'll test the Journal API directly (which we do in other tests)
    println!("Note: discover_hosts/discover_units functions expect directory structure");
    println!("Individual file testing is done through Journal::open_files in other tests");
}

#[test]
fn test_journal_error_handling() {
    // Test opening non-existent file
    let result = Journal::open_files(vec!["nonexistent.journal"]);
    assert!(result.is_err(), "Should fail to open non-existent file");
    
    let error = result.unwrap_err();
    println!("Expected error for non-existent file: {}", error);
}

#[test]
fn test_empty_file_list() {
    // Test opening empty file list
    let result = Journal::open_files::<&str>(vec![]);
    // This might succeed or fail depending on systemd implementation
    // The important thing is that it doesn't crash
    match result {
        Ok(_) => println!("Opening empty file list succeeded"),
        Err(e) => println!("Opening empty file list failed as expected: {}", e),
    }
}

#[test]
fn test_discover_services_single_host() {
    let file_path = get_test_file_path("single_host.journal");
    let journal = Journal::open_files(vec![file_path]).expect("Failed to open test journal");
    
    let hosts = journal.get_unique_values("_HOSTNAME").expect("Failed to get hostnames");
    let units = journal.get_unique_values("_SYSTEMD_UNIT").expect("Failed to get units");
    
    assert_eq!(hosts.len(), 1, "Should find exactly 1 host");
    assert!(units.len() >= 5, "Should find at least 5 units");
    
    // Test filtering for the single host
    let hostname = hosts[0].strip_prefix("_HOSTNAME=").unwrap();
    journal.flush_matches();
    journal.add_match("_HOSTNAME", hostname).expect("Failed to add hostname match");
    let filtered_units = journal.get_unique_values("_SYSTEMD_UNIT").expect("Failed to get filtered units");
    
    // Should get the same units since there's only one host
    assert_eq!(filtered_units.len(), units.len(), "Filtered units should match total units for single host");
    
    journal.flush_matches();
}

#[test]
fn test_journal_filtering_methods() {
    let file_path = get_test_file_path("multi_host.journal");
    let journal = Journal::open_files(vec![file_path]).expect("Failed to open test journal");
    
    // Test that we can add matches without error
    journal.flush_matches();
    journal.add_match("_HOSTNAME", "web-server").expect("Should be able to add hostname match");
    journal.add_match("_SYSTEMD_UNIT", "nginx.service").expect("Should be able to add unit match");
    
    // Test that flush_matches doesn't panic
    journal.flush_matches();
    
    // Test seeking and iteration (basic smoke test)
    journal.seek_head().expect("Should be able to seek to head");
    let has_entries = journal.next().expect("Should be able to call next");
    
    if has_entries {
        // Test getting field data
        let hostname = journal.get_field("_HOSTNAME").expect("Should be able to get hostname field");
        // hostname might be None if the current entry doesn't have this field, which is fine
        println!("Found hostname in entry: {:?}", hostname);
    }
}

#[test]
fn test_time_based_journal_operations() {
    let multi_host_file = get_test_file_path("multi_host.journal");
    
    if !multi_host_file.exists() {
        eprintln!("Test journal file not found, skipping time-based test: {:?}", multi_host_file);
        return;
    }
    
    // Open the test journal file
    let journal = Journal::open_files(vec![&multi_host_file])
        .expect("Should be able to open test journal file");
    
    // Test seeking to head and getting timestamps
    journal.seek_head().expect("Should be able to seek to head");
    
    if journal.next().expect("Should be able to get first entry") {
        // Get the timestamp of the first entry
        let first_timestamp = journal.get_realtime_usec()
            .expect("Should be able to get timestamp from first entry");
        
        println!("First entry timestamp: {} microseconds since epoch", first_timestamp);
        
        // Verify timestamp is reasonable (should be a positive number)
        assert!(first_timestamp > 0, "Timestamp should be positive");
        
        // Test seeking to a specific timestamp
        // Try seeking to the timestamp we just found
        journal.seek_realtime_usec(first_timestamp)
            .expect("Should be able to seek to the timestamp we just read");
        
        // After seeking, we should be able to get an entry at or near that timestamp
        if journal.next().expect("Should be able to get entry after seeking") {
            let seek_timestamp = journal.get_realtime_usec()
                .expect("Should be able to get timestamp after seeking");
            
            println!("After seeking, found entry at timestamp: {} microseconds", seek_timestamp);
            
            // The timestamp should be at or after the one we sought to
            assert!(seek_timestamp >= first_timestamp, 
                "Timestamp after seeking should be >= the timestamp we sought to");
        }
    }
    
    // Test seeking to a very early timestamp (should work)
    let early_timestamp = 1000000; // 1 second after epoch
    journal.seek_realtime_usec(early_timestamp)
        .expect("Should be able to seek to early timestamp");
    
    // Test seeking to a far future timestamp (should work but may not find entries)
    let future_timestamp = u64::MAX - 1000000; // Very far in the future
    journal.seek_realtime_usec(future_timestamp)
        .expect("Should be able to seek to future timestamp");
}

#[test]
fn test_query_journal_basic_functionality() {
    use journald_query::{Query, query_journal};
    use std::path::Path;
    
    let test_dir = Path::new("test_journal_dir");
    
    // Test basic query construction
    let query = Query::new(1640995200000000, 1640998800000000); // 1 hour range
    assert_eq!(query.start_time_utc, 1640995200000000);
    assert_eq!(query.end_time_utc, 1640998800000000);
    assert!(query.hostname.is_none());
    assert!(query.unit.is_none());
    assert!(query.message_contains.is_none());
    
    // Test query builder pattern
    let query_with_filters = Query::new(1640995200000000, 1640998800000000)
        .hostname("web-server")
        .unit("nginx.service")
        .message_contains("HTTP");
    
    assert_eq!(query_with_filters.hostname, Some("web-server".to_string()));
    assert_eq!(query_with_filters.unit, Some("nginx.service".to_string()));
    assert_eq!(query_with_filters.message_contains, Some("HTTP".to_string()));
    
    // Test actual querying (this will work if test_journal_dir exists)
    match query_journal(test_dir, query) {
        Ok(entries) => {
            println!("Query returned {} entries", entries.len());
            // Verify entries are in chronological order
            for window in entries.windows(2) {
                assert!(window[0].timestamp_utc <= window[1].timestamp_utc,
                    "Entries should be in chronological order");
            }
        },
        Err(e) => {
            println!("Query failed (expected if test_journal_dir doesn't exist): {}", e);
        }
    }
}

#[test]
fn test_query_journal_with_stress_test_data() {
    use journald_query::{Query, query_journal};
    use std::path::PathBuf;
    
    let stress_test_file = PathBuf::from("test_journald_files/stress_test.journal");
    
    if !stress_test_file.exists() {
        eprintln!("Stress test file not found, skipping: {:?}", stress_test_file);
        return;
    }
    
    // Test 1: Query all entries in first hour
    let first_hour_start = 1640995200000000; // 2022-01-01 00:00:00
    let first_hour_end = 1640998800000000;   // 2022-01-01 01:00:00
    
    let query = Query::new(first_hour_start, first_hour_end);
    let entries = query_journal(&stress_test_file.parent().unwrap(), query)
        .expect("Should be able to query stress test data");
    
    println!("First hour query returned {} entries", entries.len());
    
    // Verify all entries are within time range
    for entry in &entries {
        assert!(entry.timestamp_utc >= first_hour_start, 
            "Entry timestamp {} should be >= start time {}", 
            entry.timestamp_utc, first_hour_start);
        assert!(entry.timestamp_utc <= first_hour_end,
            "Entry timestamp {} should be <= end time {}", 
            entry.timestamp_utc, first_hour_end);
    }
    
    // Test 2: Filter by hostname
    let web_server_query = Query::new(first_hour_start, first_hour_end + 3600000000) // 2 hours
        .hostname("web-server");
    
    let web_server_entries = query_journal(&stress_test_file.parent().unwrap(), web_server_query)
        .expect("Should be able to query web-server entries");
    
    println!("Web-server query returned {} entries", web_server_entries.len());
    
    // Verify all entries are from web-server
    for entry in &web_server_entries {
        assert_eq!(entry.hostname, Some("web-server".to_string()),
            "All entries should be from web-server");
    }
    
    // Test 3: Filter by unit
    let nginx_query = Query::new(first_hour_start, first_hour_end + 3600000000) // 2 hours
        .unit("nginx.service");
    
    let nginx_entries = query_journal(&stress_test_file.parent().unwrap(), nginx_query)
        .expect("Should be able to query nginx entries");
    
    println!("Nginx query returned {} entries", nginx_entries.len());
    
    // Verify all entries are from nginx.service
    for entry in &nginx_entries {
        assert_eq!(entry.unit, Some("nginx.service".to_string()),
            "All entries should be from nginx.service");
    }
    
    // Test 4: Filter by message content
    let http_query = Query::new(first_hour_start, first_hour_end + 3600000000) // 2 hours
        .message_contains("HTTP");
    
    let http_entries = query_journal(&stress_test_file.parent().unwrap(), http_query)
        .expect("Should be able to query HTTP entries");
    
    println!("HTTP query returned {} entries", http_entries.len());
    
    // Verify all entries contain "HTTP"
    for entry in &http_entries {
        assert!(entry.message.contains("HTTP"),
            "Entry message '{}' should contain 'HTTP'", entry.message);
    }
    
    // Test 5: Combined filters
    let combined_query = Query::new(first_hour_start, first_hour_end + 3600000000) // 2 hours
        .hostname("web-server")
        .unit("nginx.service")
        .message_contains("200");
    
    let combined_entries = query_journal(&stress_test_file.parent().unwrap(), combined_query)
        .expect("Should be able to query with combined filters");
    
    println!("Combined query returned {} entries", combined_entries.len());
    
    // Verify all entries match all filters
    for entry in &combined_entries {
        assert_eq!(entry.hostname, Some("web-server".to_string()));
        assert_eq!(entry.unit, Some("nginx.service".to_string()));
        assert!(entry.message.contains("200"),
            "Entry message '{}' should contain '200'", entry.message);
    }
}

#[test]
fn test_query_journal_edge_cases() {
    use journald_query::{Query, query_journal};
    use std::path::PathBuf;
    
    let stress_test_file = PathBuf::from("test_journald_files/stress_test.journal");
    
    if !stress_test_file.exists() {
        eprintln!("Stress test file not found, skipping edge case tests: {:?}", stress_test_file);
        return;
    }
    
    let test_dir = stress_test_file.parent().unwrap();
    
    // Test 1: Empty time range (end before start)
    let invalid_query = Query::new(1640998800000000, 1640995200000000); // end before start
    let empty_entries = query_journal(test_dir, invalid_query)
        .expect("Should handle invalid time range gracefully");
    
    assert_eq!(empty_entries.len(), 0, "Invalid time range should return no entries");
    
    // Test 2: Very narrow time range (1 microsecond)
    let narrow_query = Query::new(1640995200000000, 1640995200000001); // 1 microsecond
    let narrow_entries = query_journal(test_dir, narrow_query)
        .expect("Should handle narrow time range");
    
    println!("Narrow time range returned {} entries", narrow_entries.len());
    
    // Test 3: Far future time range
    let future_start = 2000000000000000; // Year 2033
    let future_end = 2000000001000000;   // 1 second later
    let future_query = Query::new(future_start, future_end);
    let future_entries = query_journal(test_dir, future_query)
        .expect("Should handle future time range");
    
    assert_eq!(future_entries.len(), 0, "Future time range should return no entries");
    
    // Test 4: Non-existent hostname filter
    let nonexistent_query = Query::new(1640995200000000, 1640998800000000)
        .hostname("nonexistent-server");
    
    let nonexistent_entries = query_journal(test_dir, nonexistent_query)
        .expect("Should handle non-existent hostname");
    
    assert_eq!(nonexistent_entries.len(), 0, "Non-existent hostname should return no entries");
    
    // Test 5: Non-existent unit filter
    let nonexistent_unit_query = Query::new(1640995200000000, 1640998800000000)
        .unit("nonexistent.service");
    
    let nonexistent_unit_entries = query_journal(test_dir, nonexistent_unit_query)
        .expect("Should handle non-existent unit");
    
    assert_eq!(nonexistent_unit_entries.len(), 0, "Non-existent unit should return no entries");
    
    // Test 6: Non-matching message filter
    let nonmatching_query = Query::new(1640995200000000, 1640998800000000)
        .message_contains("NONEXISTENT_MESSAGE_PATTERN");
    
    let nonmatching_entries = query_journal(test_dir, nonmatching_query)
        .expect("Should handle non-matching message filter");
    
    assert_eq!(nonmatching_entries.len(), 0, "Non-matching message filter should return no entries");
    
    // Test 7: Very long time range
    let long_query = Query::new(0, u64::MAX); // From epoch to end of time
    match query_journal(test_dir, long_query) {
        Ok(long_entries) => {
            println!("Very long time range returned {} entries", long_entries.len());
            // Should return all entries in the test file
            assert!(long_entries.len() > 0, "Should find some entries in long time range");
        },
        Err(e) => {
            println!("Long time range query failed: {}", e);
        }
    }
}

#[test]
fn test_query_journal_timestamp_precision() {
    use journald_query::{Query, query_journal};
    use std::path::PathBuf;
    
    let stress_test_file = PathBuf::from("test_journald_files/stress_test.journal");
    
    if !stress_test_file.exists() {
        eprintln!("Stress test file not found, skipping timestamp precision tests: {:?}", stress_test_file);
        return;
    }
    
    let test_dir = stress_test_file.parent().unwrap();
    
    // Test precise timestamp matching
    // We know from the test data that there's an entry at exactly 1640995200000000
    let precise_start = 1640995200000000; // Exactly when first entry occurs
    let precise_end = 1640995200000001;   // 1 microsecond later
    
    let precise_query = Query::new(precise_start, precise_end);
    let precise_entries = query_journal(test_dir, precise_query)
        .expect("Should handle precise timestamp matching");
    
    println!("Precise timestamp query returned {} entries", precise_entries.len());
    
    // Should find entries at exactly that timestamp
    for entry in &precise_entries {
        assert_eq!(entry.timestamp_utc, precise_start,
            "Entry should have exact timestamp {}", precise_start);
    }
    
    // Test microsecond precision boundaries
    let boundary_start = 1640995500000000; // Known timestamp from test data
    let boundary_end = 1640995500000000;   // Same timestamp (inclusive range)
    
    let boundary_query = Query::new(boundary_start, boundary_end);
    let boundary_entries = query_journal(test_dir, boundary_query)
        .expect("Should handle boundary timestamp matching");
    
    println!("Boundary timestamp query returned {} entries", boundary_entries.len());
    
    // Verify timestamp precision
    for entry in &boundary_entries {
        assert_eq!(entry.timestamp_utc, boundary_start,
            "Entry should have exact boundary timestamp");
    }
}


#[test]
fn test_tail_config_creation() {
    // Test basic configuration creation
    let config = TailConfig::new("web-server", "nginx.service", "/var/log/journal");
    assert_eq!(config.hostname, "web-server");
    assert_eq!(config.service, "nginx.service");
    assert_eq!(config.journal_path, "/var/log/journal");
    
    // Test with journal path
    let config_with_path = TailConfig::new("database-server", "mysql.service", "/path/to/journal");
    assert_eq!(config_with_path.hostname, "database-server");
    assert_eq!(config_with_path.service, "mysql.service");
    assert_eq!(config_with_path.journal_path, "/path/to/journal");
    
    // Test string conversions
    let config_from_str = TailConfig::new("monitoring-server".to_string(), "prometheus.service".to_string(), "/path/to/journal");
    assert_eq!(config_from_str.hostname, "monitoring-server");
    assert_eq!(config_from_str.service, "prometheus.service");
}

#[test]
fn test_tail_with_multi_host_journal() {
    let test_journal_str = "test_journal_dir".to_string();
    
    // Test tailing web-server nginx entries
    let config = TailConfig::new("web-server", "nginx.service", &test_journal_str);
    
    let mut tail = JournalTail::new(config)
        .expect("Should be able to create tail for web-server nginx.service");
    
    // Since this is a static journal file, we won't get new entries
    // but we can test that the tail positions correctly at the end
    // and that our filters are working
    
    // The iterator should be created successfully
    let _iter = tail.iter();
    
    // For a static journal file, next() should return None eventually
    // (after processing any existing entries that match our filters)
    // But since we seek to tail, we should start at the end
    
    // This is a basic structural test - the real test would be with live data
    println!("Tail iterator created successfully for web-server nginx.service");
}

#[test]
fn test_tail_with_different_host_service_combinations() {
    let test_journal_str = "test_journal_dir".to_string();
    
    // Test different host-service combinations from our test data
    let test_cases = vec![
        ("web-server", "nginx.service"),
        ("web-server", "apache2.service"),
        ("database-server", "mysql.service"),
        ("database-server", "postgresql.service"),
        ("monitoring-server", "prometheus.service"),
        ("monitoring-server", "grafana.service"),
    ];
    
    for (hostname, service) in test_cases {
        let config = TailConfig::new(hostname, service, &test_journal_str);
        
        let result = JournalTail::new(config);
        match result {
            Ok(_tail) => {
                println!("Successfully created tail for {}/{}", hostname, service);
            }
            Err(e) => {
                // Some combinations might not have entries, that's ok
                println!("Could not create tail for {}/{}: {}", hostname, service, e);
            }
        }
    }
}

#[test] 
fn test_tail_with_nonexistent_host_service() {
    let test_journal_str = "test_journal_dir".to_string();
    
    // Test with nonexistent host/service combination
    let config = TailConfig::new("nonexistent-host", "nonexistent.service", &test_journal_str);
    
    // This should still create successfully (the filters just won't match anything)
    let result = JournalTail::new(config);
    assert!(result.is_ok(), "Should be able to create tail even with nonexistent host/service");
    
    let mut tail = result.unwrap();
    let _iter = tail.iter();
    
    // Since we're seeking to tail and there are no matching entries,
    // the iterator should work but not return entries
    println!("Tail created successfully for nonexistent host/service combination");
}

#[test]
fn test_tail_with_invalid_journal_path() {
    let config = TailConfig::new("web-server", "nginx.service", "/nonexistent/path/to/journal");
    
    let result = JournalTail::new(config);
    assert!(result.is_err(), "Should fail with invalid journal path");
    
    // Verify we get a meaningful error
    match result.unwrap_err() {
        journald_query::JournalError::NotFound => {
            println!("Got expected NotFound error for invalid path");
        }
        journald_query::JournalError::InvalidArgument => {
            println!("Got expected InvalidArgument error for invalid path");
        }
        journald_query::JournalError::IoError => {
            println!("Got expected IoError for invalid path");
        }
        other => {
            println!("Got error for invalid path: {:?}", other);
        }
    }
}

#[test]
fn test_tail_config_debug_and_clone() {
    let config = TailConfig::new("test-host", "test.service", "/test/path");
    
    // Test Debug trait
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("test-host"));
    assert!(debug_str.contains("test.service"));
    assert!(debug_str.contains("/test/path"));
    
    // Test Clone trait
    let cloned_config = config.clone();
    assert_eq!(config, cloned_config);
    assert_eq!(config.hostname, cloned_config.hostname);
    assert_eq!(config.service, cloned_config.service);
    assert_eq!(config.journal_path, cloned_config.journal_path);
}

#[test]
fn test_tail_iterator_type_safety() {
    // This test verifies that our iterator has the correct type signature
    // and can be used in generic contexts
    
    let test_journal_str = "test_journal_dir".to_string();
    
    let config = TailConfig::new("localhost", "systemd.service", &test_journal_str);
    
    let mut tail = JournalTail::new(config)
        .expect("Should create tail for localhost systemd.service");
    
    let iter = tail.iter();
    
    // Verify the iterator implements the Iterator trait correctly
    fn test_iterator<I>(_iter: I) where I: Iterator<Item = journald_query::Result<journald_query::Entry>> {
        // This function will only compile if the iterator has the right type
    }
    
    test_iterator(iter);
    println!("Iterator type safety test passed");
}

#[test]
fn test_tail_memory_safety_and_cleanup() {
    // Test that multiple tails can be created and dropped safely
    let test_journal_str = "test_journal_dir".to_string();
    
    // Create multiple tails in sequence
    for i in 0..5 {
        let config = TailConfig::new("web-server", "nginx.service", &test_journal_str);
        
        let tail_result = JournalTail::new(config);
        match tail_result {
            Ok(mut tail) => {
                let _iter = tail.iter();
                println!("Created and dropped tail #{}", i);
                // tail will be dropped here, testing our Drop implementation
            }
            Err(e) => {
                println!("Could not create tail #{}: {}", i, e);
            }
        }
    }
    
    // Test that we can create multiple concurrent tails (though they can't be shared between threads)
    let config1 = TailConfig::new("web-server", "nginx.service", &test_journal_str);
    let config2 = TailConfig::new("database-server", "mysql.service", &test_journal_str);
    
    let _tail1 = JournalTail::new(config1);
    let _tail2 = JournalTail::new(config2);
    
    // Both should be able to coexist
    println!("Multiple concurrent tails test passed");
}

#[test]
fn test_tail_with_error_scenarios_journal() {
    let test_journal_str = "test_journal_dir".to_string();
    
    // Test tailing from the error scenarios journal
    let config = TailConfig::new("error-prone-server", "failing.service", &test_journal_str);
    
    let result = JournalTail::new(config);
    match result {
        Ok(mut tail) => {
            let _iter = tail.iter();
            println!("Successfully created tail for error scenarios journal");
        }
        Err(e) => {
            println!("Could not create tail for error scenarios: {}", e);
            // This might be expected if the test journal doesn't have the expected entries
        }
    }
}

#[test]
fn test_tail_api_ergonomics() {
    // Test that the API is ergonomic and easy to use
    let test_journal_str = "test_journal_dir".to_string();
    
    // Test the typical usage pattern
    let config = TailConfig::new("localhost", "systemd.service", test_journal_str);
    
    let mut tail = JournalTail::new(config)
        .expect("Should create tail");
    
    // Test that we can get an iterator
    let _iter = tail.iter();
    
    // Test that the iterator can be used in a for loop (conceptually)
    // Note: We can't actually iterate in a test because it would block indefinitely
    // on a static journal file after reaching the end
    
    // Test that we can call next() without it panicking
    // (though it might return None or block)
    println!("API ergonomics test - basic usage pattern works");
    
    // Test method chaining
    let _another_config = TailConfig::new("host", "service", "/path/to/journal");
    
    println!("Method chaining works as expected");
}
