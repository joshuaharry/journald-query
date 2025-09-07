#!/bin/bash
set -euo pipefail

export PATH=/usr/lib/systemd:$PATH

# Script to create test journal files for integration testing
# This creates export format files that can be imported by systemd

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="${SCRIPT_DIR}"

echo "Creating test journal files in: ${TEST_DIR}"

# Clean up any existing test files
rm -f "${TEST_DIR}"/*.journal*
rm -rf "${TEST_DIR}"/journal_*

echo "Creating structured test journal files..."

# Create test scenario 1: Multi-host, multi-unit
cat > "${TEST_DIR}/multi_host_multi_unit.journal" << 'EOF'
__CURSOR=s=1234567890abcdef;i=1;b=1234567890abcdef;m=1000000;t=1640995200000000;x=1
__REALTIME_TIMESTAMP=1640995200000000
__MONOTONIC_TIMESTAMP=1000000
_BOOT_ID=1234567890abcdef1234567890abcdef
_HOSTNAME=web-server
_SYSTEMD_UNIT=nginx.service
MESSAGE=Starting nginx web server
PRIORITY=6
_PID=1234
_UID=0
_GID=0

__CURSOR=s=1234567890abcdef;i=2;b=1234567890abcdef;m=1001000;t=1640995201000000;x=2
__REALTIME_TIMESTAMP=1640995201000000
__MONOTONIC_TIMESTAMP=1001000
_BOOT_ID=1234567890abcdef1234567890abcdef
_HOSTNAME=web-server
_SYSTEMD_UNIT=nginx.service
MESSAGE=HTTP request processed: GET /api/users
PRIORITY=6
_PID=1234

__CURSOR=s=1234567890abcdef;i=3;b=1234567890abcdef;m=1002000;t=1640995202000000;x=3
__REALTIME_TIMESTAMP=1640995202000000
__MONOTONIC_TIMESTAMP=1002000
_BOOT_ID=1234567890abcdef1234567890abcdef
_HOSTNAME=web-server
_SYSTEMD_UNIT=apache2.service
MESSAGE=Apache HTTP Server started
PRIORITY=6
_PID=1235

__CURSOR=s=1234567890abcdef;i=4;b=1234567890abcdef;m=1003000;t=1640995203000000;x=4
__REALTIME_TIMESTAMP=1640995203000000
__MONOTONIC_TIMESTAMP=1003000
_BOOT_ID=2234567890abcdef1234567890abcdef
_HOSTNAME=database-server
_SYSTEMD_UNIT=postgresql.service
MESSAGE=PostgreSQL database system is ready to accept connections
PRIORITY=6
_PID=2001

__CURSOR=s=1234567890abcdef;i=5;b=1234567890abcdef;m=1004000;t=1640995204000000;x=5
__REALTIME_TIMESTAMP=1640995204000000
__MONOTONIC_TIMESTAMP=1004000
_BOOT_ID=2234567890abcdef1234567890abcdef
_HOSTNAME=database-server
_SYSTEMD_UNIT=mysql.service
MESSAGE=MySQL server started successfully
PRIORITY=6
_PID=2002

__CURSOR=s=1234567890abcdef;i=6;b=1234567890abcdef;m=1005000;t=1640995205000000;x=6
__REALTIME_TIMESTAMP=1640995205000000
__MONOTONIC_TIMESTAMP=1005000
_BOOT_ID=3234567890abcdef1234567890abcdef
_HOSTNAME=monitoring-server
_SYSTEMD_UNIT=prometheus.service
MESSAGE=Prometheus metrics server started
PRIORITY=6
_PID=3001

__CURSOR=s=1234567890abcdef;i=7;b=1234567890abcdef;m=1006000;t=1640995206000000;x=7
__REALTIME_TIMESTAMP=1640995206000000
__MONOTONIC_TIMESTAMP=1006000
_BOOT_ID=3234567890abcdef1234567890abcdef
_HOSTNAME=monitoring-server
_SYSTEMD_UNIT=grafana.service
MESSAGE=Grafana dashboard server started
PRIORITY=6
_PID=3002

EOF

# Create test scenario 2: Single host, many units
cat > "${TEST_DIR}/single_host_many_units.journal" << 'EOF'
__CURSOR=s=5234567890abcdef;i=10;b=5234567890abcdef;m=2000000;t=1640995300000000;x=10
__REALTIME_TIMESTAMP=1640995300000000
__MONOTONIC_TIMESTAMP=2000000
_BOOT_ID=5234567890abcdef1234567890abcdef
_HOSTNAME=localhost
_SYSTEMD_UNIT=systemd.service
MESSAGE=System boot completed
PRIORITY=6
_PID=1

__CURSOR=s=5234567890abcdef;i=11;b=5234567890abcdef;m=2001000;t=1640995301000000;x=11
__REALTIME_TIMESTAMP=1640995301000000
__MONOTONIC_TIMESTAMP=2001000
_BOOT_ID=5234567890abcdef1234567890abcdef
_HOSTNAME=localhost
_SYSTEMD_UNIT=NetworkManager.service
MESSAGE=NetworkManager started
PRIORITY=6
_PID=100

__CURSOR=s=5234567890abcdef;i=12;b=5234567890abcdef;m=2002000;t=1640995302000000;x=12
__REALTIME_TIMESTAMP=1640995302000000
__MONOTONIC_TIMESTAMP=2002000
_BOOT_ID=5234567890abcdef1234567890abcdef
_HOSTNAME=localhost
_SYSTEMD_UNIT=sshd.service
MESSAGE=OpenSSH Daemon started
PRIORITY=6
_PID=200

__CURSOR=s=5234567890abcdef;i=13;b=5234567890abcdef;m=2003000;t=1640995303000000;x=13
__REALTIME_TIMESTAMP=1640995303000000
__MONOTONIC_TIMESTAMP=2003000
_BOOT_ID=5234567890abcdef1234567890abcdef
_HOSTNAME=localhost
_SYSTEMD_UNIT=docker.service
MESSAGE=Docker daemon started
PRIORITY=6
_PID=300

__CURSOR=s=5234567890abcdef;i=14;b=5234567890abcdef;m=2004000;t=1640995304000000;x=14
__REALTIME_TIMESTAMP=1640995304000000
__MONOTONIC_TIMESTAMP=2004000
_BOOT_ID=5234567890abcdef1234567890abcdef
_HOSTNAME=localhost
_SYSTEMD_UNIT=cron.service
MESSAGE=Cron daemon started
PRIORITY=6
_PID=400

__CURSOR=s=5234567890abcdef;i=15;b=5234567890abcdef;m=2005000;t=1640995305000000;x=15
__REALTIME_TIMESTAMP=1640995305000000
__MONOTONIC_TIMESTAMP=2005000
_BOOT_ID=5234567890abcdef1234567890abcdef
_HOSTNAME=localhost
_SYSTEMD_UNIT=systemd-logind.service
MESSAGE=User login service started
PRIORITY=6
_PID=500

EOF

# Create test scenario 3: Error and edge cases
cat > "${TEST_DIR}/error_scenarios.journal" << 'EOF'
__CURSOR=s=7234567890abcdef;i=20;b=7234567890abcdef;m=3000000;t=1640995400000000;x=20
__REALTIME_TIMESTAMP=1640995400000000
__MONOTONIC_TIMESTAMP=3000000
_BOOT_ID=7234567890abcdef1234567890abcdef
_HOSTNAME=error-prone-server
_SYSTEMD_UNIT=failing.service
MESSAGE=Critical system failure detected
PRIORITY=2
_PID=666

__CURSOR=s=7234567890abcdef;i=21;b=7234567890abcdef;m=3001000;t=1640995401000000;x=21
__REALTIME_TIMESTAMP=1640995401000000
__MONOTONIC_TIMESTAMP=3001000
_BOOT_ID=7234567890abcdef1234567890abcdef
_HOSTNAME=error-prone-server
_SYSTEMD_UNIT=broken.service
MESSAGE=Service failed to start: permission denied
PRIORITY=3
_PID=667

__CURSOR=s=7234567890abcdef;i=22;b=7234567890abcdef;m=3002000;t=1640995402000000;x=22
__REALTIME_TIMESTAMP=1640995402000000
__MONOTONIC_TIMESTAMP=3002000
_BOOT_ID=7234567890abcdef1234567890abcdef
_HOSTNAME=disk-full-server
_SYSTEMD_UNIT=disk-monitor.service
MESSAGE=Disk usage critical: 98% full
PRIORITY=1
_PID=777

__CURSOR=s=7234567890abcdef;i=23;b=7234567890abcdef;m=3003000;t=1640995403000000;x=23
__REALTIME_TIMESTAMP=1640995403000000
__MONOTONIC_TIMESTAMP=3003000
_BOOT_ID=7234567890abcdef1234567890abcdef
_HOSTNAME=memory-constrained-server
_SYSTEMD_UNIT=memory-monitor.service
MESSAGE=Memory usage warning: 85% used
PRIORITY=4
_PID=888

EOF

# Now convert these export format files to proper journal files
echo "Converting export files to journal format..."

# Create a temporary directory to work in
TEMP_DIR=$(mktemp -d)
echo "Using temporary directory: ${TEMP_DIR}"

# Function to convert export to journal
convert_export_to_journal() {
    local export_file="$1"
    local output_name="$2"
    
    echo "Converting ${export_file} to journal format..."
    
    # Use systemd-journal-remote if available, otherwise create binary format
    if command -v systemd-journal-remote >/dev/null 2>&1; then
        # systemd-journal-remote needs a specific output file, not directory
        local output_journal="${TEST_DIR}/${output_name}.journal"
        systemd-journal-remote --output="${output_journal}" - < "${export_file}" || {
            echo "Failed to convert ${export_file}, keeping export format"
            cp "${export_file}" "${TEST_DIR}/${output_name}.export"
        }
    else
        echo "systemd-journal-remote not available, keeping export format"
        cp "${export_file}" "${TEST_DIR}/${output_name}.export"
    fi
}

# Convert each export file
convert_export_to_journal "${TEST_DIR}/multi_host_multi_unit.journal" "multi_host"
convert_export_to_journal "${TEST_DIR}/single_host_many_units.journal" "single_host" 
convert_export_to_journal "${TEST_DIR}/error_scenarios.journal" "errors"

# Fix permissions on generated journal files (systemd-journal-remote creates them with restrictive permissions)
chmod 644 "${TEST_DIR}"/*.journal 2>/dev/null || true

# Clean up temporary directory
rm -rf "${TEMP_DIR}"

# Create a comprehensive test data summary
cat > "${TEST_DIR}/test_data_summary.txt" << EOF
Test Journal Files Created: $(date)

=== EXPECTED TEST DATA ===

Multi-Host Multi-Unit Scenario:
  Hosts: web-server, database-server, monitoring-server
  Units: nginx.service, apache2.service, postgresql.service, mysql.service, prometheus.service, grafana.service
  Messages: Various startup and operational messages

Single-Host Many-Units Scenario:
  Hosts: localhost
  Units: systemd.service, NetworkManager.service, sshd.service, docker.service, cron.service, systemd-logind.service
  Messages: System boot and service startup messages

Error Scenarios:
  Hosts: error-prone-server, disk-full-server, memory-constrained-server
  Units: failing.service, broken.service, disk-monitor.service, memory-monitor.service
  Messages: Various error and warning messages (priorities 1-4)

=== FILES CREATED ===
$(ls -la "${TEST_DIR}"/*.journal* "${TEST_DIR}"/*.export 2>/dev/null || echo "No journal files found")

=== USAGE ===
For Rust API testing:
  Journal::open_directory("${TEST_DIR}")

For manual verification:
  journalctl --file="${TEST_DIR}/filename.journal"

=== EXPECTED UNIQUE VALUES ===
Hosts (_HOSTNAME): web-server, database-server, monitoring-server, localhost, error-prone-server, disk-full-server, memory-constrained-server

Units (_SYSTEMD_UNIT): nginx.service, apache2.service, postgresql.service, mysql.service, prometheus.service, grafana.service, systemd.service, NetworkManager.service, sshd.service, docker.service, cron.service, systemd-logind.service, failing.service, broken.service, disk-monitor.service, memory-monitor.service
EOF

echo ""
echo "=== Test Journal Creation Complete ==="
echo "Files created in: ${TEST_DIR}"
ls -la "${TEST_DIR}"/*.journal* "${TEST_DIR}"/*.export "${TEST_DIR}"/test_data_summary.txt 2>/dev/null || true
echo ""
echo "Summary: ${TEST_DIR}/test_data_summary.txt"