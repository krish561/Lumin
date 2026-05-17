use crate::brightness;
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

pub struct Device {
    pub name: String,
    pub description: String,
    pub brightness: u16,
    pub backend: BrightnessBackend,
    pub bus: Option<String>,
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
}

impl App {
    fn detect_backend(monitor: &monitor::HyprMonitor) -> BrightnessBackend {
        if monitor.name.starts_with("eDP") {
            return BrightnessBackend::Laptop;
        }

        if (monitor.name.starts_with("HDMI") || monitor.name.starts_with("DP"))
            && let Some(bus) = &monitor.bus
            && brightness::supports_ddc(bus)
        {
            return BrightnessBackend::Ddc;
        }

        BrightnessBackend::Software
    }

    pub fn new(window_mode: WindowMode) -> Self {
        let monitors = monitor::get_monitors().unwrap_or_default();

        let devices: Vec<Device> = monitors
            .into_iter()
            .map(|monitor| {
                let backend = Self::detect_backend(&monitor);

                let brightness = match backend {
                    BrightnessBackend::Laptop => brightness::get_laptop_brightness().unwrap_or(50),
                    BrightnessBackend::Ddc => 50,
                    BrightnessBackend::Software => 50,
                };

                Device {
                    name: monitor.name,
                    description: monitor.description,
                    brightness,
                    backend: backend.clone(),
                    bus: monitor.bus,
                }
            })
            .collect();

        // Spawn overlays for software-backend displays
        for device in &devices {
            if matches!(device.backend, BrightnessBackend::Software) {
                let _ = software::spawn_overlay(&device.name, device.brightness as u8);
            }
        }

        Self {
            devices,
            selected: 0,
            ui: UiState::new(window_mode),
        }
    }
    pub fn cleanup(&self) {
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

        if let Some(message) = Self::apply_brightness(device) {
            self.notify(message);
        }
    }
    pub fn decrease(&mut self) {
        let Some(device) = self.devices.get_mut(self.selected) else {
            return;
        };

        device.brightness = device.brightness.saturating_sub(5);

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

    fn notify(&mut self, message: String) {
        self.ui.notification = Some(Notification {
            message,
            expires_at: Instant::now() + Duration::from_secs(4),
        });
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
                    let message = format!(
                        "DDC failed for {}, switching to Software backend",
                        device.name
                    );

                    device.backend = BrightnessBackend::Software;
                    Some(message)
                } else {
                    None
                }
            }
            BrightnessBackend::Software => {
                //let _ = software::set_software_brightness(device.brightness);
                let _ = software::update_software_brightness(&device.name, device.brightness as u8);
                None
            }
        }
    }
}
