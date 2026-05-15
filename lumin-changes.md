# Lumin UI Changes

## Summary

The lumin UI has been reshaped to follow wiremix's TUI design language while keeping lumin's brightness-controller behavior and simpler app model.

## Files Changed

- `src/ui.rs`
  - Replaced the boxed title/list/gauge/footer layout with a full-screen wiremix-style device list.
  - Added local `Theme` and `CharSet` primitives modeled after wiremix defaults.
  - Added three-line selected-row markers using `░`, `▒`, `░`.
  - Added wiremix-style brightness bars using `━` for filled segments and `╌` for empty segments.
  - Added `•••` scroll indicators for lists that extend beyond the viewport.
  - Added a wiremix-style bottom-left section strip with display-relevant placeholders: `[Displays]`, `Profiles`, `Gamma`, and `Night`.
  - Added placeholder overlay windows for each bottom-left section. These are visual scaffolds for future display, profile, gamma, and night-light controls.
  - Added hit-testing helpers for mouse interaction with section tabs, display rows, and brightness bars.
  - Kept the UI compatible with the terminal/system theme by inheriting the default foreground and background. Only accent roles use explicit colors, matching wiremix's behavior: light cyan for selection, light blue for filled bars, and dark gray for secondary text/empty bars.

- `src/ui/`
  - Split the former monolithic `src/ui.rs` into focused modules:
    - `mod.rs` owns render orchestration and shared layout helpers.
    - `theme.rs` owns `Theme` and `CharSet`.
    - `devices.rs` owns display rows and row hit testing.
    - `bars.rs` owns brightness bar rendering and value hit testing.
    - `footer.rs` owns the bottom section strip and section hit testing.
    - `overlays.rs` owns placeholder section windows.
    - `notifications.rs` owns top-right transient notification rendering.

- `src/app.rs`
  - Introduced a real UI state model:
    - `ActiveSection` replaces the earlier display-section naming.
    - `UiState` now owns overlay state, notification state, and window mode.
    - `WindowMode` records whether lumin is running inline or in the floating TUI window.
  - Made selection movement safe when no displays are detected.
  - Made brightness adjustment safe when no device is selected.
  - Clamped brightness changes to the `0..=100` range using saturating arithmetic.
  - Replaced the DDC fallback `println!` with transient notification state. The message `DDC failed for <display>, switching to Software backend` is now shown in the UI for four seconds.

- `src/main.rs`
  - Added an Omarchy/Hyprland startup handoff: when launched normally under Hyprland, lumin opens itself in a separate pinned floating terminal window using `hyprctl dispatch exec` window rules. The spawned TUI sets `LUMIN_FLOATING_TUI=1` to avoid recursively spawning more windows.
  - Enabled terminal mouse capture.
  - Added mouse support for selecting displays, clicking/dragging brightness bars, scrolling through displays, and opening/closing the bottom-left placeholder windows.
  - Added keyboard shortcuts `1` through `4` for toggling the placeholder sections and `Esc` for closing a section window.

## Verification

- `cargo fmt`
- `cargo check`
- `cargo run` under Hyprland returned `ok` from the floating-window dispatch.

## Design Notes

This is a visual port, not a direct architecture port. wiremix has a larger configurable UI system for PipeWire objects, tabs, dropdowns, mouse areas, and help screens. lumin now copies the parts that matter for the first brightness-controller UI pass: dense rows, selector glyphs, terminal-theme-friendly colors, compact controls, text-mode bars, and a bottom-left section strip. The placeholder sections are visual scaffolding for future display workflows.

The original `src/ui.rs` has been replaced by the `src/ui/` module tree. The public UI API remains `ui::render`, `ui::section_at`, `ui::device_at`, and `ui::brightness_at`, so input handling in `main.rs` did not need a broad rewrite.
