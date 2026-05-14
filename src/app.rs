pub struct Device {
    pub name: String,
    pub brightness: u16,
}

pub struct App {
    pub devices: Vec<Device>,
    pub selected: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            devices: vec![
                Device {
                    name: String::from("Laptop Display"),
                    brightness: 50,
                },
                Device {
                    name: String::from("Dell P2412H"),
                    brightness: 70,
                },
            ],
            selected: 0,
        }
    }

    pub fn next(&mut self) {
        if self.selected < self.devices.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn increase(&mut self) {
        let device = &mut self.devices[self.selected];

        if device.brightness < 100 {
            device.brightness += 5;
        }
    }

    pub fn decrease(&mut self) {
        let device = &mut self.devices[self.selected];

        if device.brightness > 0 {
            device.brightness -= 5;
        }
    }
}
