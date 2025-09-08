use crate::journal::Journal;
use crate::error::Result;
use std::path::Path;
use std::collections::HashSet;

/// Represents a single host and its associated systemd units
/// 
/// This struct contains information about a host that has logged entries
/// to the systemd journal, along with all the systemd units that have
/// logged entries from that host.
#[derive(Debug, Clone, PartialEq)]
pub struct Host {
    /// The hostname of the system
    pub hostname: String,
    /// List of systemd unit names that have logged entries from this host
    pub units: Vec<String>,
}

/// Collection of hosts discovered from journal logs
/// 
/// This struct represents the result of scanning journal logs to discover
/// all hosts and their associated systemd units. It provides methods to
/// access and iterate over the discovered services.
#[derive(Debug, Clone, PartialEq)]
pub struct Hosts {
    /// Vector of all discovered hosts and their units
    pub hosts: Vec<Host>,
}

impl Hosts {
    /// Create a new empty Hosts collection
    pub fn new() -> Self {
        Hosts {
            hosts: Vec::new(),
        }
    }

    /// Get the number of hosts discovered
    pub fn len(&self) -> usize {
        self.hosts.len()
    }

    /// Check if any hosts were discovered
    pub fn is_empty(&self) -> bool {
        self.hosts.is_empty()
    }

    /// Get all unique hostnames
    pub fn hostnames(&self) -> Vec<&String> {
        self.hosts.iter().map(|host| &host.hostname).collect()
    }

    /// Get all unique units across all hosts
    pub fn all_units(&self) -> Vec<&String> {
        self.hosts
            .iter()
            .flat_map(|host| &host.units)
            .collect()
    }

    /// Find a host by hostname
    pub fn find_host(&self, hostname: &str) -> Option<&Host> {
        self.hosts.iter().find(|host| host.hostname == hostname)
    }
}

/// Discover services grouped by host from journal logs
/// 
/// This function scans journal logs to discover all hosts and their associated
/// systemd units, returning the data in a structured format that groups units
/// by their host.
/// 
/// # Arguments
/// * `journal_dir` - Directory containing journal files
/// 
/// # Returns
/// A `Hosts` struct containing all discovered hosts and their units
/// 
/// # Examples
/// ```no_run
/// use journald_query::discover_services;
/// 
/// let services = discover_services("/var/log/journal")?;
/// println!("Found {} hosts", services.len());
/// 
/// for host in &services.hosts {
///     println!("Host: {} has {} units", host.hostname, host.units.len());
/// }
/// # Ok::<(), journald_query::JournalError>(())
/// ```
pub fn discover_services<P: AsRef<Path>>(journal_dir: P) -> Result<Hosts> {
    let journal = Journal::open_directory(journal_dir)?;
    discover_services_from_journal(&journal)
}

/// Ideally we could use sd_journal_enumerate_entries with a couple of filters
/// to get the results, but according to the API docs:
/// 
/// "Note that these functions currently are not influenced by matches set with sd_journal_add_match() but 
/// this might change in a later version of this software."
/// 
/// As such, we have to instead:
/// - Query to get all the unique hostnames
/// - Query to get all the unique units
/// - Check for each hostname+unit combination if it exists in the journal
/// 
/// This is... not great, but the best one can reasonably do with the API.
fn discover_services_from_journal(journal: &Journal) -> Result<Hosts> {
    let hostname_values = journal.get_unique_values("_HOSTNAME")?;
    let hostnames: HashSet<String> = hostname_values
        .into_iter()
        .filter_map(|value| value.strip_prefix("_HOSTNAME=").map(|s| s.to_string()))
        .collect();
    
    let unit_values = journal.get_unique_values("_SYSTEMD_UNIT")?;
    let units: HashSet<String> = unit_values
        .into_iter()
        .filter_map(|value| value.strip_prefix("_SYSTEMD_UNIT=").map(|s| s.to_string()))
        .collect();
    
    let mut host_units: std::collections::HashMap<String, HashSet<String>> = std::collections::HashMap::new();
    
    for hostname in &hostnames {
        let mut units_for_host = HashSet::new();
        // For each unit, check if the hostname+unit combination exists
        for unit in &units {
            journal.flush_matches();
            journal.add_match("_HOSTNAME", hostname)?;
            journal.add_match("_SYSTEMD_UNIT", unit)?;
            journal.seek_head()?;
            if journal.next()? {
                units_for_host.insert(unit.clone());
            }
        }
        
        host_units.insert(hostname.clone(), units_for_host);
    }
    
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

#[cfg(test)]
mod tests {
    use super::{Host, Hosts};
    
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

    #[test]
    fn test_host_struct_creation() {
        let host = Host {
            hostname: "test-server".to_string(),
            units: vec!["sshd.service".to_string(), "nginx.service".to_string()],
        };
        
        assert_eq!(host.hostname, "test-server");
        assert_eq!(host.units.len(), 2);
        assert!(host.units.contains(&"sshd.service".to_string()));
    }

    #[test]
    fn test_hosts_struct_methods() {
        let hosts = Hosts {
            hosts: vec![
                Host {
                    hostname: "server1".to_string(),
                    units: vec!["sshd.service".to_string(), "nginx.service".to_string()],
                },
                Host {
                    hostname: "server2".to_string(),
                    units: vec!["mysql.service".to_string()],
                },
            ],
        };
        
        assert_eq!(hosts.len(), 2);
        assert!(!hosts.is_empty());
        
        let hostnames = hosts.hostnames();
        assert_eq!(hostnames.len(), 2);
        assert!(hostnames.contains(&&"server1".to_string()));
        assert!(hostnames.contains(&&"server2".to_string()));
        
        let all_units = hosts.all_units();
        assert_eq!(all_units.len(), 3);
        assert!(all_units.contains(&&"sshd.service".to_string()));
        assert!(all_units.contains(&&"nginx.service".to_string()));
        assert!(all_units.contains(&&"mysql.service".to_string()));
        
        let found_host = hosts.find_host("server1");
        assert!(found_host.is_some());
        assert_eq!(found_host.unwrap().hostname, "server1");
        
        let not_found = hosts.find_host("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_empty_hosts() {
        let hosts = Hosts::new();
        assert_eq!(hosts.len(), 0);
        assert!(hosts.is_empty());
        assert_eq!(hosts.hostnames().len(), 0);
        assert_eq!(hosts.all_units().len(), 0);
    }
}
