#!/bin/bash

set -e

echo "🔨 Building Minimal Rust Docker Images"
echo "═══════════════════════════════════════════════════════════"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker daemon is not running."
    echo "💡 Please start Docker Desktop or your Docker daemon."
    exit 1
fi

echo "✅ Docker daemon is running."

echo "📦 Building minimal images..."

# Build the minimal image
echo "Building FROM scratch image..."
docker build -f Dockerfile.minimal -t aprio-swarm-minimal:latest .

# Build the distroless image
echo "Building distroless image..."
docker build -f Dockerfile.distroless -t aprio-swarm-distroless:latest .

echo "📊 Image sizes:"
echo "Minimal (FROM scratch):"
docker images aprio-swarm-minimal:latest --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

echo "Distroless:"
docker images aprio-swarm-distroless:latest --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

echo "🎉 Build complete!"
echo ""
echo "To run the minimal setup:"
echo "  docker compose -f docker-compose.minimal.yml up -d"
echo ""
echo "To test individual images:"
echo "  docker run --rm aprio-swarm-minimal:latest"
echo "  docker run --rm aprio-swarm-distroless:latest"
