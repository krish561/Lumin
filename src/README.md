````md id="w7m2qx"
# lumin

A terminal-first brightness and display control utility for Hyprland/Wayland setups.

`lumin` is a Rust-based TUI focused on unified brightness control with:
- laptop backlight support via `brightnessctl`
- experimental DDC/CI integration
- planned software dimming fallback
- floating Hyprland integration
- mouse-driven sliders
- Wiremix-inspired interface design

> ⚠️ work in progress 🙇‍♂️

---

## Preview

![Preview](preview.png)

Current UI features:
- dense Wiremix-style layout
- floating overlay mode
- transient notifications
- keyboard + mouse interaction
- compact text-mode sliders
- terminal-theme-aware rendering

---

## Why?

Brightness handling on Linux is messy.

Some displays work with:
- `brightnessctl`
- `ddcutil`
- physical monitor controls

Others:
- expose no usable brightness interface
- fail DDC communication
- break through adapters/KVMs
- use older DVI/VGA paths

`lumin` aims to provide:
```text
one consistent brightness-control experience
inside a modern terminal UI
````

with graceful fallback behavior where possible.

---

## Current Features

### Brightness

* Laptop brightness control (`brightnessctl`)
* Experimental DDC backend structure
* Automatic DDC → Software fallback behavior
* Mouse-draggable brightness sliders
* Keyboard controls

### UI

* Wiremix-inspired TUI
* Floating Hyprland window mode
* Scrollable display list
* Compact Unicode sliders
* Overlay section system
* Notification popups

### Sections

Current placeholder sections:

* Displays
* Profiles
* Gamma
* Night

These are scaffolding for future display-management workflows.

---

## Current Status

### Working

* Laptop brightness control
* Hyprland floating integration
* Mouse interaction
* Notification system
* Backend abstraction architecture
* Wiremix-style UI

### In Progress

* Real DDC/CI support
* Software dimming backend
* Capability probing
* Overlay rendering

---

## Dependencies

Runtime tools:

* `brightnessctl`
* `ddcutil`
* `hyprctl`

Compositor:

* Hyprland

---

## Build

```bash
cargo run
```

---

## Notes

### DDC/CI

DDC support varies heavily depending on:

* monitor firmware
* GPU drivers
* cables/adapters
* docks/KVMs
* display inputs

Current DDC support is experimental and still under active development.

### Software Backend

The software dimming backend is currently being prototyped.
The planned approach is fullscreen Wayland overlay dimming for displays that lack usable hardware brightness controls.

---

## Planned Features

* Real software dimming overlays
* Proper DDC probing
* Refresh rate controls
* Resolution management
* Gamma controls
* Display information panels
* Multi-monitor workflows
* DPMS integration

---

## Inspiration

* [wiremix](https://github.com/tsowell/wiremix)

---

## License

MIT

```
```
