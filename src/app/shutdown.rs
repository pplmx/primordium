//! Graceful shutdown handling for the application.
//!
//! Provides signal handling and cleanup procedures for graceful shutdown.

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Manages graceful shutdown of the application.
pub struct ShutdownManager {
    shutdown_requested: Arc<AtomicBool>,
    save_on_exit: bool,
    exit_code: i32,
}

impl Default for ShutdownManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownManager {
    /// Creates a new shutdown manager.
    pub fn new() -> Self {
        Self {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            save_on_exit: true,
            exit_code: 0,
        }
    }

    /// Sets whether to save state on exit.
    pub fn set_save_on_exit(&mut self, save: bool) {
        self.save_on_exit = save;
    }

    /// Requests shutdown.
    pub fn request_shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
        tracing::info!("Shutdown requested");
    }

    /// Checks if shutdown has been requested.
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    /// Returns whether to save state on exit.
    pub fn should_save_on_exit(&self) -> bool {
        self.save_on_exit
    }

    /// Sets the exit code.
    pub fn set_exit_code(&mut self, code: i32) {
        self.exit_code = code;
    }

    /// Gets the exit code.
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Performs cleanup operations before shutdown.
    pub async fn cleanup(&self, app: &mut crate::app::App) -> Result<()> {
        tracing::info!("Performing shutdown cleanup...");

        if self.save_on_exit {
            tracing::info!("Saving state before exit...");
            app.save_state()?;
        }

        tracing::info!("Cleanup complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_manager_new() {
        let manager = ShutdownManager::new();
        assert!(!manager.is_shutdown_requested());
        assert!(manager.should_save_on_exit());
        assert_eq!(manager.exit_code(), 0);
    }

    #[test]
    fn test_shutdown_request() {
        let manager = ShutdownManager::new();
        manager.request_shutdown();
        assert!(manager.is_shutdown_requested());
    }

    #[test]
    fn test_save_on_exit() {
        let mut manager = ShutdownManager::new();
        manager.set_save_on_exit(false);
        assert!(!manager.should_save_on_exit());
    }

    #[test]
    fn test_exit_code() {
        let mut manager = ShutdownManager::new();
        manager.set_exit_code(1);
        assert_eq!(manager.exit_code(), 1);
    }
}
