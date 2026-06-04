# FluAutoClicker v0.1.0-beta.5

Release date: 2026-06-04

This release focuses on Linux improvements & QoL updates.

## Overview

Linux support is now much closer to release.

## Enhancements

### Main part

- CPS Test now more accurate. ( with high cps )
- Upgraded Macro Mode to record/create independent Mouse/Keyboard Down and Up events, enabling full support for Move+Hold drag-and-drop recording and complex sequences.
- Optimized Macro Recording CPU usage and UI performance:
    - Replaced real-time disk writes and DOM re-rendering on every mouse move with a throttled 1Hz update cycle.
    - Added a "Live Update" toggle in Settings → General to allow completely disabling UI/disk updates during recording (saving to disk only on stop for minimal CPU overhead).
    - Optimized macro action list animations.
- Added two new mouse movement styles for macros:
    - "Linear": Smoothly slides the cursor in a straight line between points over a set duration.
    - "Smooth\*" (Beta/WIP): Records and replays the exact natural path and curves of your hand movement.
- Added the ability to edit existing macro actions directly from the list.
- Updated app icon.

### Linux part (see note)

- Reduced the built-in Linux click interval cap so ultra-low intervals now behave more accurately.
- Linux macro playback now uses `/dev/uinput` virtual mouse and keyboard devices instead of the previous `enigo` playback path.
- Added a Linux startup permission check with a `/dev/uinput` modal.
- Added an in-app `pkexec` setup action that can install a persistent udev rule and `uinput` group setup for FluAutoClicker.
- Added Linux platform capability fields for `/dev/uinput` availability, macro playback backend, recording backend, session type, desktop environment, and compositor/window-manager hints.
- Added Linux bundle targets for AppImage, DEB, and RPM in Tauri config.
- Extended Linux keyboard virtual device support and added focused mapping tests for UI-exposed keys.
- Added `CapsLock` to the supported key set for the Linux virtual keyboard device.

## Removed

- Removed the Multi-Threading feature. It was not needed :/

## Bug Fixes

- Fixed a bug where global hotkeys and macro key/mouse presses were ignored when the main application window had focus.
- Maximum speed at <3 ms delay (when 2ms is set now it's 2ms and not 0ms as it was before) (stupid bug, yeah...)

## Notes

- ⚠️ This version has not been fully tested on Linux, so there may be errors in some of the new changes!
- ⚠️ Config migration: the `record_mouse_moves` option in macro settings changed from a boolean (true/false) to a string mode ("off"|"instant"|"linear"|"smooth"). Existing configs are auto-migrated: true → instant, false → off. Re-saving the config will store the new string form.
- Wayland macro recording and global hotkeys remain intentionally unavailable for now; the app reports those limitations through capabilities/UX instead of failing silently.
- Linux input automation requires writable `/dev/uinput`. Use the in-app permission prompt or configure a persistent udev rule.
