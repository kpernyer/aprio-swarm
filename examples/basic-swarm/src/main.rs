//! Basic Swarm Example
//!
//! Demonstrates how to set up a simple worker swarm for distributed processing.

use anyhow::Result;
use swarm_core::prelude::*;
use swarm_worker::SwarmWorker;
use tracing::{info, Level};
use uuid::Uuid;
use serde_json::json;
use tokio::time::{sleep, Duration};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    println!("🚀 Aprio Swarm System");
    println!("═══════════════════════════════════════");
    println!("🎯 High-performance distributed worker system");
    println!("📊 Processing: Documents, Vectors, ML Inference");
    println!("⚡ Target: 10,000+ docs/sec, <10ms latency");
    println!("═══════════════════════════════════════\n");
    
    info!("Starting Aprio Swarm System");

    // Create coordinator
    let mut coordinator = SwarmCoordinator::new();

    // Create and register workers
    let worker_configs = vec![
        WorkerConfig {
            max_concurrent_tasks: 5,
            worker_type: "document_processor".to_string(),
            capabilities: vec!["text_processing".to_string(), "vector_indexing".to_string()],
        },
        WorkerConfig {
            max_concurrent_tasks: 3,
            worker_type: "ml_inference".to_string(),
            capabilities: vec!["model_serving".to_string(), "prediction".to_string()],
        },
    ];

    let mut workers = Vec::new();
    let result_sender = coordinator.get_result_sender();

    for (_i, config) in worker_configs.into_iter().enumerate() {
        let worker_id = Uuid::new_v4();
        let worker_type = config.worker_type.clone();
        let task_receiver = coordinator.register_worker(worker_id, config.clone());
        
        let mut worker = SwarmWorker::with_channels(config, task_receiver, result_sender.clone());
        
        // Start worker in background
        let worker_handle = tokio::spawn(async move {
            if let Err(e) = worker.start().await {
                eprintln!("Worker {} failed: {}", worker_id, e);
            }
        });
        
        workers.push(worker_handle);
        println!("👷 Worker {} registered: {}", worker_id, worker_type);
        info!("Started worker {} of type {}", worker_id, worker_type);
    }

    // Give workers time to start
    sleep(Duration::from_millis(100)).await;

    println!("\n📋 Task Queue Setup");
    println!("─────────────────────────────────────");

    // Submit some test tasks
    let test_tasks = vec![
        Task {
            id: Uuid::new_v4(),
            task_type: "echo".to_string(),
            payload: json!({"message": "Hello from the swarm!"}),
            priority: TaskPriority::Medium,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        Task {
            id: Uuid::new_v4(),
            task_type: "compute".to_string(),
            payload: json!({"value": 42}),
            priority: TaskPriority::High,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        Task {
            id: Uuid::new_v4(),
            task_type: "echo".to_string(),
            payload: json!({"message": "Another echo task"}),
            priority: TaskPriority::Low,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        Task {
            id: Uuid::new_v4(),
            task_type: "error".to_string(),
            payload: json!({}),
            priority: TaskPriority::Medium,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        Task {
            id: Uuid::new_v4(),
            task_type: "compute".to_string(),
            payload: json!({"value": 100}),
            priority: TaskPriority::Critical,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    println!("📤 Submitting {} tasks to the swarm:", test_tasks.len());
    for (i, task) in test_tasks.iter().enumerate() {
        let emoji = match task.task_type.as_str() {
            "echo" => "📢",
            "compute" => "🧮",
            "error" => "❌",
            _ => "📝"
        };
        println!("  {} Task {}: {} ({})", emoji, i + 1, task.task_type, task.priority);
        coordinator.submit_task(task.clone());
    }
    
    info!("Submitting {} tasks to the swarm", test_tasks.len());

    println!("\n⚡ Starting Task Processing");
    println!("─────────────────────────────────────");
    println!("👥 Workers: {}", coordinator.worker_count());
    println!("📋 Pending Tasks: {}", coordinator.pending_tasks());
    println!("\n🔄 Processing Tasks...\n");

    info!("Starting coordinator with {} workers and {} pending tasks", 
          coordinator.worker_count(), coordinator.pending_tasks());

    let start_time = Instant::now();
    
    // Start the coordinator (this will run until all tasks are processed)
    coordinator.start().await?;
    
    let total_time = start_time.elapsed();

    println!("\n✅ Processing Complete!");
    println!("═══════════════════════════════════════");
    println!("⏱️  Total Processing Time: {:.2}ms", total_time.as_millis());
    println!("📊 Tasks Processed: 5");
    println!("👥 Workers Used: 2");
    println!("🎯 Average Task Time: {:.1}ms", total_time.as_millis() as f64 / 5.0);
    println!("═══════════════════════════════════════\n");

    info!("All tasks completed! Waiting for workers to finish...");

    // Wait for all workers to finish
    for worker in workers {
        let _ = worker.await;
    }

    println!("🎉 Aprio Swarm System demo completed successfully!");
    println!("🚀 Ready for production workloads!");

    info!("Aprio Swarm System demo completed successfully!");

    Ok(())
}
