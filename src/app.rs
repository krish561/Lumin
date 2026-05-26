use crate::brightness;
use crate::config::Config;
use crate::monitor;
use crate::software;

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum BrightnessBackend {
    Laptop,
    Ddc,
    Software,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveSection {
    Displays,
    Profiles,
    Gamma,
    Night,
}

impl ActiveSection {
    pub const ALL: [Self; 4] = [Self::Displays, Self::Profiles, Self::Gamma, Self::Night];

    pub fn title(self) -> &'static str {
        match self {
            Self::Displays => "Displays",
            Self::Profiles => "Profiles",
            Self::Gamma => "Gamma",
            Self::Night => "Night",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    Inline,
    Floating,
}

#[derive(Debug, Clone)]
pub enum BackendReason {
    /// Laptop backend: matches eDP* pattern
    LaptopDetected,
    /// DDC backend: HDMI/DP with working bus
    DdcDetected { bus: String },
    /// Software backend: default fallback
    SoftwareFallback,
    /// Software backend: DDC failed during brightness change
    DdcFailed { original_bus: String },
    /// User override: preferred backend from config
    UserPreferred { reason: String },
}

impl BackendReason {
    pub fn describe(&self) -> String {
        match self {
            Self::LaptopDetected => "Laptop (eDP)".to_string(),
            Self::DdcDetected { bus } => format!("DDC (bus {})", bus),
            Self::SoftwareFallback => "Software overlay (fallback)".to_string(),
            Self::DdcFailed { original_bus } => {
                format!("Software overlay (DDC failed on bus {})", original_bus)
            }
            Self::UserPreferred { reason } => format!("User preferred ({})", reason),
        }
    }
}

pub struct Device {
    pub name: String,
    pub description: String,
    pub brightness: u16,
    pub backend: BrightnessBackend,
    pub backend_reason: BackendReason,
    pub bus: Option<String>,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: f32,
    pub available_modes: Vec<String>,
    pub scale: f32,
}

pub struct Notification {
    pub message: String,
    expires_at: Instant,
}

pub struct UiState {
    pub active_section: Option<ActiveSection>,
    pub notification: Option<Notification>,
    pub window_mode: WindowMode,
}

impl UiState {
    fn new(window_mode: WindowMode) -> Self {
        Self {
            active_section: None,
            notification: None,
            window_mode,
        }
    }
}

pub struct App {
    pub devices: Vec<Device>,
    pub selected: usize,
    pub ui: UiState,
    pub config: Config,
    pub temperature: u32, // current color temperature in K (default 6000)
    pub gamma: u32,       // current gamma percent (default 100)
    pub pending_mode_revert: Option<(String, Instant)>, // (previous_mode, revert_at)
}

impl App {
    fn detect_backend(
        monitor: &monitor::HyprMonitor,
        config: &Config,
    ) -> (BrightnessBackend, BackendReason) {
        // Check for user preference override first
        if let Some(pref) = config.preferred_backend(&monitor.name) {
            let backend = match pref {
                "Laptop" => BrightnessBackend::Laptop,
                "Ddc" => BrightnessBackend::Ddc,
                "Software" => BrightnessBackend::Software,
                _ => BrightnessBackend::Software,
            };
            return (
                backend,
                BackendReason::UserPreferred {
                    reason: pref.to_string(),
                },
            );
        }

        // Auto-detect
        if monitor.name.starts_with("eDP") {
            return (BrightnessBackend::Laptop, BackendReason::LaptopDetected);
        }

        if (monitor.name.starts_with("HDMI") || monitor.name.starts_with("DP"))
            && let Some(bus) = &monitor.bus
            && brightness::supports_ddc(bus)
        {
            return (
                BrightnessBackend::Ddc,
                BackendReason::DdcDetected { bus: bus.clone() },
            );
        }

        (BrightnessBackend::Software, BackendReason::SoftwareFallback)
    }

    pub fn new(window_mode: WindowMode) -> Self {
        // Load config from disk
        let mut config = Config::load().unwrap_or_default();

        let monitors = monitor::get_monitors().unwrap_or_default();

        let devices: Vec<Device> = monitors
            .into_iter()
            .map(|monitor| {
                let (backend, reason) = Self::detect_backend(&monitor, &config);

                // Update config with DDC bus info if available
                if let Some(ref bus) = monitor.bus {
                    config.set_ddc_bus(&monitor.name, Some(bus.clone()));
                }

                // Restore brightness from config, or read from system
                let brightness = match config.last_brightness(&monitor.name) {
                    Some(saved) => saved,
                    None => match backend {
                        BrightnessBackend::Laptop => {
                            brightness::get_laptop_brightness().unwrap_or(50)
                        }
                        BrightnessBackend::Ddc => 50,
                        BrightnessBackend::Software => 50,
                    },
                };

                Device {
                    name: monitor.name,
                    description: monitor.description,
                    brightness,
                    backend: backend.clone(),
                    backend_reason: reason,
                    bus: monitor.bus,
                    width: monitor.width,
                    height: monitor.height,
                    refresh_rate: monitor.refresh_rate,
                    available_modes: monitor.available_modes,
                    scale: monitor.scale,
                }
            })
            .collect();

        // Spawn overlays for software-backend displays
        for device in &devices {
            if matches!(device.backend, BrightnessBackend::Software) {
                let _ = software::spawn_overlay(&device.name, device.brightness as u8);
            }
        }

        // Ensure all detected devices are in config
        for device in &devices {
            config.get_or_create_monitor(&device.name);
            config.set_brightness(&device.name, device.brightness);
            if let Some(ref bus) = device.bus {
                config.set_ddc_bus(&device.name, Some(bus.clone()));
            }
        }

        let app = Self {
            devices,
            selected: 0,
            ui: UiState::new(window_mode),
            config,
            temperature: 6000,
            gamma: 100,
            pending_mode_revert: None,
        };

        // Apply restored brightness to hardware
        for device in &app.devices {
            let _ = Self::apply_brightness_silent(device);
        }

        app
    }

    pub fn cleanup(&mut self) {
        // Save config before cleanup
        let _ = self.config.save();

        for device in &self.devices {
            if matches!(device.backend, BrightnessBackend::Software) {
                let _ = software::kill_overlay(&device.name);
            }
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < self.devices.len() {
            self.selected = index;
        }
    }

    pub fn set_brightness(&mut self, value: u16) {
        let Some(device) = self.devices.get_mut(self.selected) else {
            return;
        };

        device.brightness = value.min(100);

        // Update config
        self.config.set_brightness(&device.name, device.brightness);

        if let Some(message) = Self::apply_brightness(device) {
            self.notify(message);
        }
    }

    pub fn toggle_section(&mut self, section: ActiveSection) {
        self.ui.active_section = (self.ui.active_section != Some(section)).then_some(section);
    }

    pub fn close_section(&mut self) {
        self.ui.active_section = None;
    }

    pub fn next(&mut self) {
        if self.selected + 1 < self.devices.len() {
            self.selected += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn increase(&mut self) {
        let Some(device) = self.devices.get_mut(self.selected) else {
            return;
        };

        device.brightness = device.brightness.saturating_add(5).min(100);

        // Update config
        self.config.set_brightness(&device.name, device.brightness);

        if let Some(message) = Self::apply_brightness(device) {
            self.notify(message);
        }
    }

    pub fn decrease(&mut self) {
        let Some(device) = self.devices.get_mut(self.selected) else {
            return;
        };

        device.brightness = device.brightness.saturating_sub(5);

        // Update config
        self.config.set_brightness(&device.name, device.brightness);

        if let Some(message) = Self::apply_brightness(device) {
            self.notify(message);
        }
    }

    pub fn dismiss_expired_notifications(&mut self) {
        if self
            .ui
            .notification
            .as_ref()
            .is_some_and(|notification| Instant::now() >= notification.expires_at)
        {
            self.ui.notification = None;
        }
    }

    pub fn set_refresh_rate(&mut self, mode_str: String) {
        let Some(device) = self.devices.get(self.selected) else {
            return;
        };

        // Save current mode before changing
        let previous_mode = format!(
            "{}x{}@{:.2}Hz",
            device.width, device.height, device.refresh_rate
        );
        let name = device.name.clone();
        let scale = device.scale;

        match monitor::set_monitor_mode(&name, &mode_str, scale) {
            Ok(_) => {
                // Update device state
                if let Some(rate) = parse_refresh_rate(&mode_str) {
                    self.devices[self.selected].refresh_rate = rate;
                }
                if let Some((w, h)) = parse_resolution(&mode_str) {
                    self.devices[self.selected].width = w;
                    self.devices[self.selected].height = h;
                }
                // Set revert timer — 10 seconds to confirm
                self.pending_mode_revert =
                    Some((previous_mode, Instant::now() + Duration::from_secs(10)));
                self.notify(
                    "Mode changed. Press Enter to confirm or wait 10s to revert.".to_string(),
                );
            }
            Err(_) => self.notify(format!("Failed to set mode {mode_str}")),
        }
    }

    pub fn confirm_mode(&mut self) {
        self.pending_mode_revert = None;
        self.notify("Mode confirmed.".to_string());
    }

    pub fn check_mode_revert(&mut self) {
        let Some((ref mode, revert_at)) = self.pending_mode_revert.clone() else {
            return;
        };
        if Instant::now() >= revert_at {
            let name = self
                .devices
                .get(self.selected)
                .map(|d| d.name.clone())
                .unwrap_or_default();
            let scale = self
                .devices
                .get(self.selected)
                .map(|d| d.scale)
                .unwrap_or(1.0);
            let _ = monitor::set_monitor_mode(&name, &mode, scale);
            if let Some(rate) = parse_refresh_rate(&mode) {
                self.devices[self.selected].refresh_rate = rate;
            }
            if let Some((w, h)) = parse_resolution(&mode) {
                self.devices[self.selected].width = w;
                self.devices[self.selected].height = h;
            }
            self.pending_mode_revert = None;
            self.notify("Mode reverted.".to_string());
        }
    }

    pub fn increase_temperature(&mut self) {
        self.temperature = (self.temperature + 100).min(6500);
        let t = self.temperature;
        if let Err(_) = brightness::set_temperature(t) {
            self.notify("Failed to set temperature".to_string());
        }
    }

    pub fn decrease_temperature(&mut self) {
        self.temperature = self.temperature.saturating_sub(100).max(2500);
        let t = self.temperature;
        if let Err(_) = brightness::set_temperature(t) {
            self.notify("Failed to set temperature".to_string());
        }
    }

    pub fn increase_gamma(&mut self) {
        self.gamma = (self.gamma + 5).min(200);
        let g = self.gamma;
        if let Err(_) = brightness::set_gamma(g) {
            self.notify("Failed to set gamma".to_string());
        }
    }

    pub fn decrease_gamma(&mut self) {
        self.gamma = self.gamma.saturating_sub(5).max(10);
        let g = self.gamma;
        if let Err(_) = brightness::set_gamma(g) {
            self.notify("Failed to set gamma".to_string());
        }
    }

    pub fn reset_gamma(&mut self) {
        self.temperature = 6000;
        self.gamma = 100;
        let _ = brightness::reset_gamma();
        self.notify("Gamma reset to identity".to_string());
    }

    pub fn toggle_night_light(&mut self) {
        // Night light = warm temperature (4000K)
        // Toggle between night (4000K) and neutral (6000K)
        if self.temperature <= 4500 {
            self.temperature = 6000;
            let _ = brightness::reset_gamma();
            self.notify("Night light off".to_string());
        } else {
            self.temperature = 4000;
            let _ = brightness::set_temperature(4000);
            self.notify("Night light on (4000K)".to_string());
        }
    }

    fn notify(&mut self, message: String) {
        self.ui.notification = Some(Notification {
            message,
            expires_at: Instant::now() + Duration::from_secs(4),
        });
    }

    /// Apply brightness without returning a notification (for silent init)
    fn apply_brightness_silent(device: &Device) {
        match device.backend {
            BrightnessBackend::Laptop => {
                let _ = brightness::set_laptop_brightness(device.brightness);
            }
            BrightnessBackend::Ddc => {
                if let Some(bus) = device.bus.as_deref() {
                    let _ = brightness::set_ddc_brightness(device.brightness, bus);
                }
            }
            BrightnessBackend::Software => {
                let _ = software::update_software_brightness(&device.name, device.brightness as u8);
            }
        }
    }

    fn apply_brightness(device: &mut Device) -> Option<String> {
        match device.backend {
            BrightnessBackend::Laptop => {
                let _ = brightness::set_laptop_brightness(device.brightness);
                None
            }

            BrightnessBackend::Ddc => {
                let success = device
                    .bus
                    .as_deref()
                    .and_then(|bus| brightness::set_ddc_brightness(device.brightness, bus).ok())
                    .unwrap_or(false);

                if !success {
                    let original_bus = device.bus.clone().unwrap_or_default();
                    let message = format!(
                        "DDC failed for {}, switching to Software backend",
                        device.name
                    );

                    device.backend = BrightnessBackend::Software;
                    device.backend_reason = BackendReason::DdcFailed { original_bus };

                    // Spawn overlay on fallback
                    let _ = software::spawn_overlay(&device.name, device.brightness as u8);

                    Some(message)
                } else {
                    None
                }
            }
            BrightnessBackend::Software => {
                let _ = software::update_software_brightness(&device.name, device.brightness as u8);
                None
            }
        }
    }

    pub fn retry_ddc(&mut self) {
        let Some(device) = self.devices.get_mut(self.selected) else {
            return;
        };

        if matches!(device.backend, BrightnessBackend::Laptop) {
            return;
        }

        // Collect what we need before dropping the borrow
        let bus = device.bus.clone();
        let name = device.name.clone();

        if let Some(ref bus) = bus {
            if brightness::supports_ddc(bus) {
                // Re-borrow to mutate
                let device = self.devices.get_mut(self.selected).unwrap();
                device.backend = BrightnessBackend::Ddc;
                device.backend_reason = BackendReason::DdcDetected { bus: bus.clone() };
                let _ = device; // explicit drop before notify

                let _ = software::kill_overlay(&name);
                let cfg = self.config.get_or_create_monitor(&name);
                cfg.preferred_backend = None;
                self.notify(format!("DDC working on {name}! Switched to DDC backend."));
            } else {
                let _ = device; // drop before notify
                self.notify(format!("DDC still not responding on {name}."));
            }
        } else {
            let _ = device;
            self.notify(format!("No DDC bus known for {name}."));
        }
    }
    /// Force the selected device to Software backend and save preference.
    pub fn force_software(&mut self) {
        let Some(device) = self.devices.get_mut(self.selected) else {
            return;
        };

        // Already software
        if matches!(device.backend, BrightnessBackend::Software) {
            return;
        }

        let name = device.name.clone();
        let brightness = device.brightness;

        device.backend = BrightnessBackend::Software;
        device.backend_reason = BackendReason::UserPreferred {
            reason: "Software".to_string(),
        };

        // Spawn overlay
        let _ = software::spawn_overlay(&name, brightness as u8);

        // Save preference to config
        let cfg = self.config.get_or_create_monitor(&name);
        cfg.preferred_backend = Some("Software".to_string());

        self.notify(format!("Forced Software backend for {}.", name));
    }
    pub fn cycle_refresh_rate(&mut self, reverse: bool) {
        let Some(device) = self.devices.get(self.selected) else {
            return;
        };
        let modes = device.available_modes.clone();
        let current = format!(
            "{}x{}@{:.2}Hz",
            device.width, device.height, device.refresh_rate
        );
        let pos = modes.iter().position(|m| m == &current).unwrap_or(0);
        let next = if reverse {
            if pos == 0 { modes.len() - 1 } else { pos - 1 }
        } else {
            (pos + 1) % modes.len()
        };
        if let Some(mode) = modes.get(next).cloned() {
            self.set_refresh_rate(mode);
        }
    }
}
fn parse_refresh_rate(mode_str: &str) -> Option<f32> {
    // "1920x1080@60.00Hz" -> 60.0
    let hz = mode_str.split('@').nth(1)?.trim_end_matches("Hz");
    hz.parse().ok()
}

fn parse_resolution(mode_str: &str) -> Option<(u32, u32)> {
    // "1920x1080@60.00Hz" -> (1920, 1080)
    let res = mode_str.split('@').next()?;
    let mut parts = res.split('x');
    let w = parts.next()?.parse().ok()?;
    let h = parts.next()?.parse().ok()?;
    Some((w, h))
}
