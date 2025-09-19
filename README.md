# Aprio Swarm System

High-performance Rust-based distributed worker system for the Aprio AI platform.

## Overview

The swarm system provides distributed processing capabilities optimized for:

- **Document Processing**: High-speed document analysis and indexing
- **Vector Operations**: Efficient vector database operations
- **Real-time Analysis**: Stream processing for live data
- **ML Inference**: Distributed machine learning model serving

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Swarm           │    │ Task            │    │ Worker          │
│ Coordinator     │───▶│ Scheduler       │───▶│ Pool            │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Communication   │    │ Storage         │    │ Metrics         │
│ Layer           │    │ Layer           │    │ Collection      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Quick Start

```bash
# Build the workspace
cargo build --release

# Run the visual demo (recommended first run)
cd examples/basic-swarm && cargo run

# Run document processor worker
cargo run --bin document-processor

# Run benchmarks
cargo bench

# Generate performance visualizations
python3 scripts/visualize_performance.py
```

## 🎯 Live Demo Output

When you run the basic swarm example, you'll see:

```
🚀 Aprio Swarm System
═══════════════════════════════════════
🎯 High-performance distributed worker system
📊 Processing: Documents, Vectors, ML Inference
⚡ Target: 10,000+ docs/sec, <10ms latency
═══════════════════════════════════════

👷 Worker f4cdbbf9-6e98-4bbe-8ad5-6245d7bbbbfc registered: document_processor
👷 Worker 645b0690-394e-4cee-b2f1-4be21ea4b495 registered: ml_inference

📋 Task Queue Setup
─────────────────────────────────────
📤 Submitting 5 tasks to the swarm:
  📢 Task 1: echo (Medium)
  🧮 Task 2: compute (High)
  📢 Task 3: echo (Low)
  ❌ Task 4: error (Medium)
  🧮 Task 5: compute (Critical)

⚡ Starting Task Processing
─────────────────────────────────────
👥 Workers: 2
📋 Pending Tasks: 5

🔄 Processing Tasks...

[Real-time processing logs...]

✅ Processing Complete!
═══════════════════════════════════════
⏱️  Total Processing Time: 454.00ms
📊 Tasks Processed: 5
👥 Workers Used: 2
🎯 Average Task Time: 90.8ms
═══════════════════════════════════════

🎉 Aprio Swarm System demo completed successfully!
🚀 Ready for production workloads!
```

## Performance Goals

- **Throughput**: 10,000+ documents/second processing
- **Latency**: Sub-10ms response times for simple operations
- **Scalability**: Linear scaling across worker nodes
- **Efficiency**: Minimal resource overhead

## Integration

The swarm system integrates with the Aprio Agentic Framework:

```rust
use swarm_core::prelude::*;

// High-performance document processing for agents
let result = swarm.process_document(document).await?;

// Vector indexing for intelligence agents
let index_result = swarm.index_vectors(vectors).await?;
```

## Development

See [docs/](./docs/) for detailed development guides and architecture documentation.
