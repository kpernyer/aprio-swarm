//! NATS Document Subscriber
//!
//! Subscribes to document messages and processes them.

use swarm_core::prelude::*;
use swarm_core::TaskResultData;
use swarm_comms::{NatsBroker, NatsConfig, MessageSerializer};
use swarm_documents::{SwarmDocumentProcessor, DocumentProcessingConfig};
use anyhow::Result;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let worker_id = Uuid::new_v4();
    println!("📥 NATS Document Subscriber - Worker {}", worker_id);
    println!("═══════════════════════════════════════════════════════════");
    
    // Connect to NATS
    let config = NatsConfig::default();
    let broker = NatsBroker::new(config).await?;
    println!("✅ Connected to NATS");
    
    // Create document processor
    let processor_config = DocumentProcessingConfig::default();
    let processor = SwarmDocumentProcessor::new(processor_config);
    println!("✅ Document processor ready");
    
    // Subscribe to documents
    let mut receiver = broker.subscribe_to_subject("swarm.documents.incoming").await?;
    println!("✅ Subscribed to swarm.documents.incoming");
    println!("🔄 Waiting for documents...");
    
    let mut processed_count = 0;
    
    loop {
        if let Ok(document_message) = receiver.try_recv() {
            processed_count += 1;
            
            println!("📄 Received document #{}", processed_count);
            println!("   Message ID: {}", document_message.id);
            println!("   Subject: {}", document_message.subject);
            
            // Deserialize document
            let document = MessageSerializer::deserialize_document(&document_message)?;
            println!("   Document: {} ({:?})", document.filename, document.document_type);
            
            // Process document
            let result = processor.process_document(&document).await?;
            println!("   ✅ Processed: language: {:?}, keywords: {:?}", 
                result.language, result.keywords);
            
            // Publish result
            let result_message = MessageSerializer::serialize_task_result(&TaskResult {
                task_id: Uuid::new_v4(),
                status: TaskStatus::Completed,
                result: Some(TaskResultData::DocumentProcessing(result)),
                error: None,
                processing_time_ms: 100,
                completed_at: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            })?;
            
            broker.publish_message("swarm.documents.results", &result_message).await?;
            println!("   📤 Published result to swarm.documents.results");
            println!();
        }
        
        sleep(Duration::from_millis(100)).await;
    }
}
