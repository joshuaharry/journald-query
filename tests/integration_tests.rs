use journald_query::{Journal, discover_hosts, discover_units, discover_hosts_and_units};
use std::path::Path;

// These integration tests require libsystemd to be installed and linked.

#[test]
#[ignore] // Ignore by default since it requires system setup
fn test_journal_open_system_directory() {
    // Try to open the system journal directory
    let system_journal_dir = "/var/log/journal";
    
    if !Path::new(system_journal_dir).exists() {
        eprintln!("System journal directory not found, skipping test");
        return;
    }
    
    let result = Journal::open_directory(system_journal_dir);
    
    match result {
        Ok(_journal) => {
            println!("Successfully opened system journal directory");
            // Journal will be closed automatically when dropped
        }
        Err(e) => {
            eprintln!("Failed to open system journal: {}", e);
            // This might fail due to permissions, which is expected
        }
    }
}

#[test]
#[ignore] // Ignore by default since it requires system setup
fn test_discover_hosts_system() {
    let system_journal_dir = "/var/log/journal";
    
    if !Path::new(system_journal_dir).exists() {
        eprintln!("System journal directory not found, skipping test");
        return;
    }
    
    match discover_hosts(system_journal_dir) {
        Ok(hosts) => {
            println!("Found {} hosts:", hosts.len());
            for host in &hosts {
                println!("  - {}", host);
            }
            // Should find at least the local hostname
            assert!(!hosts.is_empty(), "Should find at least one host");
        }
        Err(e) => {
            eprintln!("Failed to discover hosts: {}", e);
            // This might fail due to permissions or missing journal files
        }
    }
}

#[test]
#[ignore] // Ignore by default since it requires system setup
fn test_discover_units_system() {
    let system_journal_dir = "/var/log/journal";
    
    if !Path::new(system_journal_dir).exists() {
        eprintln!("System journal directory not found, skipping test");
        return;
    }
    
    match discover_units(system_journal_dir) {
        Ok(units) => {
            println!("Found {} units:", units.len());
            for unit in &units {
                println!("  - {}", unit);
            }
            // Should find at least some systemd units
            assert!(!units.is_empty(), "Should find at least one unit");
        }
        Err(e) => {
            eprintln!("Failed to discover units: {}", e);
            // This might fail due to permissions or missing journal files
        }
    }
}

#[test]
#[ignore] // Ignore by default since it requires system setup
fn test_discover_hosts_and_units_system() {
    let system_journal_dir = "/var/log/journal";
    
    if !Path::new(system_journal_dir).exists() {
        eprintln!("System journal directory not found, skipping test");
        return;
    }
    
    match discover_hosts_and_units(system_journal_dir) {
        Ok((hosts, units)) => {
            println!("Found {} hosts and {} units", hosts.len(), units.len());
            
            // Should find at least something
            assert!(!hosts.is_empty() || !units.is_empty(), 
                    "Should find at least one host or unit");
        }
        Err(e) => {
            eprintln!("Failed to discover hosts and units: {}", e);
            // This might fail due to permissions or missing journal files
        }
    }
}

#[test]
fn test_journal_unique_query_basic() {
    // This test doesn't require real journal files
    // It tests the basic API structure
    
    // Try to open a non-existent directory - should fail gracefully
    let result = Journal::open_directory("/nonexistent/journal/directory");
    assert!(result.is_err(), "Should fail to open non-existent directory");
    
    // The error should be meaningful
    let err = result.unwrap_err();
    println!("Expected error for non-existent directory: {}", err);
}

// Helper function to check if we can run integration tests
#[allow(dead_code)]
fn can_run_integration_tests() -> bool {
    // Check if we have libsystemd available
    std::env::var("JOURNALD_QUERY_NO_LINK").is_err() && 
    Path::new("/var/log/journal").exists()
}

// Instructions for running integration tests
#[test]
fn integration_test_instructions() {
    println!();
    println!("=== Integration Test Instructions ===");
    println!("To run integration tests that require libsystemd and journal files:");
    println!("  cargo test -- --ignored");
    println!();
    println!("These tests require:");
    println!("  1. libsystemd-dev package installed");
    println!("  2. systemd journal files present (usually in /var/log/journal)");
    println!("  3. Appropriate permissions to read journal files");
    println!();
    println!("On Ubuntu/Debian: sudo apt install libsystemd-dev");
    println!("On RHEL/CentOS: sudo yum install systemd-devel");
    println!();
}
