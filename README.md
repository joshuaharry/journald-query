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

## Dependencies

This library requires `libsystemd` to be available at build time.

### Standard Installation

```bash
# Ubuntu/Debian
sudo apt install libsystemd-dev

# RHEL/CentOS/Fedora  
sudo yum install systemd-devel
```

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