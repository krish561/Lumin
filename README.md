## lumin

A small brightness controller TUI for Hyprland.

It gives one place to control laptop brightness, try DDC/CI for external displays, and fall back to a software dimming overlay when hardware brightness is not available.

The UI is inspired by [wiremix](https://github.com/tsowell/wiremix): dense rows, text sliders, mouse support, and a small footer menu.

![Preview](preview.png)

## What works

- laptop brightness with `brightnessctl`
- experimental external monitor control with `ddcutil`
- software overlay dimming fallback
- keyboard and mouse brightness controls
- Hyprland floating window launch
- fallback notifications

## Controls

- `q`: quit
- `Esc`: close popup
- `Up` / `Down`: select display
- `Left` / `Right`: adjust brightness
- `1` / `2` / `3` / `4`: open footer sections
- mouse click/drag on a slider: set brightness

## Requirements

- Hyprland
- `hyprctl`
- `brightnessctl`
- `ddcutil`
- Ghostty or Alacritty for the floating TUI window

## Run

```bash
cargo run
```

On Hyprland, this opens lumin in a pinned floating terminal window.

The overlay helper is built as a second binary:

```bash
cargo check --bin lumin-overlay
```

## Notes

DDC support is still rough. Some monitors simply do not respond reliably, especially through adapters, docks, KVMs, or older inputs.

The software backend uses a fullscreen layer-shell overlay. It dims visually; it does not change the monitor panel brightness.

The footer sections are placeholders for now:

- Displays
- Profiles
- Gamma
- Night

## Layout

```text
src/
├── app.rs
├── brightness.rs
├── software.rs
├── monitor.rs
├── main.rs
├── bin/lumin-overlay.rs
└── ui/
```
