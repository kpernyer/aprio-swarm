//! Document Processor Implementation
//!
//! This module provides concrete implementations of document processing
//! capabilities for the Aprio Swarm system.

use super::*;
use swarm_core::prelude::*;
use swarm_core::{DocumentProcessingOptions, TextAnalysisOptions};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;
use chrono::Utc;

/// Concrete implementation of DocumentProcessor trait
pub struct SwarmDocumentProcessor {
    config: DocumentProcessingConfig,
    stats: DocumentProcessingStats,
    supported_types: Vec<DocumentType>,
}

impl SwarmDocumentProcessor {
    /// Create a new document processor
    pub fn new(config: DocumentProcessingConfig) -> Self {
        let supported_types = config.supported_types.clone();
        Self {
            config,
            stats: DocumentProcessingStats::default(),
            supported_types,
        }
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> &DocumentProcessingStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = DocumentProcessingStats::default();
    }
    
    /// Process text content for analysis
    async fn process_text_content(&self, content: &str, options: &DocumentProcessingOptions) -> DocumentProcessingResult {
        let start_time = Instant::now();
        
        let mut result = DocumentProcessingResult {
            document_id: Uuid::new_v4(),
            extracted_text: if options.extract_text { Some(content.to_string()) } else { None },
            metadata: HashMap::new(),
            language: None,
            keywords: Vec::new(),
            sentiment: None,
            classification: None,
            embeddings: None,
            processing_time_ms: 0,
            processed_at: Utc::now(),
        };
        
        // Language detection
        if options.detect_language {
            result.language = self.detect_language(content);
        }
        
        // Keyword extraction
        if options.extract_keywords {
            result.keywords = self.extract_keywords(content);
        }
        
        // Sentiment analysis (simplified)
        if options.extract_text {
            result.sentiment = self.analyze_sentiment(content);
        }
        
        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        result
    }
    
    /// Detect language from text content
    fn detect_language(&self, text: &str) -> Option<String> {
        // Simplified language detection based on common words
        let text_lower = text.to_lowercase();
        
        // English indicators
        let english_words = ["the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"];
        let english_count = english_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>();
        
        // Swedish indicators
        let swedish_words = ["och", "att", "det", "som", "en", "på", "är", "av", "för", "med", "till", "den"];
        let swedish_count = swedish_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>();
        
        if english_count > swedish_count && english_count > 0 {
            Some("en".to_string())
        } else if swedish_count > 0 {
            Some("sv".to_string())
        } else {
            Some("unknown".to_string())
        }
    }
    
    /// Extract keywords from text
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction based on word frequency
        let words: Vec<&str> = text
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| word.len() > 3)
            .collect();
        
        let mut word_count: HashMap<String, usize> = HashMap::new();
        for word in words {
            if word.len() > 3 {
                *word_count.entry(word.to_lowercase()).or_insert(0) += 1;
            }
        }
        
        // Get top 10 most frequent words
        let mut sorted_words: Vec<(String, usize)> = word_count.into_iter().collect();
        sorted_words.sort_by(|a, b| b.1.cmp(&a.1));
        
        sorted_words
            .into_iter()
            .take(10)
            .map(|(word, _)| word)
            .collect()
    }
    
    /// Analyze sentiment (simplified)
    fn analyze_sentiment(&self, text: &str) -> Option<f32> {
        let text_lower = text.to_lowercase();
        
        // Positive words
        let positive_words = ["good", "great", "excellent", "amazing", "wonderful", "fantastic", "love", "like"];
        let positive_count = positive_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>();
        
        // Negative words
        let negative_words = ["bad", "terrible", "awful", "hate", "dislike", "horrible", "worst", "disappointed"];
        let negative_count = negative_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>();
        
        let total_words = text.split_whitespace().count();
        if total_words == 0 {
            return Some(0.0);
        }
        
        let sentiment = (positive_count as f32 - negative_count as f32) / total_words as f32;
        Some(sentiment.clamp(-1.0, 1.0))
    }
    
    /// Simulate PDF processing
    async fn process_pdf(&self, content: &DocumentContent, options: &DocumentProcessingOptions) -> DocumentProcessingResult {
        let start_time = Instant::now();
        
        // Simulate PDF text extraction
        let extracted_text = match content {
            DocumentContent::Text(text) => {
                if options.extract_text {
                    Some(format!("[PDF EXTRACTED] {}", text))
                } else {
                    None
                }
            }
            DocumentContent::Binary(_) => {
                if options.extract_text {
                    Some("[PDF EXTRACTED] Binary PDF content processed".to_string())
                } else {
                    None
                }
            }
            DocumentContent::Reference { .. } => {
                if options.extract_text {
                    Some("[PDF EXTRACTED] Referenced PDF content processed".to_string())
                } else {
                    None
                }
            }
        };
        
        let mut result = DocumentProcessingResult {
            document_id: Uuid::new_v4(),
            extracted_text,
            metadata: HashMap::new(),
            language: None,
            keywords: Vec::new(),
            sentiment: None,
            classification: Some("PDF Document".to_string()),
            embeddings: None,
            processing_time_ms: 0,
            processed_at: Utc::now(),
        };
        
        // Add PDF-specific metadata
        result.metadata.insert("page_count".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));
        result.metadata.insert("pdf_version".to_string(), serde_json::Value::String("1.4".to_string()));
        
        // Process extracted text if available
        if let Some(ref text) = result.extracted_text {
            if options.detect_language {
                result.language = self.detect_language(text);
            }
            if options.extract_keywords {
                result.keywords = self.extract_keywords(text);
            }
            if options.extract_text {
                result.sentiment = self.analyze_sentiment(text);
            }
        }
        
        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        result
    }
    
    /// Simulate Word document processing
    async fn process_word(&self, content: &DocumentContent, options: &DocumentProcessingOptions) -> DocumentProcessingResult {
        let start_time = Instant::now();
        
        // Simulate Word document processing
        let extracted_text = match content {
            DocumentContent::Text(text) => {
                if options.extract_text {
                    Some(format!("[WORD EXTRACTED] {}", text))
                } else {
                    None
                }
            }
            DocumentContent::Binary(_) => {
                if options.extract_text {
                    Some("[WORD EXTRACTED] Binary Word document processed".to_string())
                } else {
                    None
                }
            }
            DocumentContent::Reference { .. } => {
                if options.extract_text {
                    Some("[WORD EXTRACTED] Referenced Word document processed".to_string())
                } else {
                    None
                }
            }
        };
        
        let mut result = DocumentProcessingResult {
            document_id: Uuid::new_v4(),
            extracted_text,
            metadata: HashMap::new(),
            language: None,
            keywords: Vec::new(),
            sentiment: None,
            classification: Some("Word Document".to_string()),
            embeddings: None,
            processing_time_ms: 0,
            processed_at: Utc::now(),
        };
        
        // Add Word-specific metadata
        result.metadata.insert("word_count".to_string(), serde_json::Value::Number(serde_json::Number::from(100)));
        result.metadata.insert("document_format".to_string(), serde_json::Value::String("docx".to_string()));
        
        // Process extracted text if available
        if let Some(ref text) = result.extracted_text {
            if options.detect_language {
                result.language = self.detect_language(text);
            }
            if options.extract_keywords {
                result.keywords = self.extract_keywords(text);
            }
            if options.extract_text {
                result.sentiment = self.analyze_sentiment(text);
            }
        }
        
        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        result
    }
    
    /// Process HTML content
    async fn process_html(&self, content: &DocumentContent, options: &DocumentProcessingOptions) -> DocumentProcessingResult {
        let start_time = Instant::now();
        
        // Extract text from HTML (simplified)
        let extracted_text = match content {
            DocumentContent::Text(text) => {
                if options.extract_text {
                    // Simple HTML tag removal
                    let clean_text = text
                        .replace("<[^>]*>", "")
                        .replace("&nbsp;", " ")
                        .replace("&amp;", "&")
                        .replace("&lt;", "<")
                        .replace("&gt;", ">")
                        .replace("&quot;", "\"")
                        .replace("&#39;", "'");
                    Some(format!("[HTML EXTRACTED] {}", clean_text.trim()))
                } else {
                    None
                }
            }
            DocumentContent::Binary(_) => {
                if options.extract_text {
                    Some("[HTML EXTRACTED] Binary HTML content processed".to_string())
                } else {
                    None
                }
            }
            DocumentContent::Reference { .. } => {
                if options.extract_text {
                    Some("[HTML EXTRACTED] Referenced HTML content processed".to_string())
                } else {
                    None
                }
            }
        };
        
        let mut result = DocumentProcessingResult {
            document_id: Uuid::new_v4(),
            extracted_text,
            metadata: HashMap::new(),
            language: None,
            keywords: Vec::new(),
            sentiment: None,
            classification: Some("HTML Document".to_string()),
            embeddings: None,
            processing_time_ms: 0,
            processed_at: Utc::now(),
        };
        
        // Add HTML-specific metadata
        result.metadata.insert("content_type".to_string(), serde_json::Value::String("text/html".to_string()));
        
        // Process extracted text if available
        if let Some(ref text) = result.extracted_text {
            if options.detect_language {
                result.language = self.detect_language(text);
            }
            if options.extract_keywords {
                result.keywords = self.extract_keywords(text);
            }
            if options.extract_text {
                result.sentiment = self.analyze_sentiment(text);
            }
        }
        
        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        result
    }
    
    /// Process Markdown content
    async fn process_markdown(&self, content: &DocumentContent, options: &DocumentProcessingOptions) -> DocumentProcessingResult {
        let start_time = Instant::now();
        
        // Extract text from Markdown (simplified)
        let extracted_text = match content {
            DocumentContent::Text(text) => {
                if options.extract_text {
                    // Simple Markdown processing
                    let clean_text = text
                        .replace("#", "")
                        .replace("*", "")
                        .replace("_", "")
                        .replace("`", "")
                        .replace("[", "")
                        .replace("]", "")
                        .replace("(", "")
                        .replace(")", "");
                    Some(format!("[MARKDOWN EXTRACTED] {}", clean_text.trim()))
                } else {
                    None
                }
            }
            DocumentContent::Binary(_) => {
                if options.extract_text {
                    Some("[MARKDOWN EXTRACTED] Binary Markdown content processed".to_string())
                } else {
                    None
                }
            }
            DocumentContent::Reference { .. } => {
                if options.extract_text {
                    Some("[MARKDOWN EXTRACTED] Referenced Markdown content processed".to_string())
                } else {
                    None
                }
            }
        };
        
        let mut result = DocumentProcessingResult {
            document_id: Uuid::new_v4(),
            extracted_text,
            metadata: HashMap::new(),
            language: None,
            keywords: Vec::new(),
            sentiment: None,
            classification: Some("Markdown Document".to_string()),
            embeddings: None,
            processing_time_ms: 0,
            processed_at: Utc::now(),
        };
        
        // Add Markdown-specific metadata
        result.metadata.insert("content_type".to_string(), serde_json::Value::String("text/markdown".to_string()));
        
        // Process extracted text if available
        if let Some(ref text) = result.extracted_text {
            if options.detect_language {
                result.language = self.detect_language(text);
            }
            if options.extract_keywords {
                result.keywords = self.extract_keywords(text);
            }
            if options.extract_text {
                result.sentiment = self.analyze_sentiment(text);
            }
        }
        
        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        result
    }
}

#[async_trait]
impl DocumentProcessor for SwarmDocumentProcessor {
    async fn process_document(&self, document: &Document) -> Result<DocumentProcessingResult> {
        // Validate document
        utils::validate_document(document, &self.config)?;
        
        // Process based on document type
        let result = match document.document_type {
            DocumentType::Pdf => {
                self.process_pdf(&document.content, &self.config.processing_options).await
            }
            DocumentType::Word => {
                self.process_word(&document.content, &self.config.processing_options).await
            }
            DocumentType::Text => {
                match &document.content {
                    DocumentContent::Text(text) => {
                        self.process_text_content(text, &self.config.processing_options).await
                    }
                    _ => {
                        return Err(DocumentError::ProcessingFailed {
                            reason: "Text document must have text content".to_string(),
                        }.into());
                    }
                }
            }
            DocumentType::Html => {
                self.process_html(&document.content, &self.config.processing_options).await
            }
            DocumentType::Markdown => {
                self.process_markdown(&document.content, &self.config.processing_options).await
            }
            _ => {
                return Err(DocumentError::UnsupportedDocumentType {
                    document_type: format!("{:?}", document.document_type),
                }.into());
            }
        };
        
        Ok(result)
    }
    
    fn supported_document_types(&self) -> &[DocumentType] {
        &self.supported_types
    }
    
    fn can_process(&self, document_type: &DocumentType) -> bool {
        self.supported_types.contains(document_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::PathBuf;
    
    fn create_test_config() -> DocumentProcessingConfig {
        DocumentProcessingConfig {
            max_file_size: 1024 * 1024, // 1MB
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
            max_concurrent_documents: 5,
        }
    }
    
    fn create_test_document(document_type: DocumentType, content: &str) -> Document {
        Document {
            id: Uuid::new_v4(),
            filename: "test.txt".to_string(),
            document_type,
            content: DocumentContent::Text(content.to_string()),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            size_bytes: content.len(),
        }
    }
    
    #[tokio::test]
    async fn test_document_processor_creation() {
        let config = create_test_config();
        let processor = SwarmDocumentProcessor::new(config);
        
        assert_eq!(processor.supported_document_types().len(), 5);
        assert!(processor.can_process(&DocumentType::Pdf));
        assert!(processor.can_process(&DocumentType::Text));
        assert!(!processor.can_process(&DocumentType::Image));
    }
    
    #[tokio::test]
    async fn test_text_document_processing() {
        let config = create_test_config();
        let processor = SwarmDocumentProcessor::new(config);
        let document = create_test_document(
            DocumentType::Text,
            "This is a test document with some content for processing. It contains multiple words and sentences."
        );
        
        let result = processor.process_document(&document).await.unwrap();
        
        assert!(result.extracted_text.is_some());
        assert!(result.language.is_some());
        assert!(!result.keywords.is_empty());
        assert!(result.sentiment.is_some());
        assert!(result.processing_time_ms > 0);
    }
    
    #[tokio::test]
    async fn test_pdf_document_processing() {
        let config = create_test_config();
        let processor = SwarmDocumentProcessor::new(config);
        let document = create_test_document(
            DocumentType::Pdf,
            "This is PDF content that should be processed."
        );
        
        let result = processor.process_document(&document).await.unwrap();
        
        assert!(result.extracted_text.is_some());
        assert!(result.extracted_text.unwrap().contains("[PDF EXTRACTED]"));
        assert_eq!(result.classification, Some("PDF Document".to_string()));
        assert!(result.metadata.contains_key("page_count"));
    }
    
    #[tokio::test]
    async fn test_html_document_processing() {
        let config = create_test_config();
        let processor = SwarmDocumentProcessor::new(config);
        let document = create_test_document(
            DocumentType::Html,
            "<h1>Title</h1><p>This is <strong>HTML</strong> content with <em>formatting</em>.</p>"
        );
        
        let result = processor.process_document(&document).await.unwrap();
        
        assert!(result.extracted_text.is_some());
        let extracted = result.extracted_text.unwrap();
        assert!(extracted.contains("[HTML EXTRACTED]"));
        assert!(!extracted.contains("<h1>"));
        assert!(!extracted.contains("<strong>"));
        assert_eq!(result.classification, Some("HTML Document".to_string()));
    }
    
    #[tokio::test]
    async fn test_markdown_document_processing() {
        let config = create_test_config();
        let processor = SwarmDocumentProcessor::new(config);
        let document = create_test_document(
            DocumentType::Markdown,
            "# Title\n\nThis is **markdown** content with *formatting* and `code`."
        );
        
        let result = processor.process_document(&document).await.unwrap();
        
        assert!(result.extracted_text.is_some());
        let extracted = result.extracted_text.unwrap();
        assert!(extracted.contains("[MARKDOWN EXTRACTED]"));
        assert!(!extracted.contains("#"));
        assert!(!extracted.contains("**"));
        assert_eq!(result.classification, Some("Markdown Document".to_string()));
    }
    
    #[tokio::test]
    async fn test_unsupported_document_type() {
        let config = create_test_config();
        let processor = SwarmDocumentProcessor::new(config);
        let document = create_test_document(
            DocumentType::Image,
            "This is image content"
        );
        
        let result = processor.process_document(&document).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_language_detection() {
        let config = create_test_config();
        let processor = SwarmDocumentProcessor::new(config);
        
        // Test English detection
        let english_doc = create_test_document(
            DocumentType::Text,
            "The quick brown fox jumps over the lazy dog. This is a test document."
        );
        let result = processor.process_document(&english_doc).await.unwrap();
        assert_eq!(result.language, Some("en".to_string()));
        
        // Test Swedish detection
        let swedish_doc = create_test_document(
            DocumentType::Text,
            "Detta är ett test dokument på svenska. Och det innehåller flera ord."
        );
        let result = processor.process_document(&swedish_doc).await.unwrap();
        assert_eq!(result.language, Some("sv".to_string()));
    }
    
    #[tokio::test]
    async fn test_keyword_extraction() {
        let config = create_test_config();
        let processor = SwarmDocumentProcessor::new(config);
        let document = create_test_document(
            DocumentType::Text,
            "This is a test document with important keywords. The document contains multiple important words and phrases that should be extracted as keywords."
        );
        
        let result = processor.process_document(&document).await.unwrap();
        
        assert!(!result.keywords.is_empty());
        assert!(result.keywords.len() <= 10); // Should not exceed max keywords
        assert!(result.keywords.iter().all(|kw| kw.len() > 3)); // All keywords should be longer than 3 chars
    }
    
    #[tokio::test]
    async fn test_sentiment_analysis() {
        let config = create_test_config();
        let processor = SwarmDocumentProcessor::new(config);
        
        // Test positive sentiment
        let positive_doc = create_test_document(
            DocumentType::Text,
            "This is a great document! I love it and it's amazing. Wonderful work!"
        );
        let result = processor.process_document(&positive_doc).await.unwrap();
        assert!(result.sentiment.unwrap() > 0.0);
        
        // Test negative sentiment
        let negative_doc = create_test_document(
            DocumentType::Text,
            "This is a terrible document. I hate it and it's awful. Worst work ever!"
        );
        let result = processor.process_document(&negative_doc).await.unwrap();
        assert!(result.sentiment.unwrap() < 0.0);
    }
}
