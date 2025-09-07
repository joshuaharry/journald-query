use journald_query::{discover_hosts, discover_units, discover_hosts_and_units};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <journal_directory>", args[0]);
        eprintln!("Example: {} /var/log/journal", args[0]);
        std::process::exit(1);
    }
    
    let journal_dir = &args[1];
    
    println!("Discovering hosts and units in: {}", journal_dir);
    println!();
    
    // Method 1: Discover hosts and units separately
    println!("=== Method 1: Separate Discovery ===");
    
    match discover_hosts(journal_dir) {
        Ok(hosts) => {
            println!("Found {} hosts:", hosts.len());
            for host in &hosts {
                println!("  📡 {}", host);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to discover hosts: {}", e);
        }
    }
    
    println!();
    
    match discover_units(journal_dir) {
        Ok(units) => {
            println!("Found {} systemd units:", units.len());
            for unit in &units {
                println!("  ⚙️  {}", unit);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to discover units: {}", e);
        }
    }
    
    println!();
    
    // Method 2: Discover both at once (more efficient)
    println!("=== Method 2: Combined Discovery ===");
    
    match discover_hosts_and_units(journal_dir) {
        Ok((hosts, units)) => {
            println!("✅ Successfully discovered:");
            println!("   📡 {} hosts", hosts.len());
            println!("   ⚙️  {} units", units.len());
            
            if !hosts.is_empty() {
                println!("\n🏠 Sample hosts:");
                for host in hosts.iter().take(5) {
                    println!("   - {}", host);
                }
                if hosts.len() > 5 {
                    println!("   ... and {} more", hosts.len() - 5);
                }
            }
            
            if !units.is_empty() {
                println!("\n🔧 Sample units:");
                for unit in units.iter().take(5) {
                    println!("   - {}", unit);
                }
                if units.len() > 5 {
                    println!("   ... and {} more", units.len() - 5);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to discover hosts and units: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
