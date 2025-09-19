//! File Discovery Module
//!
//! This module provides utilities for discovering and monitoring files
//! in the file system for document processing.

use super::*;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;

/// File discovery configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileDiscoveryConfig {
    /// Directories to monitor
    pub watch_directories: Vec<PathBuf>,
    
    /// File patterns to include (glob patterns)
    pub include_patterns: Vec<String>,
    
    /// File patterns to exclude (glob patterns)
    pub exclude_patterns: Vec<String>,
    
    /// Maximum file size to process (in bytes)
    pub max_file_size: usize,
    
    /// Minimum file size to process (in bytes)
    pub min_file_size: usize,
    
    /// Scan interval in milliseconds
    pub scan_interval_ms: u64,
    
    /// Enable recursive directory scanning
    pub recursive_scan: bool,
    
    /// Enable file system watching (inotify/fsevents)
    pub enable_fs_watching: bool,
    
    /// File age threshold (ignore files older than this)
    pub max_file_age_hours: Option<u64>,
}

impl Default for FileDiscoveryConfig {
    fn default() -> Self {
        Self {
            watch_directories: vec![PathBuf::from("./test-data")],
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![".*".to_string(), "*.tmp".to_string(), "*.log".to_string()],
            max_file_size: 100 * 1024 * 1024, // 100MB
            min_file_size: 1, // 1 byte
            scan_interval_ms: 1000,
            recursive_scan: true,
            enable_fs_watching: false,
            max_file_age_hours: Some(24 * 7), // 1 week
        }
    }
}

/// File discovery statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileDiscoveryStats {
    /// Total files discovered
    pub total_files_discovered: u64,
    
    /// Files discovered per second
    pub discovery_rate_per_second: f32,
    
    /// Total directories scanned
    pub total_directories_scanned: u64,
    
    /// Last scan time
    pub last_scan_time: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Error count
    pub error_count: u64,
    
    /// Files by extension
    pub files_by_extension: HashMap<String, u64>,
    
    /// Files by size range
    pub files_by_size_range: HashMap<String, u64>,
}

impl Default for FileDiscoveryStats {
    fn default() -> Self {
        Self {
            total_files_discovered: 0,
            discovery_rate_per_second: 0.0,
            total_directories_scanned: 0,
            last_scan_time: None,
            error_count: 0,
            files_by_extension: HashMap::new(),
            files_by_size_range: HashMap::new(),
        }
    }
}

/// File information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub filename: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub modified_time: chrono::DateTime<chrono::Utc>,
    pub created_time: chrono::DateTime<chrono::Utc>,
    pub is_hidden: bool,
    pub is_symlink: bool,
    pub mime_type: Option<String>,
}

/// File discovery service
pub struct FileDiscoveryService {
    config: FileDiscoveryConfig,
    stats: FileDiscoveryStats,
    discovered_files: HashMap<PathBuf, FileInfo>,
    is_running: bool,
}

impl FileDiscoveryService {
    /// Create a new file discovery service
    pub fn new(config: FileDiscoveryConfig) -> Self {
        Self {
            config,
            stats: FileDiscoveryStats::default(),
            discovered_files: HashMap::new(),
            is_running: false,
        }
    }
    
    /// Get current configuration
    pub fn config(&self) -> &FileDiscoveryConfig {
        &self.config
    }
    
    /// Get current statistics
    pub fn stats(&self) -> &FileDiscoveryStats {
        &self.stats
    }
    
    /// Get discovered files
    pub fn discovered_files(&self) -> &HashMap<PathBuf, FileInfo> {
        &self.discovered_files
    }
    
    /// Check if file matches include patterns
    fn matches_include_patterns(&self, path: &Path) -> bool {
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
    
    /// Check if file should be discovered
    fn should_discover_file(&self, path: &Path, size_bytes: u64) -> bool {
        // Check patterns
        if !self.matches_include_patterns(path) || self.matches_exclude_patterns(path) {
            return false;
        }
        
        // Check size constraints
        if size_bytes < self.config.min_file_size as u64 || size_bytes > self.config.max_file_size as u64 {
            return false;
        }
        
        // Check file age if configured
        if let Some(max_age_hours) = self.config.max_file_age_hours {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified_time) = metadata.modified() {
                    let modified_datetime: chrono::DateTime<chrono::Utc> = modified_time.into();
                    let age_hours = (Utc::now() - modified_datetime).num_hours();
                    if age_hours > max_age_hours as i64 {
                        return false;
                    }
                }
            }
        }
        
        true
    }
    
    /// Get file information
    async fn get_file_info(&self, path: &Path) -> Result<FileInfo> {
        let metadata = fs::metadata(path).await?;
        let size_bytes = metadata.len();
        let modified_time = metadata.modified()?;
        let created_time = metadata.created().unwrap_or(modified_time);
        
        let modified_datetime: chrono::DateTime<chrono::Utc> = modified_time.into();
        let created_datetime: chrono::DateTime<chrono::Utc> = created_time.into();
        
        let filename = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());
        
        let is_hidden = filename.starts_with('.');
        let is_symlink = metadata.file_type().is_symlink();
        
        // Determine MIME type based on extension
        let mime_type = extension.as_ref().map(|ext| {
            match ext.as_str() {
                "txt" => "text/plain",
                "md" => "text/markdown",
                "html" | "htm" => "text/html",
                "pdf" => "application/pdf",
                "doc" | "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "xls" | "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "ppt" | "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "mp3" => "audio/mpeg",
                "wav" => "audio/wav",
                "mp4" => "video/mp4",
                "avi" => "video/x-msvideo",
                _ => "application/octet-stream",
            }.to_string()
        });
        
        Ok(FileInfo {
            path: path.to_path_buf(),
            filename,
            extension,
            size_bytes,
            modified_time: modified_datetime,
            created_time: created_datetime,
            is_hidden,
            is_symlink,
            mime_type,
        })
    }
    
    /// Scan a single directory for files
    async fn scan_directory(&mut self, directory: &Path) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        
        if !directory.exists() {
            tracing::warn!("Directory does not exist: {:?}", directory);
            return Ok(files);
        }
        
        let mut entries = fs::read_dir(directory).await?;
        self.stats.total_directories_scanned += 1;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() {
                match self.get_file_info(&path).await {
                    Ok(file_info) => {
                        if self.should_discover_file(&path, file_info.size_bytes) {
                            files.push(file_info);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to get file info for {}: {}", path.display(), e);
                        self.stats.error_count += 1;
                    }
                }
            }
            // Note: Recursive scanning removed for simplicity
        }
        
        Ok(files)
    }
    
    /// Scan all configured directories
    async fn scan_all_directories(&mut self) -> Result<Vec<FileInfo>> {
        let mut all_files = Vec::new();
        let directories = self.config.watch_directories.clone();
        
        for directory in directories {
            match self.scan_directory(&directory).await {
                Ok(files) => {
                    all_files.extend(files);
                }
                Err(e) => {
                    tracing::error!("Failed to scan directory {:?}: {}", directory, e);
                    self.stats.error_count += 1;
                }
            }
        }
        
        Ok(all_files)
    }
    
    /// Update statistics
    fn update_stats(&mut self, new_files: &[FileInfo]) {
        self.stats.total_files_discovered += new_files.len() as u64;
        self.stats.last_scan_time = Some(Utc::now());
        
        // Update files by extension
        for file in new_files {
            if let Some(ref ext) = file.extension {
                *self.stats.files_by_extension.entry(ext.clone()).or_insert(0) += 1;
            }
            
            // Update files by size range
            let size_range = match file.size_bytes {
                0..=1024 => "0-1KB",
                1025..=10240 => "1-10KB",
                10241..=102400 => "10-100KB",
                102401..=1048576 => "100KB-1MB",
                1048577..=10485760 => "1-10MB",
                10485761..=104857600 => "10-100MB",
                _ => "100MB+",
            };
            *self.stats.files_by_size_range.entry(size_range.to_string()).or_insert(0) += 1;
        }
    }
    
    /// Start the file discovery service
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting file discovery service...");
        tracing::info!("Watching directories: {:?}", self.config.watch_directories);
        tracing::info!("Scan interval: {}ms", self.config.scan_interval_ms);
        
        self.is_running = true;
        
        while self.is_running {
            match self.scan_all_directories().await {
                Ok(new_files) => {
                    if !new_files.is_empty() {
                        tracing::info!("Discovered {} new files", new_files.len());
                        
                        // Update discovered files
                        for file in &new_files {
                            self.discovered_files.insert(file.path.clone(), file.clone());
                        }
                        
                        // Update statistics
                        self.update_stats(&new_files);
                        
                        // Log some file details
                        for file in new_files.iter().take(5) {
                            tracing::info!("File: {} ({}, {} bytes)", 
                                         file.filename, 
                                         file.extension.as_deref().unwrap_or("no ext"),
                                         file.size_bytes);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error during file discovery: {}", e);
                    self.stats.error_count += 1;
                }
            }
            
            sleep(Duration::from_millis(self.config.scan_interval_ms)).await;
        }
        
        Ok(())
    }
    
    /// Stop the file discovery service
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping file discovery service...");
        self.is_running = false;
        Ok(())
    }
    
    /// Get files by extension
    pub fn get_files_by_extension(&self, extension: &str) -> Vec<&FileInfo> {
        self.discovered_files.values()
            .filter(|file| file.extension.as_deref() == Some(extension))
            .collect()
    }
    
    /// Get files by size range
    pub fn get_files_by_size_range(&self, min_size: u64, max_size: u64) -> Vec<&FileInfo> {
        self.discovered_files.values()
            .filter(|file| file.size_bytes >= min_size && file.size_bytes <= max_size)
            .collect()
    }
    
    /// Get recently modified files
    pub fn get_recently_modified_files(&self, hours: u64) -> Vec<&FileInfo> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours as i64);
        self.discovered_files.values()
            .filter(|file| file.modified_time > cutoff_time)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    fn create_test_config() -> FileDiscoveryConfig {
        FileDiscoveryConfig {
            watch_directories: vec![PathBuf::from("./test-data")],
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![".*".to_string(), "*.tmp".to_string()],
            max_file_size: 1024 * 1024, // 1MB
            min_file_size: 1,
            scan_interval_ms: 100,
            recursive_scan: true,
            enable_fs_watching: false,
            max_file_age_hours: Some(24),
        }
    }
    
    #[test]
    fn test_file_discovery_creation() {
        let config = create_test_config();
        let discovery = FileDiscoveryService::new(config);
        
        assert_eq!(discovery.discovered_files().len(), 0);
        assert_eq!(discovery.stats().total_files_discovered, 0);
    }
    
    #[test]
    fn test_include_pattern_matching() {
        let config = create_test_config();
        let discovery = FileDiscoveryService::new(config);
        
        assert!(discovery.matches_include_patterns(Path::new("test.txt")));
        assert!(discovery.matches_include_patterns(Path::new("document.md")));
    }
    
    #[test]
    fn test_exclude_pattern_matching() {
        let config = create_test_config();
        let discovery = FileDiscoveryService::new(config);
        
        assert!(discovery.matches_exclude_patterns(Path::new(".hidden.txt")));
        assert!(discovery.matches_exclude_patterns(Path::new("temp.tmp")));
        assert!(!discovery.matches_exclude_patterns(Path::new("visible.txt")));
    }
    
    #[test]
    fn test_should_discover_file() {
        let config = create_test_config();
        let discovery = FileDiscoveryService::new(config);
        
        assert!(discovery.should_discover_file(Path::new("test.txt"), 1000));
        assert!(!discovery.should_discover_file(Path::new(".hidden.txt"), 1000));
        assert!(!discovery.should_discover_file(Path::new("temp.tmp"), 1000));
        assert!(!discovery.should_discover_file(Path::new("large.txt"), 2 * 1024 * 1024));
    }
    
    #[tokio::test]
    async fn test_file_info_creation() {
        // Create a temporary file
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Test content").unwrap();
        
        let config = create_test_config();
        let discovery = FileDiscoveryService::new(config);
        
        let file_info = discovery.get_file_info(&test_file).await.unwrap();
        
        assert_eq!(file_info.filename, "test.txt");
        assert_eq!(file_info.extension, Some("txt".to_string()));
        assert_eq!(file_info.size_bytes, 12);
        assert_eq!(file_info.mime_type, Some("text/plain".to_string()));
        assert!(!file_info.is_hidden);
        assert!(!file_info.is_symlink);
    }
    
    #[tokio::test]
    async fn test_directory_scanning() {
        // Create a temporary directory with test files
        let temp_dir = tempdir().unwrap();
        let test_file1 = temp_dir.path().join("test1.txt");
        let test_file2 = temp_dir.path().join("test2.md");
        let hidden_file = temp_dir.path().join(".hidden.txt");
        let temp_file = temp_dir.path().join("temp.tmp");
        
        fs::write(&test_file1, "Test content 1").unwrap();
        fs::write(&test_file2, "# Test Markdown").unwrap();
        fs::write(&hidden_file, "Hidden content").unwrap();
        fs::write(&temp_file, "Temporary content").unwrap();
        
        let mut config = create_test_config();
        config.watch_directories = vec![temp_dir.path().to_path_buf()];
        
        let mut discovery = FileDiscoveryService::new(config);
        
        let files = discovery.scan_directory(temp_dir.path()).await.unwrap();
        assert_eq!(files.len(), 2); // Should find 2 files, exclude hidden and temp files
        
        let filenames: Vec<&String> = files.iter().map(|f| &f.filename).collect();
        assert!(filenames.contains(&&"test1.txt".to_string()));
        assert!(filenames.contains(&&"test2.md".to_string()));
    }
    
    #[tokio::test]
    async fn test_statistics_update() {
        // Create a temporary directory with test files
        let temp_dir = tempdir().unwrap();
        let test_file1 = temp_dir.path().join("test1.txt");
        let test_file2 = temp_dir.path().join("test2.md");
        
        fs::write(&test_file1, "Test content 1").unwrap();
        fs::write(&test_file2, "# Test Markdown").unwrap();
        
        let mut config = create_test_config();
        config.watch_directories = vec![temp_dir.path().to_path_buf()];
        
        let mut discovery = FileDiscoveryService::new(config);
        
        let files = discovery.scan_directory(temp_dir.path()).await.unwrap();
        discovery.update_stats(&files);
        
        let stats = discovery.stats();
        assert_eq!(stats.total_files_discovered, 2);
        assert_eq!(stats.total_directories_scanned, 1);
        assert!(stats.last_scan_time.is_some());
        assert_eq!(stats.files_by_extension.get("txt"), Some(&1));
        assert_eq!(stats.files_by_extension.get("md"), Some(&1));
    }
}
