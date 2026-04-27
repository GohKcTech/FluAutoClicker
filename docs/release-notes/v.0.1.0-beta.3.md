# FluAutoClicker v0.1.0-beta.3

Release date: 2026-04-27

This release adds initial Linux support, currently tested only on Hyprland.

## Overview

- Added Linux support for mouse and keyboard automation.
- Improved the Linux UI/UX by disabling controls for features that are not supported yet.
- Added CI workflows for checking and building the project.
- Pre-release builds can now be downloaded from GitHub Actions artifacts.

## Enhancements

- Improved the Macro mode menu.
    - Made Macro action rows more compact and visually consistent with the rest of the app.
    - Reworked action labels into single readable commands like `Left click at...`, `Move to...`, and `Wait for...`.
    - Added subtle alternating row backgrounds and darker action cards.
    - Unified inline value badges, key badges, action icons, and delete buttons with the app's button styling.
    - Added smoother append animation for new Macro actions without re-rendering the full list.
- Added a WebView terminal error dialog for runtime errors.
- Error details in the dialog can now be copied for easier debugging.
- Macro actions are now loaded during app startup, making playback ready before global hotkeys are used.

## Bug Fixes

- Fixed a Macro mode layout bug where the whole Macro panel could become scrollable after switching from another mode.
- Fixed an intermittent issue where Macro mode could fail to start in production builds.
- Fixed the Start/Stop hotkey behavior so it now respects the currently selected mode instead of always toggling mouse clicking.
- Fixed an issue where repeatedly pressing `Tab` could shift the main interface upward by focusing hidden drawer controls.
- Fixed occasional `window.start_dragging not allowed` errors by adding the missing Tauri window dragging permission.
- Fixed a Macro UI scrolling issue at the minimum app window size.

## Notes

Linux support is currently limited to mouse and keyboard functionality. Other platform-specific features may still be unavailable or disabled.

Pre-release builds are now available from GitHub Actions artifacts.
