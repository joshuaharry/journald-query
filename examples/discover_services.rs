use journald_query::{Journal, JournalError, Host, Hosts};

fn main() -> Result<(), JournalError> {
    // Use a single test journal file for clear demonstration
    let test_file = "test_journald_files/multi_host.journal";
    
    println!("🔍 Discovering services from test journal file...");
    println!("📁 Using file: {}", test_file);
    println!();
    
    // Discover services using our test file
    let services = discover_services_from_files(vec![test_file])?;
    
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
    
    Ok(())
}

/// Discover services from specific journal files (similar to discover_services but for files)
fn discover_services_from_files<P: AsRef<std::path::Path>>(file_paths: Vec<P>) -> Result<Hosts, JournalError> {
    use std::collections::{HashMap, HashSet};
    
    let journal = Journal::open_files(file_paths)?;
    
    // Since sd_journal_query_unique ignores match filters (per systemd docs),
    // we need to iterate through entries manually to correlate hosts with units
    let mut host_units: HashMap<String, HashSet<String>> = HashMap::new();
    
    // Clear any existing matches and iterate through all entries
    journal.flush_matches();
    journal.seek_head()?;
    
    while journal.next()? {
        // Get hostname and unit from current entry
        let hostname = journal.get_field("_HOSTNAME")?;
        let unit = journal.get_field("_SYSTEMD_UNIT")?;
        
        if let (Some(hostname_raw), Some(unit_raw)) = (hostname, unit) {
            // Extract the actual values (remove field name prefixes)
            if let Some(hostname) = hostname_raw.strip_prefix("_HOSTNAME=") {
                if let Some(unit) = unit_raw.strip_prefix("_SYSTEMD_UNIT=") {
                    host_units
                        .entry(hostname.to_string())
                        .or_insert_with(HashSet::new)
                        .insert(unit.to_string());
                }
            }
        }
    }
    
    // Convert to our Host/Hosts structure
    let mut hosts = Vec::new();
    for (hostname, units_set) in host_units {
        let mut units: Vec<String> = units_set.into_iter().collect();
        units.sort(); // Sort for consistent output
        
        hosts.push(Host {
            hostname,
            units,
        });
    }
    
    // Sort hosts by hostname for consistent output
    hosts.sort_by(|a, b| a.hostname.cmp(&b.hostname));
    
    Ok(Hosts { hosts })
}
