# Lumin Current Progress

Updated: 2026-05-17

## Current Scope

Lumin has reached the main goal of being a TUI brightness controller for Hyprland.

The next phase is to grow it into a broader TUI display settings controller. The footer sections already point in that direction:

- `Displays`
- `Profiles`
- `Gamma`
- `Night`

Those sections are still placeholders, but they are now the right place to add real display-management workflows.

## What Works Now

- Hyprland monitor discovery through `hyprctl monitors -j`.
- Laptop brightness through `brightnessctl`.
- DDC/CI probing through `ddcutil getvcp 10 --bus <bus>`.
- DDC brightness writes through `ddcutil setvcp 10 <value> --bus <bus>`.
- Software dimming fallback through a per-monitor layer-shell overlay.
- Automatic backend selection:
  - `eDP*` -> laptop backend
  - `HDMI*` / `DP*` with a working DDC bus -> DDC backend
  - everything else -> software overlay backend
- Automatic software overlay spawn for software-backend displays.
- Overlay cleanup on normal quit.
- Top-right notification when DDC fails and a display falls back to software.
- Mouse and keyboard brightness control.
- Floating Hyprland TUI launch through Ghostty or Alacritty.

## App Architecture

Core app state lives in `src/app.rs`.

- `App`
  - owns detected display devices
  - tracks selected display
  - owns `UiState`

- `Device`
  - `name`
  - `description`
  - `brightness`
  - `backend`
  - optional DDC `bus`

- `BrightnessBackend`
  - `Laptop`
  - `Ddc`
  - `Software`

- `UiState`
  - active footer section
  - active notification
  - window mode

- `ActiveSection`
  - `Displays`
  - `Profiles`
  - `Gamma`
  - `Night`

## Backend Notes

### Laptop

Implemented with `brightnessctl`.

Current read path:

- `brightnessctl get`
- `brightnessctl max`

Current write path:

- `brightnessctl set <value>%`

### DDC

Partially implemented.

Current probe path:

- `ddcutil getvcp 10 --bus <bus>`

Current write path:

- `ddcutil setvcp 10 <value> --bus <bus>`

Current limitation:

- DDC depends on monitor, cable, adapter, dock, KVM, GPU driver, and input source.
- The code can support DDC if a valid bus is available, but this still needs more real hardware testing.

### Software Overlay

Implemented as a separate binary:

- `src/bin/lumin-overlay.rs`

The TUI controls it through:

- `src/software.rs`

The overlay receives brightness updates over a Unix socket in `XDG_RUNTIME_DIR`.

Important detail: GTK/layer-shell was inheriting theme backgrounds until the overlay was forced to paint a black surface and clear the opaque region. That fix made software dimming usable.

## UI Structure

The old single `src/ui.rs` file has been split into:

- `src/ui/mod.rs`
  - render orchestration
  - shared layout helpers
  - public UI functions used by `main.rs`

- `src/ui/theme.rs`
  - colors
  - glyphs

- `src/ui/devices.rs`
  - display rows
  - backend labels
  - row hit testing

- `src/ui/bars.rs`
  - brightness bars
  - slider hit testing

- `src/ui/footer.rs`
  - footer section strip
  - section hit testing

- `src/ui/overlays.rs`
  - placeholder section windows

- `src/ui/notifications.rs`
  - top-right notification popup

## Current Controls

Keyboard:

- `q`: cleanup overlays and quit
- `Esc`: close section window
- `Up` / `Down`: select display
- `Left` / `Right`: adjust brightness
- `1`: toggle Displays
- `2`: toggle Profiles
- `3`: toggle Gamma
- `4`: toggle Night

Mouse:

- click display row: select display
- click or drag brightness bar: set brightness
- scroll: move selection
- click footer section: open/close section window
- click elsewhere: close open section window

## Runtime Behavior

- `cargo run` under Hyprland spawns lumin in a pinned floating terminal window.
- Ghostty is preferred.
- Alacritty is the fallback.
- The spawned TUI sets `LUMIN_FLOATING_TUI=1` to avoid recursive launches.
- Event polling runs every 100ms so notifications can expire without user input.

## Completed Phase

The brightness-controller phase is effectively complete enough to move forward:

- the TUI exists
- brightness sliders work
- backends are separated
- software fallback works
- DDC has a path, even if it needs more hardware testing
- software overlays are managed by the app
- the UI has the structure needed for future display settings

## Next Phase

The next phase should treat Lumin as a display settings controller, not only a brightness controller.

Good next targets:

- Make `Displays` a real section:
  - output information
  - enabled/disabled state
  - position/layout
  - scale
  - transform/rotation
  - refresh rate

- Make `Profiles` real:
  - named brightness/display presets
  - per-monitor backend preference
  - startup profile

- Make `Gamma` real:
  - gamma controls
  - color temperature
  - contrast-style adjustments if feasible

- Make `Night` real:
  - warm overlay or gamma-based night mode
  - schedule/manual toggle

- Improve backend reliability:
  - better DDC bus mapping
  - better fallback messages
  - overlay restart/recovery
  - panic-safe terminal and overlay cleanup

## Single Progress File

This file replaces `lumin-changes.md`.

Use this as the single progress/state document going forward.


## **Next Phase**
  Phase 1: Brightness App Stabilization

  - Overlay process management:
      - prevent duplicate overlays per monitor
      - startup cleanup for stale sockets/processes
      - respawn overlay if socket update fails
      - cleanup on quit and panic if possible
  - Backend reliability:
      - record why each backend was selected
      - expose backend status in the row or overlay
      - make DDC failure switch to Software and spawn overlay immediately
      - retry DDC only when explicitly requested later
  - UX polish:
      - notification queue or replace-current behavior
      - smoother slider behavior
      - maybe show backend in footer/detail overlay
  - Safety:
      - terminal cleanup guard
      - overlay cleanup guard
      - no stdout/stderr leaks from child overlay

  Phase 2: Brightness Settings

  - Add real Displays overlay content, but brightness-focused:
      - monitor name
      - description
      - backend
      - DDC bus
      - current brightness
      - overlay status
      - last error/fallback reason
  - Add backend switching:
      - force Software
      - retry DDC
      - use Laptop where available
  - Add basic persistence:
      - store preferred backend per monitor
      - store last brightness per monitor
      - restore on startup

  Phase 3: Display Settings
  Use live hyprctl first:

  - scale
  - resolution
  - refresh rate
  - transform
  - position
  - enable/disable

  Then persistence:

  - generate a lumin-managed config file
  - ask user to source/import it from Hyprland config
  - do not rewrite the user’s main config directly

  That last part matters. A clean pattern would be:

  ~/.config/hypr/lumin-monitors.conf

  User adds once:

  source = ~/.config/hypr/lumin-monitors.conf

  Then lumin owns only that file. This avoids corrupting hand-written config.
