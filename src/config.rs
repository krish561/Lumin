use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Per-monitor configuration stored in TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub name: String,
    /// Last known brightness (0-100)
    pub brightness: u16,
    /// User's preferred backend (can override auto-detection)
    /// None = let app decide, Some = force this backend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_backend: Option<String>,
    /// Last known DDC bus (informational, helps with quick startup)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ddc_bus: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEntry {
    pub monitor: String,
    pub brightness: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub entries: Vec<ProfileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub monitors: Vec<MonitorConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<Profile>,
}

impl Config {
    /// Load config from ~/.config/lumin/lumin.toml, or create empty if not exists
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content).unwrap_or_default())
        } else {
            Ok(Config::default())
        }
    }

    /// Save config to ~/.config/lumin/lumin.toml
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;

        Ok(())
    }

    /// Find or create monitor config by name
    pub fn get_or_create_monitor(&mut self, name: &str) -> &mut MonitorConfig {
        if !self.monitors.iter().any(|m| m.name == name) {
            self.monitors.push(MonitorConfig {
                name: name.to_string(),
                brightness: 50, // Default middle brightness
                preferred_backend: None,
                ddc_bus: None,
            });
        }

        self.monitors.iter_mut().find(|m| m.name == name).unwrap()
    }

    /// Update brightness for a monitor
    pub fn set_brightness(&mut self, monitor_name: &str, brightness: u16) {
        let cfg = self.get_or_create_monitor(monitor_name);
        cfg.brightness = brightness.min(100);
    }

    /// Update DDC bus info (cached for quick reference)
    pub fn set_ddc_bus(&mut self, monitor_name: &str, bus: Option<String>) {
        let cfg = self.get_or_create_monitor(monitor_name);
        cfg.ddc_bus = bus;
    }

    /// Get user's preferred backend override, if set
    pub fn preferred_backend(&self, monitor_name: &str) -> Option<&str> {
        self.monitors
            .iter()
            .find(|m| m.name == monitor_name)
            .and_then(|m| m.preferred_backend.as_deref())
    }

    /// Get last known brightness for a monitor
    pub fn last_brightness(&self, monitor_name: &str) -> Option<u16> {
        self.monitors
            .iter()
            .find(|m| m.name == monitor_name)
            .map(|m| m.brightness)
    }

    fn config_path() -> Result<PathBuf> {
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.config", h)))?;

        Ok(PathBuf::from(config_home).join("lumin/lumin.toml"))
    }

    pub fn upsert_profile(&mut self, name: String, entries: Vec<ProfileEntry>) {
        if let Some(existing) = self.profiles.iter_mut().find(|p| p.name == name) {
            existing.entries = entries;
        } else {
            self.profiles.push(Profile { name, entries });
        }
    }

    /// Delete a profile by index, returns true if removed
    pub fn delete_profile(&mut self, index: usize) -> bool {
        if index < self.profiles.len() {
            self.profiles.remove(index);
            true
        } else {
            false
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            monitors: Vec::new(),
            profiles: Vec::new(),
        }
    }
}
#[test]
fn test_config_get_or_create() {
    let mut config = Config::default();

    // First call - creates the monitor
    {
        let monitor = config.get_or_create_monitor("eDP-1");
        assert_eq!(monitor.name, "eDP-1");
        assert_eq!(monitor.brightness, 50);
    }

    // Second call - returns existing, mutate it
    {
        let monitor2 = config.get_or_create_monitor("eDP-1");
        assert_eq!(config.monitors.len(), 1);
        monitor2.brightness = 75;
    }

    // Third call - verify mutation persisted
    {
        let monitor3 = config.get_or_create_monitor("eDP-1");
        assert_eq!(monitor3.brightness, 75);
    }
}
