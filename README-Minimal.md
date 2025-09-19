# Minimal Docker Setup

This directory contains minimal Docker configurations for the Aprio Swarm system using statically linked Rust binaries.

## 🎯 **Minimal Images**

### **FROM scratch** (Ultra-minimal)
- **Size**: ~2-5MB
- **Base**: No base image (FROM scratch)
- **Use case**: Maximum security, minimal attack surface
- **Requirements**: Fully static binary with musl

### **Distroless** (Minimal + SSL)
- **Size**: ~10-20MB  
- **Base**: `gcr.io/distroless/cc-debian12`
- **Use case**: Minimal attack surface with SSL support
- **Requirements**: Static binary with glibc

## 🚀 **Quick Start**

```bash
# Build minimal images
./scripts/build-minimal.sh

# Run minimal setup
docker compose -f docker-compose.minimal.yml up -d

# Check image sizes
docker images | grep aprio-swarm
```

## 📁 **Files**

- `Dockerfile.minimal` - FROM scratch build
- `Dockerfile.distroless` - Distroless build  
- `docker-compose.minimal.yml` - Minimal compose setup
- `scripts/build-minimal.sh` - Build script

## 🔧 **Build Process**

1. **Multi-stage build** with `rust:1.78-alpine`
2. **Static linking** with musl (`RUSTFLAGS="-C target-feature=+crt-static"`)
3. **Target**: `x86_64-unknown-linux-musl`
4. **Final stage**: FROM scratch or distroless

## 📊 **Size Comparison**

| Image Type | Base | Size | Use Case |
|------------|------|------|----------|
| FROM scratch | None | ~2-5MB | Maximum security |
| Distroless | gcr.io/distroless/cc | ~10-20MB | SSL support |
| Alpine | alpine:latest | ~50-100MB | Development |
| Debian | debian:bookworm-slim | ~100-200MB | Full features |

## ⚠️ **Notes**

- **FROM scratch** requires fully static binaries
- **Distroless** provides SSL certificates and glibc
- Both images are production-ready
- Use distroless if you need SSL/TLS support
