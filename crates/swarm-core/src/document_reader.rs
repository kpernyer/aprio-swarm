//! Document Reader Component
//!
//! Reads documents from directories and publishes them to NATS for processing.
//! This is the entry point of the document processing pipeline.

use crate::document_worker::{Document, DocumentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use serde_json::json;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReaderConfig {
    pub watch_directories: Vec<PathBuf>,
    pub supported_extensions: Vec<String>,
    pub batch_size: usize,
    pub scan_interval_ms: u64,
    pub nats_subject: String,
}

impl Default for DocumentReaderConfig {
    fn default() -> Self {
        Self {
            watch_directories: vec![PathBuf::from("./test-data")],
            supported_extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "pdf".to_string(),
                "docx".to_string(),
                "html".to_string(),
            ],
            batch_size: 10,
            scan_interval_ms: 1000,
            nats_subject: "swarm.documents.incoming".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReader {
    config: DocumentReaderConfig,
    processed_files: HashMap<PathBuf, DateTime<Utc>>,
}

impl DocumentReader {
    pub fn new(config: DocumentReaderConfig) -> Self {
        Self {
            config,
            processed_files: HashMap::new(),
        }
    }

    /// Start the document reader - scans directories and publishes documents
    pub async fn start(&mut self) -> Result<()> {
        println!("📁 Document Reader starting...");
        println!("📂 Watching directories: {:?}", self.config.watch_directories);
        println!("📋 Supported extensions: {:?}", self.config.supported_extensions);
        println!("📦 Batch size: {}", self.config.batch_size);
        println!("⏱️  Scan interval: {}ms", self.config.scan_interval_ms);
        println!("📡 NATS subject: {}", self.config.nats_subject);
        println!();

        loop {
            self.scan_directories().await?;
            sleep(Duration::from_millis(self.config.scan_interval_ms)).await;
        }
    }

    /// Scan all configured directories for new documents
    async fn scan_directories(&mut self) -> Result<()> {
        let directories = self.config.watch_directories.clone();
        for directory in directories {
            if directory.exists() {
                self.scan_directory(&directory).await?;
            } else {
                println!("⚠️  Directory does not exist: {:?}", directory);
            }
        }
        Ok(())
    }

    /// Scan a single directory for documents
    async fn scan_directory(&mut self, directory: &Path) -> Result<()> {
        let mut entries = async_fs::read_dir(directory).await?;
        let mut documents = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() {
                if self.is_supported_file(&path) {
                    if self.is_new_or_modified_file(&path).await? {
                        match self.read_document(&path).await {
                            Ok(document) => {
                                documents.push(document);
                                println!("📄 Found document: {}", path.display());
                            }
                            Err(e) => {
                                println!("❌ Failed to read document {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        // Process documents in batches
        if !documents.is_empty() {
            self.publish_documents(documents).await?;
        }

        Ok(())
    }

    /// Check if file has a supported extension
    fn is_supported_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.config.supported_extensions.contains(&ext_str.to_lowercase());
            }
        }
        false
    }

    /// Check if file is new or has been modified since last scan
    async fn is_new_or_modified_file(&self, path: &Path) -> Result<bool> {
        let metadata = async_fs::metadata(path).await?;
        let modified_time = metadata.modified()?;
        let modified_datetime: DateTime<Utc> = modified_time.into();

        if let Some(last_processed) = self.processed_files.get(path) {
            Ok(modified_datetime > *last_processed)
        } else {
            Ok(true) // New file
        }
    }

    /// Read a document from the file system
    async fn read_document(&mut self, path: &Path) -> Result<Document> {
        let filename = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Read file content
        let content = async_fs::read_to_string(path).await?;
        
        // Get file metadata
        let metadata = async_fs::metadata(path).await?;
        let size_bytes = metadata.len() as usize;
        let modified_time = metadata.modified()?;
        let modified_datetime: DateTime<Utc> = modified_time.into();

        // Create document
        let mut document = Document::new(filename, content);
        document.size_bytes = size_bytes;

        // Add file system metadata
        let mut file_metadata = HashMap::new();
        file_metadata.insert("file_path".to_string(), json!(path.to_string_lossy()));
        file_metadata.insert("file_size".to_string(), json!(size_bytes));
        file_metadata.insert("modified_time".to_string(), json!(modified_datetime.to_rfc3339()));
        file_metadata.insert("file_extension".to_string(), json!(
            path.extension().and_then(|ext| ext.to_str()).unwrap_or("unknown")
        ));
        
        document = document.with_metadata(file_metadata);

        // Update processed files tracking
        self.processed_files.insert(path.to_path_buf(), modified_datetime);

        Ok(document)
    }

    /// Publish documents to NATS (simulated for now)
    async fn publish_documents(&self, documents: Vec<Document>) -> Result<()> {
        println!("📤 Publishing {} documents to NATS subject: {}", 
                 documents.len(), self.config.nats_subject);
        
        for (i, document) in documents.iter().enumerate() {
            println!("  📄 Document {}: {} ({} bytes, {:?})", 
                     i + 1, 
                     document.filename, 
                     document.size_bytes,
                     document.document_type);
            
            // In a real implementation, this would publish to NATS:
            // nats_client.publish(&self.config.nats_subject, &document).await?;
            
            // For now, just simulate the publishing
            self.simulate_document_processing(document).await?;
        }
        
        println!("✅ All documents published successfully\n");
        Ok(())
    }

    /// Simulate document processing (replace with real NATS publishing)
    async fn simulate_document_processing(&self, document: &Document) -> Result<()> {
        // Simulate processing delay
        sleep(Duration::from_millis(10)).await;
        
        // Simulate task generation based on document type
        let tasks = self.generate_processing_tasks(document);
        println!("    📋 Generated {} processing tasks", tasks.len());
        
        for task in tasks {
            println!("      🎯 Task: {} (estimated: {}ms)", 
                     task.task_type, task.estimated_duration_ms);
        }
        
        Ok(())
    }

    /// Generate processing tasks for a document
    fn generate_processing_tasks(&self, document: &Document) -> Vec<ProcessingTask> {
        let mut tasks = Vec::new();
        
        // Always extract text
        tasks.push(ProcessingTask {
            id: Uuid::new_v4(),
            task_type: "text_extraction".to_string(),
            document_id: document.id,
            estimated_duration_ms: document.document_type.estimated_processing_time_ms(),
            priority: "high".to_string(),
        });

        // Add type-specific tasks
        match document.document_type {
            DocumentType::Pdf => {
                tasks.push(ProcessingTask {
                    id: Uuid::new_v4(),
                    task_type: "page_counting".to_string(),
                    document_id: document.id,
                    estimated_duration_ms: 50,
                    priority: "medium".to_string(),
                });
            }
            DocumentType::Word => {
                tasks.push(ProcessingTask {
                    id: Uuid::new_v4(),
                    task_type: "formatting_analysis".to_string(),
                    document_id: document.id,
                    estimated_duration_ms: 100,
                    priority: "medium".to_string(),
                });
            }
            DocumentType::Html => {
                tasks.push(ProcessingTask {
                    id: Uuid::new_v4(),
                    task_type: "link_extraction".to_string(),
                    document_id: document.id,
                    estimated_duration_ms: 75,
                    priority: "low".to_string(),
                });
            }
            _ => {}
        }

        // Common tasks for all document types
        tasks.push(ProcessingTask {
            id: Uuid::new_v4(),
            task_type: "language_detection".to_string(),
            document_id: document.id,
            estimated_duration_ms: 25,
            priority: "medium".to_string(),
        });

        tasks.push(ProcessingTask {
            id: Uuid::new_v4(),
            task_type: "keyword_extraction".to_string(),
            document_id: document.id,
            estimated_duration_ms: 50,
            priority: "low".to_string(),
        });

        tasks
    }

    /// Get statistics about processed documents
    pub fn get_stats(&self) -> DocumentReaderStats {
        DocumentReaderStats {
            total_files_processed: self.processed_files.len(),
            directories_watched: self.config.watch_directories.len(),
            supported_extensions: self.config.supported_extensions.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingTask {
    pub id: Uuid,
    pub task_type: String,
    pub document_id: Uuid,
    pub estimated_duration_ms: u64,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReaderStats {
    pub total_files_processed: usize,
    pub directories_watched: usize,
    pub supported_extensions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_document_reader_config() {
        let config = DocumentReaderConfig::default();
        assert_eq!(config.batch_size, 10);
        assert_eq!(config.scan_interval_ms, 1000);
        assert_eq!(config.nats_subject, "swarm.documents.incoming");
        assert!(config.supported_extensions.contains(&"txt".to_string()));
        assert!(config.supported_extensions.contains(&"md".to_string()));
    }

    #[tokio::test]
    async fn test_document_reader_creation() {
        let config = DocumentReaderConfig::default();
        let reader = DocumentReader::new(config);
        assert_eq!(reader.processed_files.len(), 0);
    }

    #[tokio::test]
    async fn test_supported_file_detection() {
        let config = DocumentReaderConfig::default();
        let reader = DocumentReader::new(config);
        
        assert!(reader.is_supported_file(Path::new("test.txt")));
        assert!(reader.is_supported_file(Path::new("test.md")));
        assert!(reader.is_supported_file(Path::new("test.pdf")));
        assert!(!reader.is_supported_file(Path::new("test.xyz")));
        assert!(!reader.is_supported_file(Path::new("test")));
    }

    #[tokio::test]
    async fn test_document_reading() {
        // Create a temporary directory with a test file
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "This is a test document").unwrap();

        let mut config = DocumentReaderConfig::default();
        config.watch_directories = vec![temp_dir.path().to_path_buf()];
        
        let mut reader = DocumentReader::new(config);
        
        let document = reader.read_document(&test_file).await.unwrap();
        assert_eq!(document.filename, "test.txt");
        assert_eq!(document.content, "This is a test document");
        assert_eq!(document.document_type, DocumentType::Text);
        assert!(document.metadata.contains_key("file_path"));
        assert!(document.metadata.contains_key("file_size"));
    }

    #[tokio::test]
    async fn test_task_generation() {
        let config = DocumentReaderConfig::default();
        let reader = DocumentReader::new(config);
        
        let document = Document::new("test.pdf".to_string(), "PDF content".to_string());
        let tasks = reader.generate_processing_tasks(&document);
        
        // Should have at least text extraction, language detection, and keyword extraction
        assert!(tasks.len() >= 3);
        
        let task_types: Vec<&String> = tasks.iter().map(|t| &t.task_type).collect();
        assert!(task_types.contains(&&"text_extraction".to_string()));
        assert!(task_types.contains(&&"language_detection".to_string()));
        assert!(task_types.contains(&&"keyword_extraction".to_string()));
        
        // PDF should have page counting
        assert!(task_types.contains(&&"page_counting".to_string()));
    }
}
