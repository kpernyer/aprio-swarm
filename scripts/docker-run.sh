#!/bin/bash

# Docker run script for Aprio Swarm
set -e

echo "🐳 Aprio Swarm Docker Setup"
echo "═══════════════════════════════════════════════════════════"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Check if Docker Compose is available
if ! command -v docker-compose > /dev/null 2>&1; then
    echo "❌ Docker Compose not found. Please install Docker Compose."
    exit 1
fi

echo "✅ Docker and Docker Compose are available"

# Parse command line arguments
MODE=${1:-"dev"}

case $MODE in
    "dev")
        echo "🔧 Starting development environment..."
        docker-compose -f docker-compose.dev.yml up --build
        ;;
    "prod")
        echo "🚀 Starting production environment..."
        docker-compose up --build
        ;;
    "nats-only")
        echo "📡 Starting NATS server only..."
        docker-compose -f docker-compose.dev.yml up nats
        ;;
    "clean")
        echo "🧹 Cleaning up Docker resources..."
        docker-compose -f docker-compose.dev.yml down -v
        docker-compose down -v
        docker system prune -f
        echo "✅ Cleanup complete"
        ;;
    "logs")
        echo "📋 Showing logs..."
        docker-compose -f docker-compose.dev.yml logs -f
        ;;
    *)
        echo "Usage: $0 [dev|prod|nats-only|clean|logs]"
        echo ""
        echo "Commands:"
        echo "  dev       - Start development environment (default)"
        echo "  prod      - Start production environment"
        echo "  nats-only - Start only NATS server"
        echo "  clean     - Clean up Docker resources"
        echo "  logs      - Show logs"
        exit 1
        ;;
esac
