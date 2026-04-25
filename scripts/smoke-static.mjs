import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const indexHtml = readFileSync(resolve(root, "src/index.html"), "utf8");

const requiredIds = [
  "start-btn",
  "pick-btn",
  "keyboard-section",
  "kb-record-btn",
  "macro-section",
  "macro-record-btn",
  "macro-add-btn",
  "macro-list-container",
  "profile-save-btn",
  "profile-reload-btn",
  "profiles-list",
  "hotkeys-btn",
  "update-check-btn",
  "jiggler-toggle",
];

const requiredHotkeyActions = [
  "toggle_start_stop",
  "pick_position",
];

const missingIds = requiredIds.filter((id) => !indexHtml.includes(`id="${id}"`));
const missingHotkeys = requiredHotkeyActions.filter(
  (action) => !indexHtml.includes(`data-hotkey-action="${action}"`),
);

const forbiddenHotkeyActions = [
  "toggle_jiggler",
  "toggle_macro",
  "next_profile",
  "prev_profile",
  "media_stop",
];

const forbiddenHotkeys = forbiddenHotkeyActions.filter((action) =>
  indexHtml.includes(`data-hotkey-action="${action}"`),
);

if (missingIds.length || missingHotkeys.length || forbiddenHotkeys.length) {
  if (missingIds.length) {
    console.error(`Missing required UI ids: ${missingIds.join(", ")}`);
  }
  if (missingHotkeys.length) {
    console.error(`Missing required hotkey actions: ${missingHotkeys.join(", ")}`);
  }
  if (forbiddenHotkeys.length) {
    console.error(`Forbidden hotkey actions still rendered: ${forbiddenHotkeys.join(", ")}`);
  }
  process.exit(1);
}

console.log("Static smoke check passed.");
