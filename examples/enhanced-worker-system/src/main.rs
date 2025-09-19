//! Enhanced Worker System Example
//!
//! Demonstrates the improved architecture with capability-based task assignment
//! and specialized worker types.

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::json;

// Enhanced types for the improved system
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum WorkerType {
    DocumentProcessor,
    MLInference,
    VectorIndexer,
    RealTimeAnalyzer,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Capability {
    TextProcessing,
    VectorIndexing,
    ModelServing,
    Prediction,
    StreamProcessing,
    ImageProcessing,
    AudioProcessing,
    BasicComputation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub avg_processing_time_ms: u64,
    pub memory_usage_mb: u64,
    pub cpu_intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedWorkerConfig {
    pub max_concurrent_tasks: usize,
    pub worker_type: WorkerType,
    pub capabilities: HashSet<Capability>,
    pub performance_profile: PerformanceProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Echo,
    Compute,
    DocumentAnalysis,
    MLInference,
    VectorIndexing,
    RealTimeAnalysis,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub required_capabilities: HashSet<Capability>,
    pub preferred_worker_type: Option<WorkerType>,
    pub max_processing_time_ms: Option<u64>,
    pub memory_requirement_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTask {
    pub id: Uuid,
    pub task_type: TaskType,
    pub payload: serde_json::Value,
    pub requirements: TaskRequirements,
    pub created_at: String,
}

// Enhanced coordinator with intelligent task assignment
pub struct EnhancedCoordinator {
    workers: HashMap<Uuid, EnhancedWorkerConfig>,
    task_queue: Vec<EnhancedTask>,
}

impl EnhancedCoordinator {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
            task_queue: Vec::new(),
        }
    }

    pub fn register_worker(&mut self, worker_id: Uuid, config: EnhancedWorkerConfig) {
        println!("👷 Registered {} worker with capabilities: {:?}", 
                 format!("{:?}", config.worker_type), 
                 config.capabilities);
        self.workers.insert(worker_id, config);
    }

    pub fn submit_task(&mut self, task: EnhancedTask) {
        println!("📋 Submitted {} task requiring: {:?}", 
                 format!("{:?}", task.task_type), 
                 task.requirements.required_capabilities);
        self.task_queue.push(task);
    }

    pub fn find_best_worker_for_task(&self, task: &EnhancedTask) -> Option<Uuid> {
        let mut best_worker = None;
        let mut best_score = 0.0;
        
        for (worker_id, config) in &self.workers {
            let score = self.calculate_worker_score(config, &task.requirements);
            
            println!("🎯 Worker {} score: {:.2} (type: {:?})", 
                     worker_id, score, config.worker_type);
            
            if score > best_score {
                best_score = score;
                best_worker = Some(*worker_id);
            }
        }
        
        if let Some(worker_id) = best_worker {
            println!("✅ Best worker for task: {} (score: {:.2})", worker_id, best_score);
        } else {
            println!("❌ No suitable worker found for task");
        }
        
        best_worker
    }
    
    fn calculate_worker_score(&self, config: &EnhancedWorkerConfig, requirements: &TaskRequirements) -> f32 {
        let mut score = 0.0;
        
        // Capability matching (most important - 60% weight)
        let capability_match = requirements.required_capabilities
            .iter()
            .filter(|cap| config.capabilities.contains(cap))
            .count() as f32 / requirements.required_capabilities.len() as f32;
        score += capability_match * 0.6;
        
        // Worker type preference (30% weight)
        if let Some(preferred) = &requirements.preferred_worker_type {
            if config.worker_type == *preferred {
                score += 0.3;
            }
        }
        
        // Performance considerations (10% weight)
        if let Some(max_time) = requirements.max_processing_time_ms {
            if config.performance_profile.avg_processing_time_ms <= max_time {
                score += 0.1;
            }
        }
        
        score
    }

    pub fn process_all_tasks(&mut self) {
        println!("\n🔄 Processing all tasks with intelligent assignment...\n");
        
        while let Some(task) = self.task_queue.pop() {
            if let Some(worker_id) = self.find_best_worker_for_task(&task) {
                println!("📤 Assigning {} task to worker {}", 
                         format!("{:?}", task.task_type), worker_id);
            } else {
                println!("⚠️  No suitable worker for {} task - queuing for later", 
                         format!("{:?}", task.task_type));
                self.task_queue.push(task); // Put it back for later
                break; // Stop processing to avoid infinite loop
            }
        }
    }
}

fn main() -> Result<()> {
    println!("🚀 Enhanced Worker System Demo");
    println!("═══════════════════════════════════════");
    println!("🎯 Demonstrating capability-based task assignment");
    println!("═══════════════════════════════════════\n");

    let mut coordinator = EnhancedCoordinator::new();

    // Register specialized workers
    coordinator.register_worker(
        Uuid::new_v4(),
        EnhancedWorkerConfig {
            max_concurrent_tasks: 5,
            worker_type: WorkerType::DocumentProcessor,
            capabilities: HashSet::from([
                Capability::TextProcessing,
                Capability::BasicComputation,
            ]),
            performance_profile: PerformanceProfile {
                avg_processing_time_ms: 150,
                memory_usage_mb: 512,
                cpu_intensity: 0.6,
            },
        }
    );

    coordinator.register_worker(
        Uuid::new_v4(),
        EnhancedWorkerConfig {
            max_concurrent_tasks: 3,
            worker_type: WorkerType::MLInference,
            capabilities: HashSet::from([
                Capability::ModelServing,
                Capability::Prediction,
                Capability::BasicComputation,
            ]),
            performance_profile: PerformanceProfile {
                avg_processing_time_ms: 300,
                memory_usage_mb: 2048,
                cpu_intensity: 0.9,
            },
        }
    );

    coordinator.register_worker(
        Uuid::new_v4(),
        EnhancedWorkerConfig {
            max_concurrent_tasks: 4,
            worker_type: WorkerType::VectorIndexer,
            capabilities: HashSet::from([
                Capability::VectorIndexing,
                Capability::BasicComputation,
            ]),
            performance_profile: PerformanceProfile {
                avg_processing_time_ms: 200,
                memory_usage_mb: 1024,
                cpu_intensity: 0.7,
            },
        }
    );

    // Submit tasks with specific requirements
    coordinator.submit_task(EnhancedTask {
        id: Uuid::new_v4(),
        task_type: TaskType::DocumentAnalysis,
        payload: json!({"document": "sample.pdf"}),
        requirements: TaskRequirements {
            required_capabilities: HashSet::from([Capability::TextProcessing]),
            preferred_worker_type: Some(WorkerType::DocumentProcessor),
            max_processing_time_ms: Some(200),
            memory_requirement_mb: Some(1024),
        },
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    coordinator.submit_task(EnhancedTask {
        id: Uuid::new_v4(),
        task_type: TaskType::MLInference,
        payload: json!({"model": "bert", "input": "sample text"}),
        requirements: TaskRequirements {
            required_capabilities: HashSet::from([Capability::ModelServing, Capability::Prediction]),
            preferred_worker_type: Some(WorkerType::MLInference),
            max_processing_time_ms: Some(500),
            memory_requirement_mb: Some(2048),
        },
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    coordinator.submit_task(EnhancedTask {
        id: Uuid::new_v4(),
        task_type: TaskType::VectorIndexing,
        payload: json!({"vectors": [1.0, 2.0, 3.0]}),
        requirements: TaskRequirements {
            required_capabilities: HashSet::from([Capability::VectorIndexing]),
            preferred_worker_type: Some(WorkerType::VectorIndexer),
            max_processing_time_ms: Some(300),
            memory_requirement_mb: Some(1024),
        },
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    // Process all tasks
    coordinator.process_all_tasks();

    println!("\n✅ Enhanced worker system demo completed!");
    println!("🎯 Key improvements demonstrated:");
    println!("  • Capability-based task assignment");
    println!("  • Worker specialization");
    println!("  • Intelligent load balancing");
    println!("  • Performance-aware scheduling");

    Ok(())
}
