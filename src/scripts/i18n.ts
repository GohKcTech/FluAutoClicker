export type AppLang = "en";

type Dict = Record<string, string>;

const translations: Record<AppLang, Dict> = {
  en: {
    "tab.mouse": "Mouse",
    "tab.keyboard": "Keyboard",
    "tab.macro": "Macro",

    "footer.settings": "SETTINGS",
    "footer.hotkeys": "HOTKEYS",
    "footer.profiles": "PROFILES",
    "footer.help": "HELP",

    "start.start": "START",
    "start.stop": "STOP",

    "drawer.settings": "Settings",
    "drawer.hotkeys": "Hotkeys",
    "drawer.profiles": "Profiles System",
    "drawer.help": "Help & Resources",

    "settings.group.general": "General",
    "settings.group.theme": "Theme & Appearance",
    "settings.group.utils": "Utils",
    "settings.group.profiles": "Profiles",

    "settings.autostart.title": "Start on System Startup",
    "settings.autostart.desc": "Automatically launch when computer starts",
    "settings.tray.title": "Minimize to Tray",
    "settings.tray.desc": "Close button hides window instead of exiting",
    "settings.language.title": "Language",
    "settings.language.desc": "UI localization language",
    "settings.stop_on_move.title": "Stop on Mouse Move (Custom Position)",
    "settings.stop_on_move.desc": "Stop clicker after manual cursor override",

    "settings.accent.title": "Accent Color",
    "settings.remove_italic.title": "Remove Italic",

    "settings.cps_test.title": "CPS Test",
    "settings.cps_test.desc": "Check your clicks per second speed",

    "profiles.input.placeholder": "new profile name",
    "profiles.create": "CREATE",
    "profiles.empty": "No profiles yet",

    "hotkey.toggle_start_stop": "Toggle Start/Stop",
    "hotkey.pick_position": "Pick Coordinate",
    "hotkey.hypr.apply": "Apply Hyprland Binding",

    "notify.settings_saved": "Settings saved",
    "notify.hotkey_updated": "Hotkey updated",
    "notify.profile_loaded": "Profile loaded",
    "notify.profile_renamed": "Profile renamed",
    "notify.profile_deleted": "Profile deleted",
    "notify.profile_created": "Profile created",
    "notify.profile_name_required": "Enter profile name first",
    "notify.update_check_done": "Pre-release update channel check completed.",
    "notify.position_guard_stop": "Clicker stopped: custom position was overridden by manual mouse movement.",
  },
};

let currentLanguage: AppLang = "en";

function setText(selector: string, value: string) {
  const element = document.querySelector(selector) as HTMLElement | null;
  if (element) {
    element.textContent = value;
  }
}

function applyTranslations() {
  const tr = translations[currentLanguage];

  setText('.mode-tabs .tab[data-tab="mouse"]', tr["tab.mouse"]);
  setText('.mode-tabs .tab[data-tab="keyboard"]', tr["tab.keyboard"]);
  setText('.mode-tabs .tab[data-tab="macro"]', tr["tab.macro"]);

  setText('#settings-btn .footer-label', tr["footer.settings"]);
  setText('#hotkeys-btn .footer-label', tr["footer.hotkeys"]);
  setText('#profiles-btn .footer-label', tr["footer.profiles"]);
  setText('#help-btn .footer-label', tr["footer.help"]);

  setText('#start-btn .start-label', tr["start.start"]);

  const groupTitles = document.querySelectorAll('#section-settings .settings-group-title');
  if (groupTitles[0]) groupTitles[0].textContent = tr["settings.group.general"];
  if (groupTitles[1]) groupTitles[1].textContent = tr["settings.group.theme"];
  if (groupTitles[2]) groupTitles[2].textContent = tr["settings.group.utils"];

  const settingsItems = document.querySelectorAll('#section-settings .settings-list-item .settings-item-content');
  if (settingsItems[0]) {
    const title = settingsItems[0].querySelector('.settings-item-title');
    const desc = settingsItems[0].querySelector('.settings-item-desc');
    if (title) title.textContent = tr["settings.autostart.title"];
    if (desc) desc.textContent = tr["settings.autostart.desc"];
  }
  if (settingsItems[1]) {
    const title = settingsItems[1].querySelector('.settings-item-title');
    const desc = settingsItems[1].querySelector('.settings-item-desc');
    if (title) title.textContent = tr["settings.tray.title"];
    if (desc) desc.textContent = tr["settings.tray.desc"];
  }
  if (settingsItems[2]) {
    const title = settingsItems[2].querySelector('.settings-item-title');
    const desc = settingsItems[2].querySelector('.settings-item-desc');
    if (title) title.textContent = tr["settings.language.title"];
    if (desc) desc.textContent = tr["settings.language.desc"];
  }
  if (settingsItems[3]) {
    const title = settingsItems[3].querySelector('.settings-item-title');
    const desc = settingsItems[3].querySelector('.settings-item-desc');
    if (title) title.textContent = tr["settings.stop_on_move.title"];
    if (desc) desc.textContent = tr["settings.stop_on_move.desc"];
  }

  const accentTitle = document.querySelector('#accent-settings-trigger .settings-item-title');
  if (accentTitle) accentTitle.textContent = tr["settings.accent.title"];
  const italicTitle = document.querySelector('#remove-italic-trigger .settings-item-title');
  if (italicTitle) italicTitle.textContent = tr["settings.remove_italic.title"];
  const cpsTitle = document.querySelector('#cps-test-btn .settings-item-title');
  const cpsDesc = document.querySelector('#cps-test-btn .settings-item-desc');
  if (cpsTitle) cpsTitle.textContent = tr["settings.cps_test.title"];
  if (cpsDesc) cpsDesc.textContent = tr["settings.cps_test.desc"];

  const profileInput = document.getElementById('profile-name-input') as HTMLInputElement | null;
  if (profileInput) profileInput.placeholder = tr["profiles.input.placeholder"];
  const profileCreateBtn = document.getElementById('profile-create-btn');
  if (profileCreateBtn) profileCreateBtn.textContent = tr["profiles.create"];

  const hotkeyRows = document.querySelectorAll('.hotkey-list-item[data-hotkey-action]');
  hotkeyRows.forEach((row) => {
    const action = (row as HTMLElement).dataset.hotkeyAction;
    const title = row.querySelector('.hotkey-title');
    if (!title) return;

    if (action === "toggle_start_stop") title.textContent = tr["hotkey.toggle_start_stop"];
    if (action === "pick_position") title.textContent = tr["hotkey.pick_position"];
  });

  const hyprTitle = document.querySelector('#hyprland-hotkey-apply .hotkey-title');
  if (hyprTitle) hyprTitle.textContent = tr["hotkey.hypr.apply"];

  document.documentElement.lang = currentLanguage;
}

export function initI18n(initialLanguage?: string) {
  void initialLanguage;
  currentLanguage = "en";
  localStorage.setItem("flu-language", currentLanguage);
  applyTranslations();
}

export function setLanguage(language: string) {
  void language;
  currentLanguage = "en";
  localStorage.setItem("flu-language", currentLanguage);
  applyTranslations();
}

export function getLanguage(): AppLang {
  return currentLanguage;
}

export function t(key: string, fallback?: string): string {
  return translations[currentLanguage][key] || fallback || key;
}
