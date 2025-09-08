use journald_query::{discover_services, JournalError};

fn main() -> Result<(), JournalError> {
    // Use the test journal directory
    let test_journal_dir = "test_journal_dir";
    
    println!("🔍 Discovering services from test journal directory...");
    println!("📁 Using directory: {}", test_journal_dir);
    println!();
    
    // Discover services using the actual API
    let services = discover_services(test_journal_dir)?;
    
    println!("🔍 Discovered {} hosts with their services:", services.len());
    println!();
    
    for host in &services.hosts {
        println!("🖥️  Host: {}", host.hostname);
        println!("   📦 Services ({} total):", host.units.len());
        
        // Show first few services to avoid overwhelming output
        let display_count = std::cmp::min(host.units.len(), 5);
        for (i, unit) in host.units.iter().take(display_count).enumerate() {
            println!("   {}. {}", i + 1, unit);
        }
        
        if host.units.len() > display_count {
            println!("   ... and {} more", host.units.len() - display_count);
        }
        println!();
    }
    
    // Show summary statistics
    let total_units: usize = services.hosts.iter().map(|h| h.units.len()).sum();
    let unique_units = services.all_units().len();
    
    println!("📊 Summary:");
    println!("   • Total hosts: {}", services.len());
    println!("   • Total service instances: {}", total_units);
    println!("   • Unique services: {}", unique_units);
    
    // Example of finding a specific host
    if let Some(host) = services.find_host("localhost") {
        println!("   • localhost has {} services", host.units.len());
    }
    
    // Show some example hosts from our test data
    println!("\n🔍 Looking for specific test hosts:");
    let test_hosts = ["web-server", "database-server", "monitoring-server"];
    
    for test_host in &test_hosts {
        if let Some(host) = services.find_host(test_host) {
            println!("   ✓ Found {}: {} services", test_host, host.units.len());
            // Show first few services for this host
            for (i, unit) in host.units.iter().take(3).enumerate() {
                println!("     {}. {}", i + 1, unit);
            }
            if host.units.len() > 3 {
                println!("     ... and {} more", host.units.len() - 3);
            }
        } else {
            println!("   ✗ {} not found", test_host);
        }
    }
    
    Ok(())
}
