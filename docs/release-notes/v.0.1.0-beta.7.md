# FluAutoClicker v0.1.0-beta.7

Release date: unreleased

## Overview

unreleased

## Enhancements

- Added mouse wheel scrolling support. You can now record mouse scrolls, add them manually, configure scroll amounts, and play them back.
- Redesigned the Macro Action configuration UI for Scroll and Sleep actions.

## Bug Fixes

- Fixed a bug where starting or stopping macro recording via a keybind recorded the keybind itself into the macro.
- Fixed a bug where modifiers like `ctrl`, `shift`, `alt`, `win` typed characters like `c`, `s`, `a`, `w` instead of acting as modifier keys.
- Fixed keyboard shortcut combinations not working during macro playback due to incorrect modifier key mapping.

## Notes

- ⚠️ This version has not been fully tested on Linux, so there may be errors in some of the new changes!
- ⚠️ [v0.1.0-beta.5] Config migration: the `record_mouse_moves` option in macro settings changed from a boolean (true/false) to a string mode ("off"|"instant"|"linear"|"smooth"). Existing configs are auto-migrated: true → instant, false → off. Re-saving the config will store the new string form.
- Wayland macro recording and global hotkeys remain intentionally unavailable for now; the app reports those limitations through capabilities/UX instead of failing silently.
- Linux input automation requires writable `/dev/uinput`. Use the in-app permission prompt or configure a persistent udev rule.
