//! State persistence and management for release operations.
//!
//! This module provides robust state persistence with file locking,
//! corruption recovery, and atomic operations.

use crate::error::{Result, StateError};
use crate::state::ReleaseState;
use serde_json;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// State manager for persistent release state
#[derive(Debug)]
pub struct StateManager {
    /// Path to state file
    state_file_path: PathBuf,
    /// Path to backup state file
    backup_file_path: PathBuf,
    /// Path to lock file
    lock_file_path: PathBuf,
    /// Current lock handle
    lock_handle: Option<FileLock>,
    /// Configuration for state management
    config: StateConfig,
}

/// Configuration for state management
#[derive(Debug, Clone)]
pub struct StateConfig {
    /// How often to create automatic backups (in operations)
    pub backup_frequency: usize,
    /// Maximum age of state files before cleanup (in seconds)
    pub max_state_age_seconds: u64,
    /// Whether to compress state files
    pub compress_state: bool,
    /// Timeout for acquiring file locks (in milliseconds)
    pub lock_timeout_ms: u64,
    /// Whether to validate state on load
    pub validate_on_load: bool,
    /// Whether to create backup files
    pub create_backups: bool,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            backup_frequency: 5,
            max_state_age_seconds: 86400 * 7, // 7 days
            compress_state: false,
            lock_timeout_ms: 5000, // 5 seconds
            validate_on_load: true,
            create_backups: true,
        }
    }
}

/// File lock implementation
#[derive(Debug)]
struct FileLock {
    /// Path to lock file
    lock_file: PathBuf,
    /// Process ID of the locking process
    pid: u32,
    /// Timestamp when lock was acquired
    acquired_at: SystemTime,
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Clean up lock file when FileLock is dropped
        // Log lock information for debugging
        let duration = self.acquired_at.elapsed().unwrap_or_default();
        log::debug!(
            "Releasing file lock (PID: {}, held for: {:?})",
            self.pid,
            duration
        );
        
        // Use the lock_file field to remove the lock
        if self.lock_file.exists() {
            let _ = std::fs::remove_file(&self.lock_file);
        }
    }
}

/// Result of state loading operation
#[derive(Debug)]
pub struct LoadStateResult {
    /// Loaded release state
    pub state: ReleaseState,
    /// Whether state was recovered from backup
    pub recovered_from_backup: bool,
    /// Any warnings during loading
    pub warnings: Vec<String>,
}

/// Result of state saving operation
#[derive(Debug)]
pub struct SaveStateResult {
    /// Whether save was successful
    pub success: bool,
    /// Size of saved state file in bytes
    pub file_size_bytes: u64,
    /// Duration of save operation
    pub save_duration: Duration,
    /// Whether backup was created
    pub backup_created: bool,
}

impl StateManager {
    /// Create a new state manager
    pub fn new<P: AsRef<Path>>(state_file_path: P) -> Result<Self> {
        let state_file_path = state_file_path.as_ref().to_path_buf();
        let backup_file_path = state_file_path.with_extension("backup.json");
        let lock_file_path = state_file_path.with_extension("lock");

        Ok(Self {
            state_file_path,
            backup_file_path,
            lock_file_path,
            lock_handle: None,
            config: StateConfig::default(),
        })
    }

    /// Create a state manager with custom configuration
    pub fn with_config<P: AsRef<Path>>(state_file_path: P, config: StateConfig) -> Result<Self> {
        let state_file_path = state_file_path.as_ref().to_path_buf();
        let backup_file_path = state_file_path.with_extension("backup.json");
        let lock_file_path = state_file_path.with_extension("lock");

        Ok(Self {
            state_file_path,
            backup_file_path,
            lock_file_path,
            lock_handle: None,
            config,
        })
    }

    /// Save release state to file
    pub fn save_state(&mut self, state: &ReleaseState) -> Result<SaveStateResult> {
        let start_time = SystemTime::now();
        
        // Acquire lock
        self.acquire_lock()?;

        // Validate state before saving
        if self.config.validate_on_load {
            state.validate()?;
        }

        // Serialize state
        let serialized = serde_json::to_string_pretty(state)
            .map_err(|e| StateError::SaveFailed {
                reason: format!("Failed to serialize state: {}", e),
            })?;

        // Create backup if needed
        let backup_created = self.maybe_create_backup(&serialized)?;

        // Write to temporary file first (atomic operation)
        let temp_file_path = self.state_file_path.with_extension("tmp");
        
        {
            let mut file = fs::File::create(&temp_file_path)
                .map_err(|e| StateError::SaveFailed {
                    reason: format!("Failed to create temp file: {}", e),
                })?;

            file.write_all(serialized.as_bytes())
                .map_err(|e| StateError::SaveFailed {
                    reason: format!("Failed to write state: {}", e),
                })?;

            file.sync_all()
                .map_err(|e| StateError::SaveFailed {
                    reason: format!("Failed to sync file: {}", e),
                })?;
        }

        // Atomic rename
        fs::rename(&temp_file_path, &self.state_file_path)
            .map_err(|e| StateError::SaveFailed {
                reason: format!("Failed to rename temp file: {}", e),
            })?;

        // Get file size
        let file_size_bytes = fs::metadata(&self.state_file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let save_duration = start_time.elapsed().unwrap_or_default();

        Ok(SaveStateResult {
            success: true,
            file_size_bytes,
            save_duration,
            backup_created,
        })
    }

    /// Load release state from file
    pub fn load_state(&mut self) -> Result<LoadStateResult> {
        // Acquire lock
        self.acquire_lock()?;

        let mut warnings = Vec::new();
        let mut recovered_from_backup = false;

        // Try to load from main state file first
        let state = match self.load_from_file(&self.state_file_path) {
            Ok(state) => state,
            Err(e) => {
                warnings.push(format!("Failed to load main state file: {}", e));
                
                // Try to load from backup
                match self.load_from_file(&self.backup_file_path) {
                    Ok(state) => {
                        warnings.push("Recovered state from backup file".to_string());
                        recovered_from_backup = true;
                        state
                    }
                    Err(backup_err) => {
                        return Err(StateError::LoadFailed {
                            reason: format!(
                                "Failed to load from both main ({}) and backup ({}) files",
                                e, backup_err
                            ),
                        }.into());
                    }
                }
            }
        };

        // Validate loaded state
        if self.config.validate_on_load {
            state.validate()?;
        }

        Ok(LoadStateResult {
            state,
            recovered_from_backup,
            warnings,
        })
    }

    /// Check if state file exists
    pub fn state_exists(&self) -> bool {
        self.state_file_path.exists()
    }

    /// Check if backup state file exists
    pub fn backup_exists(&self) -> bool {
        self.backup_file_path.exists()
    }

    /// Delete state files
    pub fn cleanup_state(&self) -> Result<()> {
        let mut errors = Vec::new();

        // Remove main state file
        if self.state_file_path.exists() {
            if let Err(e) = fs::remove_file(&self.state_file_path) {
                errors.push(format!("Failed to remove state file: {}", e));
            }
        }

        // Remove backup file
        if self.backup_file_path.exists() {
            if let Err(e) = fs::remove_file(&self.backup_file_path) {
                errors.push(format!("Failed to remove backup file: {}", e));
            }
        }

        // Remove lock file
        if self.lock_file_path.exists() {
            if let Err(e) = fs::remove_file(&self.lock_file_path) {
                errors.push(format!("Failed to remove lock file: {}", e));
            }
        }

        if !errors.is_empty() {
            return Err(StateError::SaveFailed {
                reason: format!("Cleanup errors: {}", errors.join("; ")),
            }.into());
        }

        Ok(())
    }

    /// Create manual backup of current state
    pub fn create_backup(&self) -> Result<()> {
        if !self.state_file_path.exists() {
            return Err(StateError::NotFound.into());
        }

        fs::copy(&self.state_file_path, &self.backup_file_path)
            .map_err(|e| StateError::SaveFailed {
                reason: format!("Failed to create backup: {}", e),
            })?;

        Ok(())
    }

    /// Restore from backup
    pub fn restore_from_backup(&self) -> Result<()> {
        if !self.backup_file_path.exists() {
            return Err(StateError::NotFound.into());
        }

        fs::copy(&self.backup_file_path, &self.state_file_path)
            .map_err(|e| StateError::LoadFailed {
                reason: format!("Failed to restore from backup: {}", e),
            })?;

        Ok(())
    }

    /// Get state file information
    pub fn get_state_info(&self) -> Result<StateFileInfo> {
        let main_info = if self.state_file_path.exists() {
            let metadata = fs::metadata(&self.state_file_path)
                .map_err(|e| StateError::LoadFailed {
                    reason: format!("Failed to get state file metadata: {}", e),
                })?;

            Some(FileInfo {
                size_bytes: metadata.len(),
                modified_at: metadata.modified().ok(),
                created_at: metadata.created().ok(),
            })
        } else {
            None
        };

        let backup_info = if self.backup_file_path.exists() {
            let metadata = fs::metadata(&self.backup_file_path)
                .map_err(|e| StateError::LoadFailed {
                    reason: format!("Failed to get backup file metadata: {}", e),
                })?;

            Some(FileInfo {
                size_bytes: metadata.len(),
                modified_at: metadata.modified().ok(),
                created_at: metadata.created().ok(),
            })
        } else {
            None
        };

        let is_locked = self.lock_file_path.exists();

        Ok(StateFileInfo {
            state_file_path: self.state_file_path.clone(),
            backup_file_path: self.backup_file_path.clone(),
            main_file_info: main_info,
            backup_file_info: backup_info,
            is_locked,
        })
    }

    /// Check if another process has locked the state
    pub fn is_locked_by_other_process(&self) -> bool {
        if !self.lock_file_path.exists() {
            return false;
        }

        // Try to read lock file
        match fs::read_to_string(&self.lock_file_path) {
            Ok(content) => {
                // Parse PID from lock file
                if let Ok(pid) = content.trim().parse::<u32>() {
                    // Check if process is still running (simple check)
                    pid != std::process::id()
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    /// Force remove lock (use with caution)
    pub fn force_unlock(&mut self) -> Result<()> {
        if self.lock_file_path.exists() {
            fs::remove_file(&self.lock_file_path)
                .map_err(|e| StateError::SaveFailed {
                    reason: format!("Failed to remove lock file: {}", e),
                })?;
        }

        self.lock_handle = None;
        Ok(())
    }

    /// Load state from specific file
    fn load_from_file(&self, file_path: &Path) -> Result<ReleaseState> {
        let mut file = fs::File::open(file_path)
            .map_err(|e| StateError::LoadFailed {
                reason: format!("Failed to open file {}: {}", file_path.display(), e),
            })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| StateError::LoadFailed {
                reason: format!("Failed to read file {}: {}", file_path.display(), e),
            })?;

        let state: ReleaseState = serde_json::from_str(&contents)
            .map_err(|e| StateError::Corrupted {
                reason: format!("Failed to deserialize state: {}", e),
            })?;

        Ok(state)
    }

    /// Create backup if configured
    fn maybe_create_backup(&self, serialized_state: &str) -> Result<bool> {
        if !self.config.create_backups {
            return Ok(false);
        }

        fs::write(&self.backup_file_path, serialized_state)
            .map_err(|e| StateError::SaveFailed {
                reason: format!("Failed to create backup: {}", e),
            })?;

        Ok(true)
    }

    /// Acquire file lock
    fn acquire_lock(&mut self) -> Result<()> {
        if self.lock_handle.is_some() {
            return Ok(()); // Already locked
        }

        let start_time = SystemTime::now();
        let timeout = Duration::from_millis(self.config.lock_timeout_ms);

        while start_time.elapsed().unwrap_or_default() < timeout {
            // Try to create lock file
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&self.lock_file_path)
            {
                Ok(mut file) => {
                    let pid = std::process::id();
                    file.write_all(pid.to_string().as_bytes())
                        .map_err(|e| StateError::SaveFailed {
                            reason: format!("Failed to write lock file: {}", e),
                        })?;

                    self.lock_handle = Some(FileLock {
                        lock_file: self.lock_file_path.clone(),
                        pid,
                        acquired_at: SystemTime::now(),
                    });

                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // Lock file exists, wait a bit and try again
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(StateError::SaveFailed {
                        reason: format!("Failed to create lock file: {}", e),
                    }.into());
                }
            }
        }

        Err(StateError::SaveFailed {
            reason: "Timeout waiting for file lock".to_string(),
        }.into())
    }

    /// Update configuration
    pub fn set_config(&mut self, config: StateConfig) {
        self.config = config;
    }

    /// Get configuration
    pub fn config(&self) -> &StateConfig {
        &self.config
    }
}

impl Drop for StateManager {
    fn drop(&mut self) {
        // Release lock when manager is dropped
        if self.lock_handle.is_some() {
            let _ = fs::remove_file(&self.lock_file_path);
        }
    }
}

/// Information about state files
#[derive(Debug, Clone)]
pub struct StateFileInfo {
    /// Path to main state file
    pub state_file_path: PathBuf,
    /// Path to backup state file
    pub backup_file_path: PathBuf,
    /// Information about main state file
    pub main_file_info: Option<FileInfo>,
    /// Information about backup file
    pub backup_file_info: Option<FileInfo>,
    /// Whether state is currently locked
    pub is_locked: bool,
}

/// Information about a single file
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// File size in bytes
    pub size_bytes: u64,
    /// Last modified time
    pub modified_at: Option<SystemTime>,
    /// Created time
    pub created_at: Option<SystemTime>,
}

impl StateFileInfo {
    /// Check if state files exist
    pub fn has_state(&self) -> bool {
        self.main_file_info.is_some()
    }

    /// Check if backup exists
    pub fn has_backup(&self) -> bool {
        self.backup_file_info.is_some()
    }

    /// Get total size of all state files
    pub fn total_size_bytes(&self) -> u64 {
        let main_size = self.main_file_info.as_ref().map(|f| f.size_bytes).unwrap_or(0);
        let backup_size = self.backup_file_info.as_ref().map(|f| f.size_bytes).unwrap_or(0);
        main_size + backup_size
    }

    /// Format state info for display
    pub fn format_info(&self) -> String {
        let mut info = String::new();
        
        if let Some(main_info) = &self.main_file_info {
            info.push_str(&format!("Main state: {} bytes", main_info.size_bytes));
            if let Some(modified) = main_info.modified_at {
                if let Ok(elapsed) = modified.elapsed() {
                    info.push_str(&format!(" (modified {}s ago)", elapsed.as_secs()));
                }
            }
        } else {
            info.push_str("No main state file");
        }

        if let Some(backup_info) = &self.backup_file_info {
            info.push_str(&format!(", Backup: {} bytes", backup_info.size_bytes));
        }

        if self.is_locked {
            info.push_str(" [LOCKED]");
        }

        info
    }
}

impl SaveStateResult {
    /// Format save result for display
    pub fn format_result(&self) -> String {
        if self.success {
            let backup_info = if self.backup_created { " (backup created)" } else { "" };
            format!(
                "✅ State saved: {} bytes in {:.2}s{}",
                self.file_size_bytes,
                self.save_duration.as_secs_f64(),
                backup_info
            )
        } else {
            "❌ Failed to save state".to_string()
        }
    }
}

impl LoadStateResult {
    /// Format load result for display
    pub fn format_result(&self) -> String {
        let mut result = if self.recovered_from_backup {
            "⚠️ State loaded from backup".to_string()
        } else {
            "✅ State loaded successfully".to_string()
        };

        if !self.warnings.is_empty() {
            result.push_str(&format!(" ({} warnings)", self.warnings.len()));
        }

        result
    }
}