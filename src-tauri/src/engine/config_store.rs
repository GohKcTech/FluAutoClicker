use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs;

use super::macro_engine::types::{MacroAction, MacroRecordingOptions, MacroRepeatMode};

pub const CURRENT_CONFIG_VERSION: u32 = 2;
pub const CURRENT_PROFILE_VERSION: u32 = 1;
pub const MIN_CPS: u32 = 1;

#[cfg(target_os = "linux")]
pub const MAX_CPS: u32 = u32::MAX;

#[cfg(not(target_os = "linux"))]
pub const MAX_CPS: u32 = 10_000;
pub const MAX_VARIATION_MS: u32 = 3_600_000;
pub const MIN_HOLD_DURATION: u32 = 1;
pub const MAX_HOLD_DURATION: u32 = 3_600_000;
pub const MIN_REPEAT_COUNT: u32 = 1;
pub const MAX_REPEAT_COUNT: u32 = 1_000_000;
pub const MIN_JIGGLER_DISTANCE: u32 = 1;
pub const MAX_JIGGLER_DISTANCE: u32 = 500;
pub const MIN_JIGGLER_INTERVAL_MS: u32 = 100;
pub const MAX_JIGGLER_INTERVAL_MS: u32 = 300_000;
pub const MIN_MACRO_REPEAT_DURATION_MS: u64 = 1_000;
pub const MAX_MACRO_REPEAT_DURATION_MS: u64 = 86_400_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    pub theme_mode: String,
    pub theme_color: String,
    pub theme_name: String,
    pub remove_italic: bool,
    pub language: String,
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub notifications_enabled: bool,
    pub stop_on_custom_position_move: bool,
    pub welcome_ack: bool,
    pub hyprland_warn_ack: bool,
    pub pre_alpha_ack: bool,
    pub reduce_motion: String,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            theme_mode: "solid".to_string(),
            theme_color: "#77B6DD".to_string(),
            theme_name: "Flu".to_string(),
            remove_italic: false,
            language: "en".to_string(),
            autostart: false,
            minimize_to_tray: false,
            notifications_enabled: true,
            stop_on_custom_position_move: true,
            welcome_ack: false,
            hyprland_warn_ack: false,
            pre_alpha_ack: false,
            reduce_motion: "none".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MouseSettings {
    pub cps: u32,
    pub variation_ms: u32,
    pub button: String,
    pub click_mode: String,
    pub hold_duration: u32,
    pub hold_unit: String,
    pub repeat_mode: String,
    pub repeat_count: u32,
    pub repeat_unit: String,
    pub position_mode: String,
    pub coord_x: i32,
    pub coord_y: i32,
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self {
            cps: 10,
            variation_ms: 0,
            button: "left".to_string(),
            click_mode: "press".to_string(),
            hold_duration: 100,
            hold_unit: "ms".to_string(),
            repeat_mode: "infinite".to_string(),
            repeat_count: 10,
            repeat_unit: "times".to_string(),
            position_mode: "current".to_string(),
            coord_x: 841,
            coord_y: 425,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyboardSettings {
    pub cps: u32,
    pub variation_ms: u32,
    pub key: String,
    pub modifiers: String,
    pub click_mode: String,
    pub hold_duration: u32,
    pub hold_unit: String,
    pub repeat_mode: String,
    pub repeat_count: u32,
    pub repeat_unit: String,
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        Self {
            cps: 10,
            variation_ms: 0,
            key: "a".to_string(),
            modifiers: "none".to_string(),
            click_mode: "press".to_string(),
            hold_duration: 100,
            hold_unit: "ms".to_string(),
            repeat_mode: "infinite".to_string(),
            repeat_count: 10,
            repeat_unit: "times".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct JigglerSettings {
    pub active: bool,
    pub distance: u32,
    pub interval_ms: u32,
    pub pattern: String,
}

impl Default for JigglerSettings {
    fn default() -> Self {
        Self {
            active: false,
            distance: 20,
            interval_ms: 30_000,
            pattern: "rnd".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MacroSettings {
    pub repeat_mode: String,
    pub repeat_count: u32,
    pub repeat_duration_ms: u64,
    pub recording_options: MacroRecordingOptions,
    pub actions: Vec<MacroAction>,
}

impl Default for MacroSettings {
    fn default() -> Self {
        Self {
            repeat_mode: "infinite".to_string(),
            repeat_count: 10,
            repeat_duration_ms: 10_000,
            recording_options: MacroRecordingOptions::default(),
            actions: Vec::new(),
        }
    }
}

impl MacroSettings {
    pub fn from_repeat_mode(mode: &MacroRepeatMode) -> (String, u32, u64) {
        match mode {
            MacroRepeatMode::Infinite => ("infinite".to_string(), 10, 10_000),
            MacroRepeatMode::FiniteTimes { count } => ("finite_times".to_string(), *count, 10_000),
            MacroRepeatMode::FiniteSeconds { duration_ms } => {
                ("finite_seconds".to_string(), 10, *duration_ms)
            }
        }
    }

    pub fn to_repeat_mode(&self) -> MacroRepeatMode {
        match self.repeat_mode.as_str() {
            "finite_times" => MacroRepeatMode::FiniteTimes {
                count: self.repeat_count.max(1),
            },
            "finite_seconds" => MacroRepeatMode::FiniteSeconds {
                duration_ms: self.repeat_duration_ms.max(1_000),
            },
            _ => MacroRepeatMode::Infinite,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HotkeySettings {
    pub toggle_start_stop: String,
    pub pick_position: String,
    pub toggle_macro_recording: String,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            toggle_start_stop: "F6".to_string(),
            pick_position: "Ctrl+P".to_string(),
            toggle_macro_recording: "Ctrl+Shift+R".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateSettings {
    pub prerelease_channel: bool,
    pub last_checked_at_unix_ms: u64,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            prerelease_channel: true,
            last_checked_at_unix_ms: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfigFile {
    pub version: u32,
    pub active_profile: String,
    pub general: GeneralSettings,
    pub mouse: MouseSettings,
    pub keyboard: KeyboardSettings,
    pub jiggler: JigglerSettings,
    pub macro_settings: MacroSettings,
    pub hotkeys: HotkeySettings,
    pub updates: UpdateSettings,
    pub frontend_state: serde_json::Value,
}

impl Default for AppConfigFile {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            active_profile: "default".to_string(),
            general: GeneralSettings::default(),
            mouse: MouseSettings::default(),
            keyboard: KeyboardSettings::default(),
            jiggler: JigglerSettings::default(),
            macro_settings: MacroSettings::default(),
            hotkeys: HotkeySettings::default(),
            updates: UpdateSettings::default(),
            frontend_state: serde_json::json!({}),
        }
    }
}

impl AppConfigFile {
    pub fn migrate(mut self) -> Self {
        if self.version < CURRENT_CONFIG_VERSION {
            self.version = CURRENT_CONFIG_VERSION;
        }

        if self.active_profile.trim().is_empty() {
            self.active_profile = "default".to_string();
        }

        self.mouse.cps = self.mouse.cps.clamp(MIN_CPS, MAX_CPS);
        self.mouse.variation_ms = self.mouse.variation_ms.min(MAX_VARIATION_MS);
        self.mouse.hold_duration = self
            .mouse
            .hold_duration
            .clamp(MIN_HOLD_DURATION, MAX_HOLD_DURATION);
        self.mouse.repeat_count = self
            .mouse
            .repeat_count
            .clamp(MIN_REPEAT_COUNT, MAX_REPEAT_COUNT);

        self.keyboard.cps = self.keyboard.cps.clamp(MIN_CPS, MAX_CPS);
        self.keyboard.variation_ms = self.keyboard.variation_ms.min(MAX_VARIATION_MS);
        self.keyboard.hold_duration = self
            .keyboard
            .hold_duration
            .clamp(MIN_HOLD_DURATION, MAX_HOLD_DURATION);
        self.keyboard.repeat_count = self
            .keyboard
            .repeat_count
            .clamp(MIN_REPEAT_COUNT, MAX_REPEAT_COUNT);

        self.jiggler.distance = self
            .jiggler
            .distance
            .clamp(MIN_JIGGLER_DISTANCE, MAX_JIGGLER_DISTANCE);
        self.jiggler.interval_ms = self
            .jiggler
            .interval_ms
            .clamp(MIN_JIGGLER_INTERVAL_MS, MAX_JIGGLER_INTERVAL_MS);

        self.macro_settings.repeat_count = self
            .macro_settings
            .repeat_count
            .clamp(MIN_REPEAT_COUNT, MAX_REPEAT_COUNT);
        self.macro_settings.repeat_duration_ms = self
            .macro_settings
            .repeat_duration_ms
            .clamp(MIN_MACRO_REPEAT_DURATION_MS, MAX_MACRO_REPEAT_DURATION_MS);

        self
    }

    pub fn normalized_for_save(&self) -> Self {
        let mut config = self.clone().migrate();
        config.version = CURRENT_CONFIG_VERSION;
        config
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileFile {
    pub version: u32,
    pub name: String,
    pub data: AppConfigFile,
}

impl ProfileFile {
    pub fn new(name: String, data: AppConfigFile) -> Self {
        Self {
            version: CURRENT_PROFILE_VERSION,
            name,
            data,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupFile {
    pub version: u32,
    pub app_config: AppConfigFile,
    pub profiles: Vec<ProfileFile>,
}

impl BackupFile {
    pub fn new(app_config: AppConfigFile, profiles: Vec<ProfileFile>) -> Self {
        Self {
            version: 1,
            app_config,
            profiles,
        }
    }
}

fn app_config_root() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("FluAutoClicker")
}

fn app_config_path() -> PathBuf {
    app_config_root().join("app_config.json")
}

fn profiles_dir() -> PathBuf {
    app_config_root().join("profiles")
}

fn profile_path(name: &str) -> PathBuf {
    let safe_name = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    profiles_dir().join(format!("{safe_name}.json"))
}

pub async fn load_config() -> Result<AppConfigFile, String> {
    let path = app_config_path();
    if !path.exists() {
        return Ok(AppConfigFile::default());
    }

    let raw = fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read app config: {e}"))?;

    serde_json::from_str::<AppConfigFile>(&raw)
        .map(AppConfigFile::migrate)
        .map_err(|e| format!("Failed to parse app config: {e}"))
}

pub async fn save_config(config: &AppConfigFile) -> Result<(), String> {
    let path = app_config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create config dir: {e}"))?;
    }

    let raw = serde_json::to_string_pretty(&config.normalized_for_save())
        .map_err(|e| format!("Failed to serialize app config: {e}"))?;

    fs::write(&path, raw)
        .await
        .map_err(|e| format!("Failed to write app config: {e}"))
}

pub async fn list_profiles() -> Result<Vec<String>, String> {
    let dir = profiles_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&dir)
        .await
        .map_err(|e| format!("Failed to read profiles dir: {e}"))?;

    let mut names = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to iterate profiles dir: {e}"))?
    {
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|v| v.to_str()) {
                names.push(stem.to_string());
            }
        }
    }

    names.sort();
    Ok(names)
}

pub async fn save_profile(name: &str, data: &AppConfigFile) -> Result<(), String> {
    let path = profile_path(name);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create profiles dir: {e}"))?;
    }

    let profile = ProfileFile::new(name.to_string(), data.normalized_for_save());
    let raw = serde_json::to_string_pretty(&profile)
        .map_err(|e| format!("Failed to serialize profile: {e}"))?;

    fs::write(&path, raw)
        .await
        .map_err(|e| format!("Failed to write profile: {e}"))
}

pub async fn load_profile(name: &str) -> Result<AppConfigFile, String> {
    let path = profile_path(name);
    let raw = fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read profile: {e}"))?;

    let profile = serde_json::from_str::<ProfileFile>(&raw)
        .map_err(|e| format!("Failed to parse profile: {e}"))?;

    Ok(profile.data.migrate())
}

pub async fn load_profile_file(name: &str) -> Result<ProfileFile, String> {
    let path = profile_path(name);
    let raw = fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read profile: {e}"))?;

    serde_json::from_str::<ProfileFile>(&raw).map_err(|e| format!("Failed to parse profile: {e}"))
}

pub async fn delete_profile(name: &str) -> Result<(), String> {
    let path = profile_path(name);
    if path.exists() {
        fs::remove_file(path)
            .await
            .map_err(|e| format!("Failed to delete profile: {e}"))?;
    }
    Ok(())
}

pub async fn rename_profile(old_name: &str, new_name: &str) -> Result<(), String> {
    let old_path = profile_path(old_name);
    let new_path = profile_path(new_name);

    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create profiles dir: {e}"))?;
    }

    fs::rename(old_path, new_path)
        .await
        .map_err(|e| format!("Failed to rename profile: {e}"))
}

pub async fn ensure_default_profile(config: &AppConfigFile) -> Result<(), String> {
    let names = list_profiles().await?;
    if names.iter().any(|n| n == "default") {
        return Ok(());
    }
    save_profile("default", config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_current_version() {
        let config = AppConfigFile::default();
        assert_eq!(config.version, CURRENT_CONFIG_VERSION);
        assert_eq!(config.active_profile, "default");
    }

    #[test]
    fn macro_repeat_mode_round_trip_preserves_values() {
        let settings = MacroSettings {
            repeat_mode: "finite_seconds".to_string(),
            repeat_count: 10,
            repeat_duration_ms: 12_000,
            recording_options: MacroRecordingOptions::default(),
            actions: Vec::new(),
        };

        let repeat_mode = settings.to_repeat_mode();
        let (kind, count, duration_ms) = MacroSettings::from_repeat_mode(&repeat_mode);

        assert_eq!(kind, "finite_seconds");
        assert_eq!(count, 10);
        assert_eq!(duration_ms, 12_000);
    }

    #[test]
    fn legacy_config_payload_is_migrated_with_defaults() {
        let raw = r##"{
            "version": 1,
            "active_profile": "",
            "mouse": {
                "cps": 24
            },
            "keyboard": {
                "key": "z"
            }
        }"##;

        let migrated = serde_json::from_str::<AppConfigFile>(raw)
            .unwrap()
            .migrate();

        assert_eq!(migrated.version, CURRENT_CONFIG_VERSION);
        assert_eq!(migrated.active_profile, "default");
        assert_eq!(migrated.mouse.cps, 24);
        assert_eq!(migrated.keyboard.key, "z");
        assert!(migrated.frontend_state.is_object());
    }

    #[test]
    fn normalized_save_upgrades_version_and_clamps_values() {
        let config = AppConfigFile {
            version: 1,
            active_profile: "default".to_string(),
            mouse: MouseSettings {
                cps: 0,
                hold_duration: 0,
                repeat_count: 0,
                ..MouseSettings::default()
            },
            keyboard: KeyboardSettings {
                cps: 0,
                hold_duration: 0,
                repeat_count: 0,
                ..KeyboardSettings::default()
            },
            jiggler: JigglerSettings {
                distance: 0,
                interval_ms: 0,
                ..JigglerSettings::default()
            },
            macro_settings: MacroSettings {
                repeat_count: 0,
                repeat_duration_ms: 0,
                ..MacroSettings::default()
            },
            ..AppConfigFile::default()
        };

        let normalized = config.normalized_for_save();

        assert_eq!(normalized.version, CURRENT_CONFIG_VERSION);
        assert_eq!(normalized.mouse.cps, 1);
        assert_eq!(normalized.mouse.hold_duration, 1);
        assert_eq!(normalized.keyboard.repeat_count, 1);
        assert_eq!(normalized.jiggler.interval_ms, 100);
        assert_eq!(normalized.macro_settings.repeat_duration_ms, 1_000);
    }
}
