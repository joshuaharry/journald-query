use crate::journal::Journal;
use crate::error::Result;
use std::path::Path;

/// Discover hosts and systemd units available in journal logs
/// 
/// This module provides functionality to scan journal directories and
/// find what hosts and systemd units have logged entries.

/// Find all unique hostnames in the journal
/// 
/// # Arguments
/// * `journal_dir` - Directory containing journal files
/// 
/// # Returns
/// Vector of hostnames found in the journal
/// 
/// # Examples
/// ```no_run
/// use journald_query::discover_hosts;
/// 
/// let hosts = discover_hosts("/var/log/journal")?;
/// for host in hosts {
///     println!("Found host: {}", host);
/// }
/// # Ok::<(), journald_query::JournalError>(())
/// ```
pub fn discover_hosts<P: AsRef<Path>>(journal_dir: P) -> Result<Vec<String>> {
    let journal = Journal::open_directory(journal_dir)?;
    let raw_values = journal.get_unique_values("_HOSTNAME")?;
    
    // Extract just the hostname part (remove "_HOSTNAME=" prefix)
    let hosts = raw_values
        .into_iter()
        .filter_map(|value| {
            if let Some(hostname) = value.strip_prefix("_HOSTNAME=") {
                Some(hostname.to_string())
            } else {
                None
            }
        })
        .collect();
    
    Ok(hosts)
}

/// Find all unique systemd units in the journal
/// 
/// # Arguments
/// * `journal_dir` - Directory containing journal files
/// 
/// # Returns
/// Vector of systemd unit names found in the journal
/// 
/// # Examples
/// ```no_run
/// use journald_query::discover_units;
/// 
/// let units = discover_units("/var/log/journal")?;
/// for unit in units {
///     println!("Found unit: {}", unit);
/// }
/// # Ok::<(), journald_query::JournalError>(())
/// ```
pub fn discover_units<P: AsRef<Path>>(journal_dir: P) -> Result<Vec<String>> {
    let journal = Journal::open_directory(journal_dir)?;
    let raw_values = journal.get_unique_values("_SYSTEMD_UNIT")?;
    
    // Extract just the unit name part (remove "_SYSTEMD_UNIT=" prefix)
    let units = raw_values
        .into_iter()
        .filter_map(|value| {
            if let Some(unit) = value.strip_prefix("_SYSTEMD_UNIT=") {
                Some(unit.to_string())
            } else {
                None
            }
        })
        .collect();
    
    Ok(units)
}

/// Discover both hosts and units in a single journal scan
/// 
/// This is more efficient than calling `discover_hosts` and `discover_units`
/// separately when you need both.
/// 
/// # Arguments
/// * `journal_dir` - Directory containing journal files
/// 
/// # Returns
/// Tuple of (hosts, units) vectors
/// 
/// # Examples
/// ```no_run
/// use journald_query::discover_hosts_and_units;
/// 
/// let (hosts, units) = discover_hosts_and_units("/var/log/journal")?;
/// println!("Found {} hosts and {} units", hosts.len(), units.len());
/// # Ok::<(), journald_query::JournalError>(())
/// ```
pub fn discover_hosts_and_units<P: AsRef<Path>>(journal_dir: P) -> Result<(Vec<String>, Vec<String>)> {
    let journal = Journal::open_directory(journal_dir)?;
    
    // Get hosts
    let raw_hosts = journal.get_unique_values("_HOSTNAME")?;
    let hosts = raw_hosts
        .into_iter()
        .filter_map(|value| {
            value.strip_prefix("_HOSTNAME=").map(|s| s.to_string())
        })
        .collect();
    
    // Get units  
    let raw_units = journal.get_unique_values("_SYSTEMD_UNIT")?;
    let units = raw_units
        .into_iter()
        .filter_map(|value| {
            value.strip_prefix("_SYSTEMD_UNIT=").map(|s| s.to_string())
        })
        .collect();
    
    Ok((hosts, units))
}

#[cfg(test)]
mod tests {
    
    // These are unit tests that don't require actual journal files
    
    #[test]
    fn test_hostname_extraction() {
        let raw_values = vec![
            "_HOSTNAME=server1".to_string(),
            "_HOSTNAME=server2".to_string(),
            "_HOSTNAME=localhost".to_string(),
        ];
        
        let hosts: Vec<String> = raw_values
            .into_iter()
            .filter_map(|value| {
                value.strip_prefix("_HOSTNAME=").map(|s| s.to_string())
            })
            .collect();
        
        assert_eq!(hosts, vec!["server1", "server2", "localhost"]);
    }
    
    #[test]
    fn test_unit_extraction() {
        let raw_values = vec![
            "_SYSTEMD_UNIT=sshd.service".to_string(),
            "_SYSTEMD_UNIT=nginx.service".to_string(),
            "_SYSTEMD_UNIT=systemd-logind.service".to_string(),
        ];
        
        let units: Vec<String> = raw_values
            .into_iter()
            .filter_map(|value| {
                value.strip_prefix("_SYSTEMD_UNIT=").map(|s| s.to_string())
            })
            .collect();
        
        assert_eq!(units, vec!["sshd.service", "nginx.service", "systemd-logind.service"]);
    }
    
    #[test]
    fn test_malformed_values_filtered() {
        let raw_values = vec![
            "_HOSTNAME=server1".to_string(),
            "INVALID=value".to_string(),  // Should be filtered out
            "_HOSTNAME=server2".to_string(),
            "".to_string(),  // Should be filtered out
        ];
        
        let hosts: Vec<String> = raw_values
            .into_iter()
            .filter_map(|value| {
                value.strip_prefix("_HOSTNAME=").map(|s| s.to_string())
            })
            .collect();
        
        assert_eq!(hosts, vec!["server1", "server2"]);
    }
}
