# FluAutoClicker v0.1.0-beta.4

Release date: 2026-04-28

This release focuses on quality-of-life improvements, smoother UI details, and better Windows stability.

## Overview

- Improved Windows stability and window behavior.
- Added more control over the custom titlebar appearance.
- Made Macro actions easier to organize with drag-and-drop reordering.
- Expanded Mouse Jiggler timing and mode behavior.
- Added new beta/dev tooling for easier diagnostics.
- Updated CI build and release workflow configuration.

## Enhancements

- Added a DevTools opener in `Settings -> Utils` for beta and development builds.
- Added an Always on Top option for keeping the app visible above other windows.
- Added a hotkey binding for starting and stopping Macro recording.
- Added drag-and-drop reordering for Macro actions in the action list.
- Updated the check/build and release workflows for the current builder setup.
- Added a "Classic Window Icons" appearance setting to restore the previous titlebar icon style.
- Updated the custom titlebar controls with new minimize and maximize icons while keeping the legacy close icon for visual consistency.
- Polished the Performance Warning slider with better handle/icon alignment and smoother drag color transitions.
- Updated the Lucide icon set. (to v.1.11.0)
- Updated the Mouse Jiggler O-Zone mode.
- Lowered the Mouse Jiggler delay minimum from 5 seconds to 1 second.
- Added JSON export/import for app configuration.
- Added JSON export/import for Profiles, with save-location selection for exported files when supported.
- Added a duplicate button for Macro actions, making repeated automation steps easier to build.
- Added release manifest smoke checks to catch version mismatches before publishing.
- Reduced the built-in Windows click interval cap so ultra-low intervals now behave more accurately. `0ms`, `1ms`, and `2ms` no longer collapse into the same effective delay.

## Notes

Linux support is currently limited to mouse and keyboard functionality. Other platform-specific features may still be unavailable or disabled.

We are actively working on full Linux support. Stay tuned!
