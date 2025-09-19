//! Document Reader Implementation
//!
//! This module provides concrete implementations of document reading
//! capabilities for the Aprio Swarm system.

use super::*;
use swarm_core::prelude::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;

/// Configuration for document reader
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentReaderConfig {
    /// Directories to watch for documents
    pub watch_directories: Vec<PathBuf>,
    
    /// Supported file extensions
    pub supported_extensions: Vec<String>,
    
    /// Maximum file size to read (in bytes)
    pub max_file_size: usize,
    
    /// Scan interval in milliseconds
    pub scan_interval_ms: u64,
    
    /// Batch size for processing documents
    pub batch_size: usize,
    
    /// Enable recursive directory scanning
    pub recursive_scan: bool,
    
    /// File patterns to include (glob patterns)
    pub include_patterns: Vec<String>,
    
    /// File patterns to exclude (glob patterns)
    pub exclude_patterns: Vec<String>,
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
            max_file_size: 100 * 1024 * 1024, // 100MB
            scan_interval_ms: 1000,
            batch_size: 10,
            recursive_scan: true,
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![".*".to_string()], // Exclude hidden files
        }
    }
}

/// Concrete implementation of DocumentReader trait
pub struct SwarmDocumentReader {
    config: DocumentReaderConfig,
    processed_files: HashMap<PathBuf, chrono::DateTime<chrono::Utc>>,
    is_running: bool,
    stats: DocumentReaderStats,
}

impl SwarmDocumentReader {
    /// Create a new document reader
    pub fn new(config: DocumentReaderConfig) -> Self {
        Self {
            config,
            processed_files: HashMap::new(),
            is_running: false,
            stats: DocumentReaderStats {
                total_documents_read: 0,
                documents_per_second: 0.0,
                error_count: 0,
                last_read_time: None,
            },
        }
    }
    
    /// Get current configuration
    pub fn config(&self) -> &DocumentReaderConfig {
        &self.config
    }
    
    /// Get current statistics
    pub fn stats(&self) -> &DocumentReaderStats {
        &self.stats
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
    
    /// Check if file matches include patterns
    fn matches_include_patterns(&self, path: &Path) -> bool {
        // For now, simple implementation - check if any include pattern matches
        // In a real implementation, you'd use glob matching
        self.config.include_patterns.iter().any(|pattern| {
            if pattern == "*" {
                return true;
            }
            // Simple pattern matching (could be enhanced with proper glob)
            path.to_string_lossy().contains(pattern.trim_start_matches('*').trim_end_matches('*'))
        })
    }
    
    /// Check if file matches exclude patterns
    fn matches_exclude_patterns(&self, path: &Path) -> bool {
        // For now, simple implementation - check if any exclude pattern matches
        self.config.exclude_patterns.iter().any(|pattern| {
            if pattern == ".*" {
                return path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with('.'))
                    .unwrap_or(false);
            }
            // Simple pattern matching (could be enhanced with proper glob)
            path.to_string_lossy().contains(pattern.trim_start_matches('*').trim_end_matches('*'))
        })
    }
    
    /// Check if file should be processed
    fn should_process_file(&self, path: &Path) -> bool {
        self.is_supported_file(path) 
            && self.matches_include_patterns(path) 
            && !self.matches_exclude_patterns(path)
    }
    
    /// Check if file is new or has been modified since last scan
    async fn is_new_or_modified_file(&self, path: &Path) -> Result<bool> {
        let metadata = fs::metadata(path).await?;
        let modified_time = metadata.modified()?;
        let modified_datetime: chrono::DateTime<chrono::Utc> = modified_time.into();
        
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
        
        // Get file metadata
        let metadata = fs::metadata(path).await?;
        let size_bytes = metadata.len() as usize;
        
        // Check file size
        if size_bytes > self.config.max_file_size {
            return Err(DocumentError::ProcessingFailed {
                reason: format!("File too large: {} bytes (max: {} bytes)", 
                               size_bytes, self.config.max_file_size),
            }.into());
        }
        
        let modified_time = metadata.modified()?;
        let modified_datetime: chrono::DateTime<chrono::Utc> = modified_time.into();
        
        // Detect document type
        let document_type = utils::detect_document_type_from_path(path);
        
        // Read file content based on type
        let content = match document_type {
            DocumentType::Text | DocumentType::Html | DocumentType::Markdown => {
                let text = fs::read_to_string(path).await?;
                DocumentContent::Text(text)
            }
            _ => {
                let bytes = fs::read(path).await?;
                DocumentContent::Binary(bytes)
            }
        };
        
        // Create document
        let mut document = Document {
            id: Uuid::new_v4(),
            filename,
            document_type: document_type.clone(),
            content,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            size_bytes,
        };
        
        // Add file system metadata
        document.metadata.insert("file_path".to_string(), serde_json::Value::String(path.to_string_lossy().to_string()));
        document.metadata.insert("file_size".to_string(), serde_json::Value::Number(size_bytes.into()));
        document.metadata.insert("modified_time".to_string(), serde_json::Value::String(modified_datetime.to_rfc3339()));
        document.metadata.insert("file_extension".to_string(), serde_json::Value::String(
            path.extension().and_then(|ext| ext.to_str()).unwrap_or("unknown").to_string()
        ));
        document.metadata.insert("mime_type".to_string(), serde_json::Value::String(
            utils::get_mime_type(&document_type).to_string()
        ));
        
        // Update processed files tracking
        self.processed_files.insert(path.to_path_buf(), modified_datetime);
        
        // Update statistics
        self.stats.total_documents_read += 1;
        self.stats.last_read_time = Some(Utc::now());
        
        Ok(document)
    }
    
    /// Scan a single directory for documents
    async fn scan_directory(&mut self, directory: &Path) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        
        if !directory.exists() {
            tracing::warn!("Directory does not exist: {:?}", directory);
            return Ok(documents);
        }
        
        let mut entries = fs::read_dir(directory).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() {
                if self.should_process_file(&path) {
                    if self.is_new_or_modified_file(&path).await? {
                        match self.read_document(&path).await {
                            Ok(document) => {
                                tracing::info!("Read document: {}", path.display());
                                documents.push(document);
                            }
                            Err(e) => {
                                tracing::error!("Failed to read document {}: {}", path.display(), e);
                                self.stats.error_count += 1;
                            }
                        }
                    }
                }
            }
            // Note: Recursive scanning removed for simplicity
        }
        
        Ok(documents)
    }
    
    /// Scan all configured directories for new documents
    async fn scan_directories(&mut self) -> Result<Vec<Document>> {
        let mut all_documents = Vec::new();
        let directories = self.config.watch_directories.clone();
        
        for directory in directories {
            let documents = self.scan_directory(&directory).await?;
            all_documents.extend(documents);
        }
        
        Ok(all_documents)
    }
}

#[async_trait]
impl DocumentReader for SwarmDocumentReader {
    async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting document reader...");
        tracing::info!("Watching directories: {:?}", self.config.watch_directories);
        tracing::info!("Supported extensions: {:?}", self.config.supported_extensions);
        tracing::info!("Scan interval: {}ms", self.config.scan_interval_ms);
        
        self.is_running = true;
        
        while self.is_running {
            match self.scan_directories().await {
                Ok(documents) => {
                    if !documents.is_empty() {
                        tracing::info!("Found {} new documents", documents.len());
                        // In a real implementation, these documents would be sent to a queue or processor
                        // For now, we just log them
                        for doc in documents {
                            tracing::info!("Document: {} ({:?}, {} bytes)", 
                                         doc.filename, doc.document_type, doc.size_bytes);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error scanning directories: {}", e);
                    self.stats.error_count += 1;
                }
            }
            
            sleep(Duration::from_millis(self.config.scan_interval_ms)).await;
        }
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping document reader...");
        self.is_running = false;
        Ok(())
    }
    
    async fn get_next_document(&mut self) -> Result<Option<Document>> {
        let documents = self.scan_directories().await?;
        Ok(documents.into_iter().next())
    }
    
    async fn get_stats(&self) -> DocumentReaderStats {
        self.stats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    fn create_test_config() -> DocumentReaderConfig {
        DocumentReaderConfig {
            watch_directories: vec![PathBuf::from("./test-data")],
            supported_extensions: vec!["txt".to_string(), "md".to_string()],
            max_file_size: 1024 * 1024, // 1MB
            scan_interval_ms: 100,
            batch_size: 5,
            recursive_scan: true,
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![".*".to_string()],
        }
    }
    
    #[test]
    fn test_document_reader_creation() {
        let config = create_test_config();
        let reader = SwarmDocumentReader::new(config);
        
        assert_eq!(reader.processed_files.len(), 0);
        assert!(!reader.is_running);
        assert_eq!(reader.stats().total_documents_read, 0);
    }
    
    #[test]
    fn test_supported_file_detection() {
        let config = create_test_config();
        let reader = SwarmDocumentReader::new(config);
        
        assert!(reader.is_supported_file(Path::new("test.txt")));
        assert!(reader.is_supported_file(Path::new("test.md")));
        assert!(!reader.is_supported_file(Path::new("test.pdf")));
        assert!(!reader.is_supported_file(Path::new("test.xyz")));
    }
    
    #[test]
    fn test_include_pattern_matching() {
        let config = create_test_config();
        let reader = SwarmDocumentReader::new(config);
        
        assert!(reader.matches_include_patterns(Path::new("test.txt")));
        assert!(reader.matches_include_patterns(Path::new("document.md")));
    }
    
    #[test]
    fn test_exclude_pattern_matching() {
        let config = create_test_config();
        let reader = SwarmDocumentReader::new(config);
        
        assert!(reader.matches_exclude_patterns(Path::new(".hidden.txt")));
        assert!(!reader.matches_exclude_patterns(Path::new("visible.txt")));
    }
    
    #[test]
    fn test_should_process_file() {
        let config = create_test_config();
        let reader = SwarmDocumentReader::new(config);
        
        assert!(reader.should_process_file(Path::new("test.txt")));
        assert!(reader.should_process_file(Path::new("document.md")));
        assert!(!reader.should_process_file(Path::new("test.pdf")));
        assert!(!reader.should_process_file(Path::new(".hidden.txt")));
    }
    
    #[tokio::test]
    async fn test_document_reading() {
        // Create a temporary directory with a test file
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "This is a test document").unwrap();
        
        let mut config = create_test_config();
        config.watch_directories = vec![temp_dir.path().to_path_buf()];
        
        let mut reader = SwarmDocumentReader::new(config);
        
        let document = reader.read_document(&test_file).await.unwrap();
        assert_eq!(document.filename, "test.txt");
        assert_eq!(document.document_type, DocumentType::Text);
        assert_eq!(document.size_bytes, 24);
        assert!(document.metadata.contains_key("file_path"));
        assert!(document.metadata.contains_key("file_size"));
        assert!(document.metadata.contains_key("mime_type"));
    }
    
    #[tokio::test]
    async fn test_directory_scanning() {
        // Create a temporary directory with test files
        let temp_dir = tempdir().unwrap();
        let test_file1 = temp_dir.path().join("test1.txt");
        let test_file2 = temp_dir.path().join("test2.md");
        let hidden_file = temp_dir.path().join(".hidden.txt");
        
        fs::write(&test_file1, "Test content 1").unwrap();
        fs::write(&test_file2, "# Test Markdown").unwrap();
        fs::write(&hidden_file, "Hidden content").unwrap();
        
        let mut config = create_test_config();
        config.watch_directories = vec![temp_dir.path().to_path_buf()];
        
        let mut reader = SwarmDocumentReader::new(config);
        
        let documents = reader.scan_directory(temp_dir.path()).await.unwrap();
        assert_eq!(documents.len(), 2); // Should find 2 files, exclude hidden file
        
        let filenames: Vec<&String> = documents.iter().map(|d| &d.filename).collect();
        assert!(filenames.contains(&&"test1.txt".to_string()));
        assert!(filenames.contains(&&"test2.md".to_string()));
    }
    
    #[tokio::test]
    async fn test_get_next_document() {
        // Create a temporary directory with a test file
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Test content").unwrap();
        
        let mut config = create_test_config();
        config.watch_directories = vec![temp_dir.path().to_path_buf()];
        
        let mut reader = SwarmDocumentReader::new(config);
        
        let document = reader.get_next_document().await.unwrap();
        assert!(document.is_some());
        let doc = document.unwrap();
        assert_eq!(doc.filename, "test.txt");
        
        // Second call should return None (no new documents)
        let document = reader.get_next_document().await.unwrap();
        assert!(document.is_none());
    }
    
    #[tokio::test]
    async fn test_statistics_tracking() {
        // Create a temporary directory with test files
        let temp_dir = tempdir().unwrap();
        let test_file1 = temp_dir.path().join("test1.txt");
        let test_file2 = temp_dir.path().join("test2.txt");
        
        fs::write(&test_file1, "Test content 1").unwrap();
        fs::write(&test_file2, "Test content 2").unwrap();
        
        let mut config = create_test_config();
        config.watch_directories = vec![temp_dir.path().to_path_buf()];
        
        let mut reader = SwarmDocumentReader::new(config);
        
        // Read documents
        let _doc1 = reader.read_document(&test_file1).await.unwrap();
        let _doc2 = reader.read_document(&test_file2).await.unwrap();
        
        let stats = reader.stats();
        assert_eq!(stats.total_documents_read, 2);
        assert!(stats.last_read_time.is_some());
        assert_eq!(stats.error_count, 0);
    }
}
