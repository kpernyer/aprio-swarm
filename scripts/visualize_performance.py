#!/usr/bin/env python3
"""
Performance Visualization for Living Twin Swarm System
Creates visual charts showing system performance metrics
"""

import matplotlib.pyplot as plt
import numpy as np
from datetime import datetime
import json

def create_performance_charts():
    """Create performance visualization charts"""
    
    # Sample performance data (you can replace with real data)
    task_types = ['Echo', 'Compute', 'Error']
    processing_times = [102, 200, 50]  # ms
    success_rates = [100, 100, 0]  # percentage
    throughput = [9.8, 5.0, 20.0]  # tasks/second
    
    # Create figure with subplots
    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 10))
    fig.suptitle('Aprio Swarm System Performance Metrics', fontsize=16, fontweight='bold')
    
    # 1. Processing Time by Task Type
    colors = ['#2E8B57', '#4169E1', '#DC143C']
    bars1 = ax1.bar(task_types, processing_times, color=colors, alpha=0.7)
    ax1.set_title('Processing Time by Task Type', fontweight='bold')
    ax1.set_ylabel('Time (ms)')
    ax1.set_xlabel('Task Type')
    
    # Add value labels on bars
    for bar, time in zip(bars1, processing_times):
        height = bar.get_height()
        ax1.text(bar.get_x() + bar.get_width()/2., height + 5,
                f'{time}ms', ha='center', va='bottom', fontweight='bold')
    
    # 2. Success Rate
    bars2 = ax2.bar(task_types, success_rates, color=colors, alpha=0.7)
    ax2.set_title('Success Rate by Task Type', fontweight='bold')
    ax2.set_ylabel('Success Rate (%)')
    ax2.set_xlabel('Task Type')
    ax2.set_ylim(0, 110)
    
    for bar, rate in zip(bars2, success_rates):
        height = bar.get_height()
        ax2.text(bar.get_x() + bar.get_width()/2., height + 2,
                f'{rate}%', ha='center', va='bottom', fontweight='bold')
    
    # 3. Throughput
    bars3 = ax3.bar(task_types, throughput, color=colors, alpha=0.7)
    ax3.set_title('Throughput by Task Type', fontweight='bold')
    ax3.set_ylabel('Tasks/Second')
    ax3.set_xlabel('Task Type')
    
    for bar, rate in zip(bars3, throughput):
        height = bar.get_height()
        ax3.text(bar.get_x() + bar.get_width()/2., height + 0.5,
                f'{rate}/s', ha='center', va='bottom', fontweight='bold')
    
    # 4. System Overview
    ax4.axis('off')
    overview_text = """
    🚀 Aprio Swarm System
    
    📊 Performance Summary:
    • Total Tasks Processed: 5
    • Workers Active: 2
    • Average Processing Time: 118ms
    • System Uptime: 100%
    • Error Rate: 20% (1/5 tasks)
    
    🎯 Key Metrics:
    • Echo Tasks: 102ms avg
    • Compute Tasks: 200ms avg
    • Error Handling: 50ms avg
    
    ⚡ Performance Goals:
    • Target: 10,000+ docs/sec
    • Latency: <10ms
    • Scalability: Linear
    """
    
    ax4.text(0.1, 0.9, overview_text, transform=ax4.transAxes, fontsize=11,
             verticalalignment='top', fontfamily='monospace',
             bbox=dict(boxstyle="round,pad=0.3", facecolor="lightblue", alpha=0.5))
    
    plt.tight_layout()
    plt.savefig('swarm_performance.png', dpi=300, bbox_inches='tight')
    plt.show()
    
    print("📊 Performance visualization saved as 'swarm_performance.png'")

def create_architecture_diagram():
    """Create a simple ASCII architecture diagram"""
    
    diagram = """
    ┌─────────────────────────────────────────────────────────────┐
    │                Living Twin Swarm System                     │
    └─────────────────────────────────────────────────────────────┘
    
    ┌─────────────┐    ┌─────────────────┐    ┌─────────────────┐
    │   Client    │───▶│  Coordinator    │───▶│   Task Queue    │
    │             │    │                 │    │                 │
    └─────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
    ┌─────────────────────────────────────────────────────────────┐
    │                    Worker Pool                              │
    │                                                             │
    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
    │  │ Document    │  │ ML          │  │ Vector      │        │
    │  │ Processor   │  │ Inference   │  │ Indexer     │        │
    │  │ Worker 1    │  │ Worker 2    │  │ Worker N    │        │
    │  └─────────────┘  └─────────────┘  └─────────────┘        │
    └─────────────────────────────────────────────────────────────┘
                                │
                                ▼
    ┌─────────────────────────────────────────────────────────────┐
    │                  Task Processing                            │
    │                                                             │
    │  📢 Echo Tasks (100ms)    🧮 Compute Tasks (200ms)        │
    │  ❌ Error Tasks (50ms)    📝 Custom Tasks (varies)        │
    └─────────────────────────────────────────────────────────────┘
                                │
                                ▼
    ┌─────────────────────────────────────────────────────────────┐
    │                  Result Collection                          │
    │                                                             │
    │  ✅ Success: 4/5 tasks    ❌ Failed: 1/5 tasks            │
    │  ⏱️  Avg Time: 118ms      📊 Throughput: 42 tasks/sec     │
    └─────────────────────────────────────────────────────────────┘
    """
    
    print(diagram)

if __name__ == "__main__":
    print("🎨 Creating Aprio Swarm System Visualizations...")
    create_architecture_diagram()
    create_performance_charts()
    print("✨ Visualizations complete!")
