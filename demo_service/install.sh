#!/bin/bash
# Set up the demo service on Ubuntu.

set -e

echo "🎭 Setting up Journald Query Demo Service..."

# Get the current directory (where the script is located)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "📁 Project directory: $PROJECT_DIR"

# Update the service file with the correct paths and user
SERVICE_FILE="$SCRIPT_DIR/journald-demo.service"
TEMP_SERVICE="/tmp/journald-demo.service"

# Replace placeholder paths with actual paths
sed "s|/home/josh|$HOME|g" "$SERVICE_FILE" > "$TEMP_SERVICE"
sed -i "s|User=josh|User=$USER|g" "$TEMP_SERVICE"

echo "📋 Installing systemd service unit..."
sudo cp "$TEMP_SERVICE" /etc/systemd/system/journald-demo.service
sudo systemctl daemon-reload

echo "🔧 Enabling and starting the demo service..."
sudo systemctl enable journald-demo.service
sudo systemctl start journald-demo.service

echo "✅ Demo service installed and started!"
echo ""
echo "📊 Check service status:"
echo "   sudo systemctl status journald-demo.service"
echo ""
echo "📜 View live logs:"
echo "   journalctl -u journald-demo.service -f"
echo ""
echo "🌐 Now start the web demo:"
echo "   cd $PROJECT_DIR"
echo "   cargo run --example sse"
echo ""
echo "🎯 Then open http://localhost:3000 and use:"
echo "   Hostname: demo-web-server"
echo "   Service: journald-demo.service"

# Clean up
rm -f "$TEMP_SERVICE"