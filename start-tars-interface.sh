#!/bin/bash

# TARS Interface Startup Script
# This script starts the TARS web interface server

set -e

echo "🤖 TARS INTERFACE STARTUP"
echo "========================="

# Check if Node.js is available
if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Installing Node.js..."
    
    # Install Node.js based on system
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
        sudo apt-get install -y nodejs
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if command -v brew &> /dev/null; then
            brew install node
        else
            echo "Please install Node.js manually from https://nodejs.org/"
            exit 1
        fi
    else
        echo "Please install Node.js manually from https://nodejs.org/"
        exit 1
    fi
fi

# Check Node.js version
node_version=$(node --version)
echo "✅ Node.js version: $node_version"

# Get network interfaces for LAN access
get_local_ip() {
    if command -v ip &> /dev/null; then
        # Linux
        ip route get 1.1.1.1 | grep -oP 'src \K\S+' 2>/dev/null || echo "localhost"
    elif command -v ifconfig &> /dev/null; then
        # macOS/BSD
        ifconfig | grep "inet " | grep -v 127.0.0.1 | awk '{print $2}' | head -1 || echo "localhost"
    else
        echo "localhost"
    fi
}

LOCAL_IP=$(get_local_ip)

# Set port (default 3000)
PORT=${1:-3000}

echo ""
echo "🚀 Starting TARS Interface Server..."
echo "   Port: $PORT"
echo "   Local IP: $LOCAL_IP"
echo ""

# Change to web-interface directory
cd "$(dirname "$0")/web-interface"

# Make server.js executable
chmod +x server.js

# Start the server
echo "🌟 Launching TARS..."
node server.js $PORT

# This will run when the server is stopped
echo ""
echo "🔴 TARS Interface Server stopped."
echo "That's what I would have said. Eventually."
