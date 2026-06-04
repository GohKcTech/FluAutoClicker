# FluAutoClicker v0.1.0-beta.6

Release date: 2026-06-04

## Overview

Just a hotfix for v0.1.0-beta.5 `:O`

## Bug Fixes / Optimizations

- Fixed a bug with double click in macro mode.
- Disabled all macro list animations, transitions, and smooth scrolling when the main window is minimized or hidden.

## Notes

- ⚠️ This version has not been fully tested on Linux, so there may be errors in some of the new changes!
- ⚠️ [v0.1.0-beta.5] Config migration: the `record_mouse_moves` option in macro settings changed from a boolean (true/false) to a string mode ("off"|"instant"|"linear"|"smooth"). Existing configs are auto-migrated: true → instant, false → off. Re-saving the config will store the new string form.
- Wayland macro recording and global hotkeys remain intentionally unavailable for now; the app reports those limitations through capabilities/UX instead of failing silently.
- Linux input automation requires writable `/dev/uinput`. Use the in-app permission prompt or configure a persistent udev rule.
