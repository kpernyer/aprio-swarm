//! Simple Document Demo
//!
//! This example demonstrates the clean architecture we've built:
//! 1. Core traits and types from swarm-core
//! 2. Concrete implementations from swarm-documents
//! 3. Type-safe, interface-oriented design

use swarm_core::prelude::*;
use swarm_documents::{SwarmDocumentProcessor, SwarmDocumentReader, DocumentProcessingConfig};
use swarm_documents::document_reader::DocumentReaderConfig;
use anyhow::Result;
use std::path::PathBuf;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    println!("🔄 Simple Document Demo");
    println!("═══════════════════════════════════════════════════════════");
    println!("📄 Demonstrating clean architecture with real implementations");
    println!("═══════════════════════════════════════════════════════════\n");

    // Step 1: Create a document processor using the new clean API
    println!("🔧 Step 1: Creating Document Processor");
    println!("─────────────────────────────────────────────");
    
    let config = DocumentProcessingConfig::default();
    let processor = SwarmDocumentProcessor::new(config);
    
    println!("✅ Document processor created!");
    println!("📋 Supported document types: {:?}", processor.supported_document_types());
    println!();

    // Step 2: Create a test document
    println!("📄 Step 2: Creating Test Document");
    println!("─────────────────────────────────────────────");
    
    let document = Document {
        id: uuid::Uuid::new_v4(),
        filename: "test.txt".to_string(),
        document_type: DocumentType::Text,
        content: DocumentContent::Text("This is a test document with some content for processing. It contains multiple words and sentences.".to_string()),
        metadata: std::collections::HashMap::new(),
        created_at: chrono::Utc::now(),
        size_bytes: 100,
    };
    
    println!("✅ Test document created!");
    println!("📄 Filename: {}", document.filename);
    println!("📊 Document type: {:?}", document.document_type);
    println!("📏 Size: {} bytes", document.size_bytes);
    println!();

    // Step 3: Process the document
    println!("⚙️  Step 3: Processing Document");
    println!("─────────────────────────────────────────────");
    
    let start_time = std::time::Instant::now();
    let result = processor.process_document(&document).await?;
    let processing_time = start_time.elapsed();
    
    println!("✅ Document processed successfully!");
    println!("⏱️  Processing time: {}ms", processing_time.as_millis());
    println!("📝 Extracted text: {:?}", result.extracted_text);
    println!("🌍 Language: {:?}", result.language);
    println!("🔑 Keywords: {:?}", result.keywords);
    println!("😊 Sentiment: {:?}", result.sentiment);
    println!();

    // Step 4: Create a document reader
    println!("📁 Step 4: Creating Document Reader");
    println!("─────────────────────────────────────────────");
    
    let reader_config = DocumentReaderConfig {
        watch_directories: vec![PathBuf::from("../../test-data")],
        supported_extensions: vec!["txt".to_string(), "md".to_string()],
        max_file_size: 1024 * 1024, // 1MB
        scan_interval_ms: 1000,
        batch_size: 5,
        recursive_scan: false,
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec![".*".to_string()],
    };
    
    let mut reader = SwarmDocumentReader::new(reader_config);
    
    println!("✅ Document reader created!");
    println!("📂 Watching directory: ../../test-data");
    println!("📋 Supported extensions: txt, md");
    println!();

    // Step 5: Test document reading
    println!("📖 Step 5: Testing Document Reading");
    println!("─────────────────────────────────────────────");
    
    // Try to read a document
    if let Some(doc) = reader.get_next_document().await? {
        println!("✅ Document read successfully!");
        println!("📄 Filename: {}", doc.filename);
        println!("📊 Document type: {:?}", doc.document_type);
        println!("📏 Size: {} bytes", doc.size_bytes);
        
        // Process the read document
        let result = processor.process_document(&doc).await?;
        println!("📝 Processed text length: {}", 
                 result.extracted_text.as_ref().map(|t| t.len()).unwrap_or(0));
        println!("🌍 Detected language: {:?}", result.language);
    } else {
        println!("ℹ️  No documents found in test-data directory");
    }
    println!();

    // Step 6: Display statistics
    println!("📊 Step 6: Statistics");
    println!("─────────────────────────────────────────────");
    
    let stats = reader.stats();
    println!("📈 Document Reader Stats:");
    println!("  📄 Total documents read: {}", stats.total_documents_read);
    println!("  ⚠️  Error count: {}", stats.error_count);
    println!("  ⏰ Last read time: {:?}", stats.last_read_time);
    println!();

    println!("🎉 Demo Complete!");
    println!("═══════════════════════════════════════════════════════════");
    println!("✅ Successfully demonstrated:");
    println!("   🔧 Clean trait-based architecture");
    println!("   📄 Real document processing");
    println!("   📁 File system document reading");
    println!("   🎯 Type-safe interfaces");
    println!("   ⚡ Async/await throughout");
    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}
