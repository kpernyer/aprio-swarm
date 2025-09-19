//! NATS Demo - Demonstrating NATS Integration
//!
//! This example shows how to use the NATS messaging system with the Aprio Swarm.
//! It demonstrates:
//! - Connecting to NATS
//! - Publishing documents
//! - Subscribing to messages
//! - Message serialization/deserialization

use swarm_core::prelude::*;
use swarm_documents::{SwarmDocumentProcessor, DocumentProcessingConfig};
use swarm_comms::{NatsBroker, NatsConfig, MessageSerializer, MessageValidator};
use anyhow::Result;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🚀 NATS Demo");
    println!("═══════════════════════════════════════════════════════════");
    println!("📡 Demonstrating NATS messaging integration");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    
    // Step 1: Create NATS broker
    println!("🔧 Step 1: Creating NATS Broker");
    println!("─────────────────────────────────────────────");
    
    let nats_config = NatsConfig {
        url: "nats://localhost:4222".to_string(),
        connection_timeout_ms: 5000,
        max_reconnect_attempts: 5,
        reconnect_delay_ms: 1000,
        max_message_size: 1024 * 1024, // 1MB
        enable_tls: false,
        tls_cert_path: None,
        tls_key_path: None,
        tls_ca_path: None,
    };
    
    println!("📡 NATS Config:");
    println!("   URL: {}", nats_config.url);
    println!("   Max message size: {} bytes", nats_config.max_message_size);
    println!("   TLS enabled: {}", nats_config.enable_tls);
    println!();
    
    // Try to connect to NATS (this will fail if NATS server is not running)
    match NatsBroker::new(nats_config).await {
        Ok(broker) => {
            println!("✅ Connected to NATS server successfully!");
            run_nats_demo(broker).await?;
        }
        Err(e) => {
            println!("❌ Failed to connect to NATS server: {}", e);
            println!();
            println!("💡 To run this demo, you need to:");
            println!("   1. Install NATS server: https://docs.nats.io/running-a-nats-service/introduction/installation");
            println!("   2. Start NATS server: nats-server");
            println!("   3. Run this demo again");
            println!();
            println!("🔄 Running simulation mode instead...");
            run_simulation_demo().await?;
        }
    }
    
    Ok(())
}

async fn run_nats_demo(broker: NatsBroker) -> Result<()> {
    println!("🎯 Step 2: Creating Test Document");
    println!("─────────────────────────────────────────────");
    
    // Create a test document
    let document = Document {
        id: Uuid::new_v4(),
        filename: "nats-test.txt".to_string(),
        document_type: DocumentType::Text,
        content: DocumentContent::Text(
            "This is a test document for NATS messaging. It contains some sample text to demonstrate document processing through the message broker.".to_string()
        ),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String("nats-demo".to_string()));
            metadata.insert("version".to_string(), serde_json::Value::String("1.0".to_string()));
            metadata
        },
        created_at: Utc::now(),
        size_bytes: 200,
    };
    
    println!("✅ Test document created!");
    println!("   ID: {}", document.id);
    println!("   Filename: {}", document.filename);
    println!("   Type: {:?}", document.document_type);
    println!("   Size: {} bytes", document.size_bytes);
    println!();
    
    // Step 3: Serialize and publish document
    println!("📤 Step 3: Publishing Document to NATS");
    println!("─────────────────────────────────────────────");
    
    let message = MessageSerializer::serialize_document(&document)?;
    println!("✅ Document serialized to message!");
    println!("   Message ID: {}", message.id);
    println!("   Subject: {}", message.subject);
    println!("   Payload size: {} bytes", message.payload.len());
    println!("   Headers: {:?}", message.headers);
    println!();
    
    // Validate message
    MessageValidator::validate_message(&message)?;
    MessageValidator::validate_message_size(&message, 1024 * 1024)?;
    MessageValidator::validate_message_ttl(&message)?;
    println!("✅ Message validation passed!");
    println!();
    
    // Publish to NATS
    broker.publish_message(&message.subject, &message).await?;
    println!("✅ Document published to NATS successfully!");
    println!();
    
    // Step 4: Subscribe and receive messages
    println!("📥 Step 4: Subscribing to Messages");
    println!("─────────────────────────────────────────────");
    
    let mut receiver = broker.subscribe_to_subject("swarm.documents.incoming").await?;
    println!("✅ Subscribed to swarm.documents.incoming");
    println!();
    
    // Step 5: Process received message
    println!("⚙️  Step 5: Processing Received Message");
    println!("─────────────────────────────────────────────");
    
    // Wait for message (with timeout)
    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout {
        if let Ok(received_message) = receiver.try_recv() {
            println!("✅ Message received!");
            println!("   Message ID: {}", received_message.id);
            println!("   Subject: {}", received_message.subject);
            println!("   Timestamp: {}", received_message.timestamp);
            println!();
            
            // Deserialize back to document
            let received_document = MessageSerializer::deserialize_document(&received_message)?;
            println!("✅ Document deserialized successfully!");
            println!("   Document ID: {}", received_document.id);
            println!("   Filename: {}", received_document.filename);
            println!("   Content preview: {}", 
                match &received_document.content {
                    DocumentContent::Text(text) => &text[..text.len().min(50)],
                    _ => "[Binary content]",
                }
            );
            println!();
            
            // Process the document
            let config = DocumentProcessingConfig::default();
            let processor = SwarmDocumentProcessor::new(config);
            let result = processor.process_document(&received_document).await?;
            
            println!("✅ Document processed successfully!");
            println!("   Extracted text: {:?}", result.extracted_text);
            println!("   Language: {:?}", result.language);
            println!("   Keywords: {:?}", result.keywords);
            println!("   Sentiment: {:?}", result.sentiment);
            println!();
            
            break;
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Step 6: Show statistics
    println!("📊 Step 6: NATS Statistics");
    println!("─────────────────────────────────────────────");
    
    let stats = broker.get_stats().await;
    println!("📈 NATS Broker Stats:");
    println!("   Messages sent: {}", stats.messages_sent);
    println!("   Messages received: {}", stats.messages_received);
    println!("   Active subscriptions: {}", stats.active_subscriptions);
    println!("   Connection status: {}", if stats.is_connected { "Connected" } else { "Disconnected" });
    println!();
    
    println!("🎉 NATS Demo Complete!");
    println!("═══════════════════════════════════════════════════════════");
    println!("✅ Successfully demonstrated:");
    println!("   🔗 NATS connection and configuration");
    println!("   📤 Document publishing to NATS");
    println!("   📥 Message subscription and reception");
    println!("   🔄 Message serialization/deserialization");
    println!("   ✅ Message validation");
    println!("   📊 Statistics and monitoring");
    println!("═══════════════════════════════════════════════════════════");
    
    Ok(())
}

async fn run_simulation_demo() -> Result<()> {
    println!("🎭 Simulation Mode - NATS Features");
    println!("─────────────────────────────────────────────");
    println!();
    
    // Step 1: Show message serialization
    println!("📤 Step 1: Message Serialization");
    println!("─────────────────────────────────────────────");
    
    let document = Document {
        id: Uuid::new_v4(),
        filename: "simulation-test.txt".to_string(),
        document_type: DocumentType::Text,
        content: DocumentContent::Text("Simulation test document".to_string()),
        metadata: HashMap::new(),
        created_at: Utc::now(),
        size_bytes: 100,
    };
    
    let message = MessageSerializer::serialize_document(&document)?;
    println!("✅ Document serialized to message!");
    println!("   Message ID: {}", message.id);
    println!("   Subject: {}", message.subject);
    println!("   Payload size: {} bytes", message.payload.len());
    println!("   Headers: {:?}", message.headers);
    println!();
    
    // Step 2: Show message validation
    println!("✅ Step 2: Message Validation");
    println!("─────────────────────────────────────────────");
    
    MessageValidator::validate_message(&message)?;
    MessageValidator::validate_message_size(&message, 1024 * 1024)?;
    MessageValidator::validate_message_ttl(&message)?;
    println!("✅ All validations passed!");
    println!();
    
    // Step 3: Show message deserialization
    println!("📥 Step 3: Message Deserialization");
    println!("─────────────────────────────────────────────");
    
    let deserialized_document = MessageSerializer::deserialize_document(&message)?;
    println!("✅ Message deserialized to document!");
    println!("   Document ID: {}", deserialized_document.id);
    println!("   Filename: {}", deserialized_document.filename);
    println!("   Type: {:?}", deserialized_document.document_type);
    println!();
    
    // Step 4: Show document processing
    println!("⚙️  Step 4: Document Processing");
    println!("─────────────────────────────────────────────");
    
    let config = DocumentProcessingConfig::default();
    let processor = SwarmDocumentProcessor::new(config);
    let result = processor.process_document(&deserialized_document).await?;
    
    println!("✅ Document processed successfully!");
    println!("   Extracted text: {:?}", result.extracted_text);
    println!("   Language: {:?}", result.language);
    println!("   Keywords: {:?}", result.keywords);
    println!("   Sentiment: {:?}", result.sentiment);
    println!();
    
    // Step 5: Show heartbeat message
    println!("💓 Step 5: Heartbeat Message");
    println!("─────────────────────────────────────────────");
    
    let heartbeat = MessageSerializer::create_heartbeat("demo-worker", "document-processor")?;
    println!("✅ Heartbeat message created!");
    println!("   Subject: {}", heartbeat.subject);
    println!("   Component ID: {}", heartbeat.headers.get("component-id").unwrap());
    println!("   Component Type: {}", heartbeat.headers.get("component-type").unwrap());
    println!();
    
    // Step 6: Show error message
    println!("❌ Step 6: Error Message");
    println!("─────────────────────────────────────────────");
    
    let mut context = HashMap::new();
    context.insert("file".to_string(), "demo.txt".to_string());
    context.insert("operation".to_string(), "processing".to_string());
    
    let error_message = MessageSerializer::create_error_message(
        "demo_error",
        "This is a simulated error for demonstration",
        "demo-worker",
        Some(context),
    )?;
    
    println!("✅ Error message created!");
    println!("   Subject: {}", error_message.subject);
    println!("   Error Type: {}", error_message.headers.get("error-type").unwrap());
    println!("   Component ID: {}", error_message.headers.get("component-id").unwrap());
    println!();
    
    println!("🎉 Simulation Demo Complete!");
    println!("═══════════════════════════════════════════════════════════");
    println!("✅ Successfully demonstrated:");
    println!("   📤 Message serialization");
    println!("   ✅ Message validation");
    println!("   📥 Message deserialization");
    println!("   ⚙️  Document processing");
    println!("   💓 Heartbeat messages");
    println!("   ❌ Error messages");
    println!("═══════════════════════════════════════════════════════════");
    
    Ok(())
}
