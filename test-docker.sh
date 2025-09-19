#!/bin/bash

echo "🐳 Docker Setup Test"
echo "═══════════════════════════════════════════════════════════"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running"
    echo "💡 Please start Docker Desktop or Docker Engine first"
    echo ""
    echo "Then run:"
    echo "  ./scripts/docker-run.sh dev    # Development environment"
    echo "  ./scripts/docker-run.sh prod   # Production environment"
    echo "  ./scripts/docker-run.sh nats-only  # NATS server only"
    exit 1
fi

echo "✅ Docker is running"

# Check if Docker Compose is available
if ! command -v docker-compose > /dev/null 2>&1; then
    echo "❌ Docker Compose not found"
    echo "💡 Please install Docker Compose"
    exit 1
fi

echo "✅ Docker Compose is available"

# Test NATS server
echo "📡 Testing NATS server..."
docker-compose -f docker-compose.dev.yml up -d nats

# Wait for NATS to be ready
echo "⏳ Waiting for NATS server to be ready..."
sleep 10

# Check NATS health
if curl -s http://localhost:8222/varz > /dev/null; then
    echo "✅ NATS server is running and healthy"
    echo "📊 NATS monitoring: http://localhost:8222"
else
    echo "❌ NATS server is not responding"
    docker-compose -f docker-compose.dev.yml logs nats
    exit 1
fi

# Test publisher
echo "📤 Testing document publisher..."
docker-compose -f docker-compose.dev.yml run --rm publisher

# Test subscriber (run for 10 seconds)
echo "📥 Testing document subscriber..."
timeout 10s docker-compose -f docker-compose.dev.yml run --rm subscriber || true

# Cleanup
echo "🧹 Cleaning up..."
docker-compose -f docker-compose.dev.yml down

echo "🎉 Docker setup test complete!"
echo ""
echo "To run the full system:"
echo "  ./scripts/docker-run.sh dev"
