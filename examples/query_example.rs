use journald_query::{Query, query_journal};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Journal Query Example ===");
    
    // Use our test data directory
    let journal_dir = Path::new("test_journald_files");
    
    if !journal_dir.exists() {
        println!("Test data directory not found. Run ./test_journald_files/create_test_journals.sh first");
        return Ok(());
    }
    
    // Example 1: Query all entries in a time range
    println!("\n1. Querying all entries from 2022-01-01 00:00:00 to 01:00:00 UTC:");
    let start_time = 1640995200000000; // 2022-01-01 00:00:00 UTC in microseconds
    let end_time = 1640998800000000;   // 2022-01-01 01:00:00 UTC in microseconds
    
    let query = Query::new(start_time, end_time);
    let entries = query_journal(journal_dir, query)?;
    
    println!("Found {} entries", entries.len());
    for (i, entry) in entries.iter().take(3).enumerate() {
        println!("  {}. [{}] {}: {}", 
            i + 1,
            entry.timestamp_utc,
            entry.hostname.as_deref().unwrap_or("unknown"),
            entry.message
        );
    }
    if entries.len() > 3 {
        println!("  ... and {} more entries", entries.len() - 3);
    }
    
    // Example 2: Filter by hostname
    println!("\n2. Querying entries from 'web-server' host:");
    let web_server_query = Query::new(start_time, end_time + 3600000000) // 2 hours
        .hostname("web-server");
    
    let web_entries = query_journal(journal_dir, web_server_query)?;
    println!("Found {} web-server entries", web_entries.len());
    for entry in web_entries.iter().take(2) {
        println!("  - [{}] {}: {}", 
            entry.timestamp_utc,
            entry.unit.as_deref().unwrap_or("unknown"),
            entry.message
        );
    }
    
    // Example 3: Filter by unit and message content
    println!("\n3. Querying nginx HTTP requests:");
    let http_query = Query::new(start_time, end_time + 7200000000) // 2 hours
        .unit("nginx.service")
        .message_contains("HTTP");
    
    let http_entries = query_journal(journal_dir, http_query)?;
    println!("Found {} HTTP requests", http_entries.len());
    for entry in http_entries.iter().take(3) {
        println!("  - [{}] {}", entry.timestamp_utc, entry.message);
    }
    
    // Example 4: Combined filters
    println!("\n4. Querying successful HTTP requests (200 status):");
    let success_query = Query::new(start_time, end_time + 7200000000) // 2 hours
        .hostname("web-server")
        .unit("nginx.service")
        .message_contains("200");
    
    let success_entries = query_journal(journal_dir, success_query)?;
    println!("Found {} successful HTTP requests", success_entries.len());
    for entry in success_entries.iter() {
        println!("  - [{}] {}", entry.timestamp_utc, entry.message);
    }
    
    println!("\n=== Query Example Complete ===");
    Ok(())
}
