# FluAutoClicker v0.1.0-beta.2

Release date: 2026-04-26

## Enhancements

- Added synchronization and health checks between the frontend and backend. 
- Implemented dynamic scrollbar updates upon window resizing.

## Bug Fixes

- Fixed an issue where the Shift button was not functioning correctly. (e.g., Shift+A => "A", A => "a", Shift+1 => "!", 1 => "1")
- Resolved a bug where the keyboard wouldn't work without the main button active. (e.g., Win => *Windows Menu*)
- Corrected the handling of keyboard modifier sequences (Ctrl -> Alt -> Shift -> Win -> Key).
- Fixed the visual styling of scrollbars.
- Fixed an issue where Mouse Jiggler or Multi-threading menus would incorrectly show in Keyboard or Macro modes.

## Miscellaneous

- Updated the "Sleep (Delay)" icon for better clarity.
- Updated the picking process: replaced "PICKING..." with a "PICK 5" to "PICK 1" countdown timer.
- Refined the styling of the keyboard preview view.
- The frontend version now automatically synchronizes with the Tauri configuration file.
- DevTools is now available when run with `pnpm tauri dev` command.