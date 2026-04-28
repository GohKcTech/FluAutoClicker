import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const indexHtml = readFileSync(resolve(root, "src/index.html"), "utf8");
const packageJson = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8"));
const tauriConfig = JSON.parse(readFileSync(resolve(root, "src-tauri/tauri.conf.json"), "utf8"));
const cargoToml = readFileSync(resolve(root, "src-tauri/Cargo.toml"), "utf8");

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
  "config-export-btn",
  "config-import-btn",
  "profile-export-btn",
  "profile-import-btn",
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

const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1] || "";
const codeVersionChecks = [
  ["package.json", packageJson.version],
  ["src-tauri/tauri.conf.json", tauriConfig.version],
  ["src-tauri/Cargo.toml", cargoVersion],
];
const versionMismatches = codeVersionChecks
  .filter(([, version]) => version !== packageJson.version)
  .map(([source, version]) => `${source}=${version || "<missing>"}`);

if (
  missingIds.length
  || missingHotkeys.length
  || forbiddenHotkeys.length
  || versionMismatches.length
) {
  if (missingIds.length) {
    console.error(`Missing required UI ids: ${missingIds.join(", ")}`);
  }
  if (missingHotkeys.length) {
    console.error(`Missing required hotkey actions: ${missingHotkeys.join(", ")}`);
  }
  if (forbiddenHotkeys.length) {
    console.error(`Forbidden hotkey actions still rendered: ${forbiddenHotkeys.join(", ")}`);
  }
  if (versionMismatches.length) {
    console.error(`Code version mismatch: ${versionMismatches.join(", ")}`);
  }
  process.exit(1);
}

console.log("Static smoke check passed.");
