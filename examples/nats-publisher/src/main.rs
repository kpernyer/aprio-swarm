//! NATS Document Publisher
//!
//! Publishes documents to NATS for processing by workers.

use swarm_core::prelude::*;
use swarm_comms::{NatsBroker, NatsConfig, MessageSerializer};
use anyhow::Result;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("📤 NATS Document Publisher");
    println!("═══════════════════════════════════════════════════════════");
    
    // Connect to NATS
    let config = NatsConfig::default();
    let broker = NatsBroker::new(config).await?;
    println!("✅ Connected to NATS");
    
    // Create test documents
    let documents = vec![
        create_document("report.pdf", DocumentType::Pdf, "PDF content here"),
        create_document("notes.txt", DocumentType::Text, "This is a text document with some content"),
        create_document("data.docx", DocumentType::Word, "Word document content"),
    ];
    
    println!("📄 Publishing {} documents...", documents.len());
    
    for (i, document) in documents.iter().enumerate() {
        let message = MessageSerializer::serialize_document(document)?;
        broker.publish_message("swarm.documents.incoming", &message).await?;
        
        println!("✅ Published document {}: {}", i + 1, document.filename);
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("🎉 All documents published successfully!");
    Ok(())
}

fn create_document(filename: &str, doc_type: DocumentType, content: &str) -> Document {
    Document {
        id: Uuid::new_v4(),
        filename: filename.to_string(),
        document_type: doc_type,
        content: DocumentContent::Text(content.to_string()),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String("publisher".to_string()));
            metadata.insert("timestamp".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
            metadata
        },
        created_at: Utc::now(),
        size_bytes: content.len(),
    }
}
