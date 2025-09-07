use journald_query::Journal;
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
