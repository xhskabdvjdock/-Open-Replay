use anyhow::Result;
use tracing::{info, warn};

pub struct ShortcutManager;

impl ShortcutManager {
    /// Validate that a shortcut string doesn't conflict with common OS shortcuts
    pub fn validate_shortcut(shortcut: &str) -> Result<()> {
        let forbidden = ["Ctrl+Z", "Ctrl+C", "Ctrl+V", "Ctrl+X", "Ctrl+A", "Alt+F4"];
        for f in &forbidden {
            if shortcut.eq_ignore_ascii_case(f) {
                anyhow::bail!("Shortcut '{}' conflicts with system shortcut", shortcut);
            }
        }
        Ok(())
    }
}
