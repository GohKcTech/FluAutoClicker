# FluAutoClicker v0.1.0-beta.8

Release date: 15.07.2026

## Overview

Multi-language localization support with 9 languages, font customization options, a completely redesigned profile section, acrylic blur visual improvements, a new raw move mode for macro mouse actions, and a redesigned drawer and time input layout.

## Enhancements

- Added localization system with 9 languages (English, Russian, German*, French*, Spanish*, Portuguese*, Japanese*, Korean*, Chinese*).
- Added font customization - two independent selectors for Application Font and Localization Font. Built-in fonts: MuseoModerno, Geologica, Montserrat. Also supports system fonts, Google Fonts search/download, and importing custom font files (.ttf, .otf, .woff, .woff2).
- Redesigned the Profile section with a new layout and improved UX.
- Added acrylic background effect update for better visuals.
- Added a Licenses drawer section for third-party attribution.
- Added an experimental raw move mode for mouse actions.
- Redesigned the drawer (top-on-windows) layout.
- Redesigned the time input layout for better usability.

And other minor enhancements...

## Bug Fixes

- Fixed various UI/UX issues related to the new profile redesign.
- Fixed font rendering and fallback behavior across platforms.
- General stability improvements for the localization system.

## Notes

- ⚠️ Languages marked with `*` are translated using AI and may contain inaccuracies.
- ⚠️ This version has not been tested on Linux, so there may be errors in some of the new changes!
- ⚠️ [v0.1.0-beta.5] Config migration: the `record_mouse_moves` option in macro settings changed from a boolean (true/false) to a string mode ("off"|"instant"|"linear"|"smooth"). Existing configs are auto-migrated: true → instant, false → off. Re-saving the config will store the new string form.
- Wayland macro recording and global hotkeys remain intentionally unavailable for now; the app reports those limitations through capabilities/UX instead of failing silently.
- Linux input automation requires writable `/dev/uinput`. Use the in-app permission prompt or configure a persistent udev rule.
