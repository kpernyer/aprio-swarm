#!/bin/bash

echo "🚀 Testing Real NATS Implementation"
echo "═══════════════════════════════════════════════════════════"

# Check if NATS server is running
if ! nc -z localhost 4222 2>/dev/null; then
    echo "❌ NATS server not running on localhost:4222"
    echo "💡 Start NATS server first: nats-server"
    echo "   Or install: go install github.com/nats-io/nats-server/v2@latest"
    exit 1
fi

echo "✅ NATS server is running"

# Start subscriber in background
echo "📥 Starting document subscriber..."
cargo run --package nats-subscriber &
SUBSCRIBER_PID=$!

# Wait a moment for subscriber to start
sleep 2

# Start publisher
echo "📤 Starting document publisher..."
cargo run --package nats-publisher

# Wait a moment for messages to be processed
sleep 3

# Clean up
echo "🧹 Cleaning up..."
kill $SUBSCRIBER_PID 2>/dev/null

echo "🎉 Test complete!"
