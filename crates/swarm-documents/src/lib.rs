//! Swarm Documents - Document Processing Implementation
//!
//! This crate provides concrete implementations of document processing
//! components for the Aprio Swarm system. It implements the core traits
//! defined in swarm-core with real document processing capabilities.

use swarm_core::prelude::*;
use swarm_core::{DocumentProcessingOptions, TextAnalysisOptions};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;
use chrono::Utc;

// Core modules
pub mod document_processor;
pub mod document_reader;
pub mod file_discovery;

// Re-export main components
pub use document_processor::*;
pub use document_reader::*;
pub use file_discovery::*;

/// Document processing error types
#[derive(thiserror::Error, Debug)]
pub enum DocumentError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Unsupported document type: {document_type}")]
    UnsupportedDocumentType { document_type: String },
    
    #[error("Processing failed: {reason}")]
    ProcessingFailed { reason: String },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type for document operations
pub type DocumentResult<T> = Result<T, DocumentError>;

/// Document processing configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentProcessingConfig {
    /// Maximum file size to process (in bytes)
    pub max_file_size: usize,
    
    /// Supported document types
    pub supported_types: Vec<DocumentType>,
    
    /// Processing options
    pub processing_options: DocumentProcessingOptions,
    
    /// Text analysis options
    pub text_analysis_options: TextAnalysisOptions,
    
    /// Enable parallel processing
    pub enable_parallel_processing: bool,
    
    /// Maximum number of concurrent documents
    pub max_concurrent_documents: usize,
}

impl Default for DocumentProcessingConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            supported_types: vec![
                DocumentType::Pdf,
                DocumentType::Word,
                DocumentType::Text,
                DocumentType::Html,
                DocumentType::Markdown,
            ],
            processing_options: DocumentProcessingOptions::default(),
            text_analysis_options: TextAnalysisOptions::default(),
            enable_parallel_processing: true,
            max_concurrent_documents: 10,
        }
    }
}

/// Document processing statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentProcessingStats {
    /// Total documents processed
    pub total_processed: u64,
    
    /// Documents processed per second
    pub throughput_per_second: f32,
    
    /// Average processing time in milliseconds
    pub average_processing_time_ms: f32,
    
    /// Error count
    pub error_count: u64,
    
    /// Success rate (0.0 to 1.0)
    pub success_rate: f32,
    
    /// Documents by type
    pub documents_by_type: HashMap<DocumentType, u64>,
    
    /// Last processing time
    pub last_processed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for DocumentProcessingStats {
    fn default() -> Self {
        Self {
            total_processed: 0,
            throughput_per_second: 0.0,
            average_processing_time_ms: 0.0,
            error_count: 0,
            success_rate: 1.0,
            documents_by_type: HashMap::new(),
            last_processed_at: None,
        }
    }
}

/// Utility functions for document processing
pub mod utils {
    use super::*;
    
    /// Detect document type from file extension
    pub fn detect_document_type_from_path(path: &Path) -> DocumentType {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "pdf" => DocumentType::Pdf,
                    "doc" | "docx" => DocumentType::Word,
                    "txt" => DocumentType::Text,
                    "html" | "htm" => DocumentType::Html,
                    "md" | "markdown" => DocumentType::Markdown,
                    "xls" | "xlsx" => DocumentType::Excel,
                    "ppt" | "pptx" => DocumentType::PowerPoint,
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" => DocumentType::Image,
                    "mp3" | "wav" | "flac" => DocumentType::Audio,
                    "mp4" | "avi" | "mov" => DocumentType::Video,
                    _ => DocumentType::Unknown,
                }
            } else {
                DocumentType::Unknown
            }
        } else {
            DocumentType::Unknown
        }
    }
    
    /// Get MIME type for document type
    pub fn get_mime_type(document_type: &DocumentType) -> &'static str {
        match document_type {
            DocumentType::Pdf => "application/pdf",
            DocumentType::Word => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            DocumentType::Text => "text/plain",
            DocumentType::Html => "text/html",
            DocumentType::Markdown => "text/markdown",
            DocumentType::Excel => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            DocumentType::PowerPoint => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            DocumentType::Image => "image/*",
            DocumentType::Audio => "audio/*",
            DocumentType::Video => "video/*",
            DocumentType::Unknown => "application/octet-stream",
        }
    }
    
    /// Estimate processing time for document type
    pub fn estimate_processing_time(document_type: &DocumentType, size_bytes: usize) -> std::time::Duration {
        let base_time_ms = match document_type {
            DocumentType::Pdf => 200,
            DocumentType::Word => 150,
            DocumentType::Text => 10,
            DocumentType::Html => 50,
            DocumentType::Markdown => 25,
            DocumentType::Excel => 300,
            DocumentType::PowerPoint => 250,
            DocumentType::Image => 100,
            DocumentType::Audio => 500,
            DocumentType::Video => 1000,
            DocumentType::Unknown => 100,
        };
        
        // Add time based on file size (roughly 1ms per 10KB)
        let size_factor = (size_bytes / 10240) as u64;
        let total_time_ms = base_time_ms + size_factor;
        
        std::time::Duration::from_millis(total_time_ms.min(10000)) // Cap at 10 seconds
    }
    
    /// Validate document for processing
    pub fn validate_document(document: &Document, config: &DocumentProcessingConfig) -> DocumentResult<()> {
        // Check file size
        if document.size_bytes > config.max_file_size {
            return Err(DocumentError::ProcessingFailed {
                reason: format!("Document too large: {} bytes (max: {} bytes)", 
                               document.size_bytes, config.max_file_size),
            });
        }
        
        // Check supported types
        if !config.supported_types.contains(&document.document_type) {
            return Err(DocumentError::UnsupportedDocumentType {
                document_type: format!("{:?}", document.document_type),
            });
        }
        
        Ok(())
    }
    
    /// Create a document from file path
    pub async fn create_document_from_path(path: &Path) -> DocumentResult<Document> {
        if !path.exists() {
            return Err(DocumentError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }
        
        let metadata = fs::metadata(path).await?;
        let size_bytes = metadata.len() as usize;
        let document_type = detect_document_type_from_path(path);
        
        // Read content based on document type
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
        
        let mut doc_metadata = HashMap::new();
        doc_metadata.insert("file_path".to_string(), serde_json::Value::String(path.to_string_lossy().to_string()));
        doc_metadata.insert("file_size".to_string(), serde_json::Value::Number(size_bytes.into()));
        doc_metadata.insert("mime_type".to_string(), serde_json::Value::String(get_mime_type(&document_type).to_string()));
        
        Ok(Document {
            id: Uuid::new_v4(),
            filename: path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string(),
            document_type,
            content,
            metadata: doc_metadata,
            created_at: Utc::now(),
            size_bytes,
        })
    }
}
