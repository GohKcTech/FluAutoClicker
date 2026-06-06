# FluAutoClicker v0.1.0-beta.7

Release date: unreleased

## Overview

unreleased

## Enhancements

- Added mouse wheel scrolling support. You can now record mouse scrolls, add them manually, configure scroll amounts, and play them back.
- Redesigned the Macro Action configuration UI for Scroll and Sleep actions.
- Redesigned the Macro tab controls panel.
- Added a Time Multiplier speed selector (1x, 2x, 5x, 10x, Custom) to speed up macro action playbacks (sleeps, holds, moves).
- Added a macro execution time indicator that scales automatically based on the selected playback speed.
- Redesigned the profile Save button to dynamically display AUTO-SAVED (for existing profiles) or CREATE (for new profiles) to clarify the auto-saving behavior.

## Bug Fixes

- Fixed a bug where starting or stopping macro recording via a keybind recorded the keybind itself into the macro.
- Fixed a bug where modifiers like `ctrl`, `shift`, `alt`, `win` typed characters like `c`, `s`, `a`, `w` instead of acting as modifier keys.
- Fixed keyboard shortcut combinations not working during macro playback due to incorrect modifier key mapping.
- Fixed the profile rename feature being broken due to conflicts with draft input name updates.

## Notes

- ⚠️ This version has not been fully tested on Linux, so there may be errors in some of the new changes!
- ⚠️ [v0.1.0-beta.5] Config migration: the `record_mouse_moves` option in macro settings changed from a boolean (true/false) to a string mode ("off"|"instant"|"linear"|"smooth"). Existing configs are auto-migrated: true → instant, false → off. Re-saving the config will store the new string form.
- Wayland macro recording and global hotkeys remain intentionally unavailable for now; the app reports those limitations through capabilities/UX instead of failing silently.
- Linux input automation requires writable `/dev/uinput`. Use the in-app permission prompt or configure a persistent udev rule.
