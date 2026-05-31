#!/bin/bash
set -e

BINARY="vector-screen"
SERVICE="vector-screen.service"
CONFIG="vector-screen.toml"

echo "Installing Vector Screen..."

# Copy binary
echo "  Copying binary to /usr/local/bin/"
cp "target/armv7-unknown-linux-musleabihf/release/$BINARY" "/usr/local/bin/$BINARY"
chmod +x "/usr/local/bin/$BINARY"

# Copy config if not present
if [ ! -f "/etc/$CONFIG" ]; then
    echo "  Installing default config to /etc/$CONFIG"
    cp "config/$CONFIG" "/etc/$CONFIG"
else
    echo "  Config /etc/$CONFIG already exists, skipping"
fi

# Copy service file
echo "  Copying service file to /etc/systemd/system/"
cp "scripts/$SERVICE" "/etc/systemd/system/$SERVICE"

# Reload and enable
echo "  Reloading systemd daemon"
systemctl daemon-reload

echo "  Enabling $SERVICE"
systemctl enable "$SERVICE"

echo "  Starting $SERVICE"
systemctl start "$SERVICE"

echo "Done. Check status with: systemctl status $SERVICE"
