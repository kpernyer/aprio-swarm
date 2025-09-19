# 🐳 Docker Setup for Aprio Swarm

This document describes how to run the Aprio Swarm document processing system using Docker and Docker Compose.

## 📋 Prerequisites

- Docker Desktop or Docker Engine
- Docker Compose
- At least 2GB of available RAM

## 🚀 Quick Start

### Development Environment

```bash
# Start the complete development environment
./scripts/docker-run.sh dev

# Or manually:
docker-compose -f docker-compose.dev.yml up --build
```

### Production Environment

```bash
# Start the production environment
./scripts/docker-run.sh prod

# Or manually:
docker-compose up --build
```

### NATS Server Only

```bash
# Start only the NATS server for local development
./scripts/docker-run.sh nats-only
```

## 🏗️ Architecture

The Docker setup includes:

- **NATS Server** - Message broker on port 4222
- **NATS Monitoring** - HTTP monitoring on port 8222
- **Document Publisher** - Publishes test documents
- **Document Subscribers** - Process documents (2 instances in prod)
- **Simple Demo** - Basic document processing demo

## 📊 Services

### NATS Server
- **Image**: `nats:2.11-alpine`
- **Ports**: 4222 (client), 8222 (monitoring)
- **Features**: JetStream enabled
- **Health Check**: HTTP endpoint monitoring

### Document Publisher
- **Purpose**: Publishes test documents to NATS
- **Behavior**: Runs once and exits
- **Subjects**: `swarm.documents.incoming`

### Document Subscribers
- **Purpose**: Process documents from NATS
- **Behavior**: Continuous processing
- **Subjects**: `swarm.documents.incoming` → `swarm.documents.results`
- **Instances**: 1 in dev, 2 in prod

### Simple Demo
- **Purpose**: Basic document processing demonstration
- **Behavior**: Runs once and exits

## 🔧 Development

### Building Images

```bash
# Build all images
docker-compose -f docker-compose.dev.yml build

# Build specific service
docker-compose -f docker-compose.dev.yml build subscriber
```

### Running Individual Services

```bash
# Run only NATS
docker-compose -f docker-compose.dev.yml up nats

# Run publisher and wait for NATS
docker-compose -f docker-compose.dev.yml up nats publisher

# Run subscriber in background
docker-compose -f docker-compose.dev.yml up -d subscriber
```

### Viewing Logs

```bash
# All services
./scripts/docker-run.sh logs

# Specific service
docker-compose -f docker-compose.dev.yml logs -f subscriber

# Last 100 lines
docker-compose -f docker-compose.dev.yml logs --tail=100
```

### Debugging

```bash
# Enter container shell
docker-compose -f docker-compose.dev.yml exec subscriber bash

# Check NATS server status
curl http://localhost:8222/varz

# Monitor NATS connections
curl http://localhost:8222/connz
```

## 🧹 Cleanup

```bash
# Stop and remove containers
./scripts/docker-run.sh clean

# Or manually:
docker-compose -f docker-compose.dev.yml down -v
docker-compose down -v
docker system prune -f
```

## 📈 Monitoring

### NATS Server Monitoring

- **HTTP Monitoring**: http://localhost:8222
- **Server Info**: http://localhost:8222/varz
- **Connections**: http://localhost:8222/connz
- **Routes**: http://localhost:8222/routez
- **Subscriptions**: http://localhost:8222/subsz

### Application Logs

```bash
# Follow all logs
docker-compose -f docker-compose.dev.yml logs -f

# Filter by service
docker-compose -f docker-compose.dev.yml logs -f | grep "subscriber"
```

## 🔍 Troubleshooting

### Common Issues

1. **Port conflicts**: Ensure ports 4222 and 8222 are available
2. **Build failures**: Check Docker has enough memory (2GB+)
3. **Connection issues**: Wait for NATS health check to pass

### Health Checks

```bash
# Check NATS health
curl -f http://localhost:8222/varz || echo "NATS not ready"

# Check container status
docker-compose -f docker-compose.dev.yml ps
```

### Performance Tuning

For production use:

1. **Increase memory limits** in docker-compose.yml
2. **Use multi-stage builds** for smaller images
3. **Enable NATS clustering** for high availability
4. **Add resource limits** and health checks

## 📝 Environment Variables

- `RUST_LOG`: Log level (debug, info, warn, error)
- `NATS_URL`: NATS server URL (default: nats://nats:4222)

## 🔄 CI/CD Integration

The Docker setup is designed for easy CI/CD integration:

```yaml
# Example GitHub Actions
- name: Build and test
  run: |
    docker-compose -f docker-compose.dev.yml up --build --abort-on-container-exit
```
