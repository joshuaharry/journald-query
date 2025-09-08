# journald-query

A Rust library for conveniently interacting with the systemd journal.

## Motivation

We want to make the following queries about the systemd journal safe,
fast, and convenient from a Rust program: 

1. Finding (given a directory of journal logs) the hosts and units of
   that are available.
2. Querying (given a host and systemd unit) the logs for a service from
   with filters for:
   2a. The time frame (e.g., from time T1 to time T2)
   2b. Arbitrary string filtering (e.g., all logs that say "foo")
3. Live tailing the logs for a service, given the host and systemd unit.

To that end, we:

1. Provide safe wrappers for part of the systemd journal API, as documented
   here: https://www.freedesktop.org/software/systemd/man/latest/sd-journal.html
2. Provides higher-level abstractions on top of said API so that it
   is easier to use.

Use this API to build higher-level applications, such as GUIs or TUI
apps that are nicer than `journalctl` based shell scripts.

## API

The `journald-query` crate provides three main APIs for working with systemd journal logs:

### 1. **Service Discovery**

Discover all hosts and services from journal logs.

```rust
use journald_query::discover_services;

// Discover all hosts and their services
let services = discover_services("/var/log/journal")?;

println!("Found {} hosts", services.len());
for host in &services.hosts {
    println!("Host: {} ({} services)", host.hostname, host.units.len());
    for unit in &host.units {
        println!("  - {}", unit);
    }
}

// Find a specific host
if let Some(host) = services.find_host("web-server") {
    println!("Web server has {} services", host.units.len());
}

// Get all unique services across all hosts
let all_services = services.all_units();
```

**Key Types:**
- `Host` - A single host with its services
- `Hosts` - Collection of all discovered hosts

### 2. **Historical Queries**

Query journal logs between specific time ranges with filters.

```rust
use journald_query::{Query, query_journal};

// Create a time-based query (timestamps in microseconds since Unix epoch)
let start_time = 1640995200000000; // 2022-01-01 00:00:00 UTC
let end_time   = 1640998800000000; // 2022-01-01 01:00:00 UTC

let query = Query::new(start_time, end_time)
    .hostname("web-server")           // Filter by hostname
    .unit("nginx.service")            // Filter by service
    .message_contains("ERROR");       // Filter by message content

// Execute the query
let entries = query_journal("/var/log/journal", query)?;

for entry in entries {
    println!("[{}] {}: {}", 
        entry.timestamp_utc, 
        entry.hostname.unwrap_or("unknown".to_string()),
        entry.message
    );
}
```

**Key Types:**
- `Query` - Fluent query builder with time range and filters
- `Entry` - A single journal entry with timestamp, hostname, unit, and message

### 3. **Live Tailing** (`tail.rs`)

Stream journal entries in real-time with configurable polling.

```rust
use journald_query::{TailConfig, JournalTail};
use std::time::Duration;

// Configure live tailing
let config = TailConfig::new("web-server", "nginx.service", "/var/log/journal")
    .with_poll_interval_ms(50)        // Poll every 50ms (fast)
    .with_start_time_offset_secs(30); // Start from 30 seconds ago

// Create the tail and iterate
let mut tail = JournalTail::new(config)?;
for entry in tail.iter() {
    match entry {
        Ok(entry) => println!("{}: {}", entry.timestamp_utc, entry.message),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

**Configuration Options:**
```rust
// Different polling strategies
let dashboard_config = TailConfig::new("host", "service", "/path")
    .with_poll_interval_ms(25)        // Very fast (25ms) for dashboards
    .with_start_time_offset_secs(60); // 1 minute of history

let background_config = TailConfig::new("host", "service", "/path")
    .with_poll_interval_ms(500)       // Slower (500ms) for background monitoring
    .with_start_time_offset_secs(300); // 5 minutes of history

let realtime_config = TailConfig::new("host", "service", "/path")
    .from_now()                       // No historical entries
    .with_poll_interval_ms(50);       // Fast polling for real-time alerts
```

**Key Types:**
- `TailConfig` - Configuration for live tailing with fluent API
- `JournalTail` - Live tail instance that provides an iterator
- `JournalIterator` - Iterator that yields journal entries in real-time

---

## **Quick Start Examples**

### Discover Services
```rust
let services = discover_services("/var/log/journal")?;
println!("Found {} hosts with {} total services", 
    services.len(), 
    services.all_units().len()
);
```

### Query Historical Logs
```rust
let entries = query_journal("/var/log/journal", 
    Query::new(start_time, end_time)
        .hostname("web-server")
        .unit("nginx.service")
)?;
println!("Found {} matching entries", entries.len());
```

### Live Tail Logs
```rust
let mut tail = JournalTail::new(
    TailConfig::new("web-server", "nginx.service", "/var/log/journal")
)?;
for entry in tail.iter().take(10) {
    println!("{}", entry?.message);
}
```

If you want to see how to use the tailing in a web app, please see
the file `examples/sse.rs` for a demo using Poem and Tokio.

## Dependencies

This library requires `libsystemd` to be available at build time.

### Standard Installation

```bash
# Ubuntu/Debian
sudo apt install libsystemd-dev

# RHEL/CentOS/Fedora  
sudo yum install systemd-devel
```

Then you can build your app.

### Custom Installation (NixOS, etc.)

If you have `libsystemd` installed in a non-standard location, you can specify the path using environment variables:

```bash
# Point to the directory containing libsystemd.so or libsystemd.a
export LIB_SYSTEMD_PATH="/nix/store/.../lib"

# Optionally, specify header path if needed
export INCLUDE_SYSTEMD_PATH="/nix/store/.../include"

cargo build
```

### NixOS Example

```bash
# In a nix-shell with systemd
nix-shell -p systemd.dev

# Or set the path explicitly
export LIB_SYSTEMD_PATH="$(nix-build '<nixpkgs>' -A systemd.lib --no-out-link)/lib"
cargo build
```

The build script will check for libraries in this order:
1. `LIB_SYSTEMD_PATH` environment variable (if set)
2. `pkg-config --libs libsystemd`
3. Standard system library paths

## Live Journal Streaming Demo

This crate includes a demo of live-streaming logs in the demo_service folder.
You can set it up on a Linux machine as follows:

### Quick Setup (Ubuntu)

1. **Install the demo service:**
   ```bash
   cd demo_service
   ./install.sh
   ```
   This creates and starts a systemd service that generates amusing log messages every 0.5-1.5 seconds.

2. **Start the web server:**
   ```bash
   cargo run --example sse
   ```

3. **Open your browser to `http://localhost:3000`**

4. **Use these filter values:**
   - **Hostname:** `journald-query` (or whatever your hostname is; on my development machine, it's sdjournal-rs)
   - **Service:** `journald-demo.service`

5. **Watch live logs stream!** You'll see messages like:
   - 🚀 "Processing rocket fuel request from Mars Base Alpha"
   - 🔧 "Calibrating flux capacitor to 1.21 gigawatts"  
   - 🐱 "Cat detected on keyboard. Initiating emergency protocols."

### How It Works

- **Real systemd service:** Creates an actual systemd service that logs to the journal
- **Live filtering:** Web interface filters by `SYSLOG_IDENTIFIER=demo-web-server` and `_SYSTEMD_UNIT=journald-demo.service`  
- **Server-Sent Events:** New journal entries are streamed in real-time via SSE
- **Cross-platform:** Works on any Linux system with systemd

### Cleanup

To remove the demo service:
```bash
sudo systemctl stop journald-demo.service
sudo systemctl disable journald-demo.service
sudo rm /etc/systemd/system/journald-demo.service
sudo systemctl daemon-reload
```