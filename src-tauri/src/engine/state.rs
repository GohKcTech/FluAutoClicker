use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32};
#[cfg(target_os = "linux")]
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Instant;
use tokio::sync::Mutex;

#[cfg(target_os = "linux")]
use evdev::uinput::VirtualDevice;
#[cfg(target_os = "linux")]
use evdev::Key;

use super::macro_engine::state::MacroEngineState;

#[derive(Clone, Copy, Default, PartialEq)]
pub enum MouseButton {
    #[default]
    Left,
    Middle,
    Right,
    Front,
    Back,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum ClickMode {
    #[default]
    Press,
    Hold,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum RepeatMode {
    #[default]
    Infinite,
    Finite,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum RepeatUnit {
    #[default]
    Times,
    Seconds,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum PositionMode {
    #[default]
    Current,
    Custom,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum HoldUnit {
    #[default]
    Milliseconds,
    Seconds,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum JigglerPattern {
    #[default]
    Random,

    Linear,

    Circle,

    OZone,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum KeyboardModifier {
    #[default]
    None,
    Ctrl,
    Shift,
    Alt,
    Win,
    CtrlShift,
    CtrlAlt,
    CtrlWin,
    ShiftAlt,
    ShiftWin,
    AltWin,
    CtrlShiftAlt,
    CtrlShiftWin,
    CtrlAltWin,
    ShiftAltWin,
    CtrlShiftAltWin,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RuntimeHotkeys {
    pub toggle_start_stop: String,
    pub pick_position: String,
}

impl Default for RuntimeHotkeys {
    fn default() -> Self {
        Self {
            toggle_start_stop: "F6".to_string(),
            pick_position: "Ctrl+P".to_string(),
        }
    }
}

pub struct ToggleHotkeyPressState {
    pub started_at: Instant,
    pub was_running_before_press: bool,
}

impl KeyboardModifier {
    #[cfg(target_os = "linux")]
    pub fn to_keys(&self) -> Vec<Key> {
        match self {
            KeyboardModifier::None => vec![],
            KeyboardModifier::Ctrl => vec![Key::KEY_LEFTCTRL],
            KeyboardModifier::Shift => vec![Key::KEY_LEFTSHIFT],
            KeyboardModifier::Alt => vec![Key::KEY_LEFTALT],
            KeyboardModifier::Win => vec![Key::KEY_LEFTMETA],
            KeyboardModifier::CtrlShift => vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT],
            KeyboardModifier::CtrlAlt => vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTALT],
            KeyboardModifier::CtrlWin => vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTMETA],
            KeyboardModifier::ShiftAlt => vec![Key::KEY_LEFTSHIFT, Key::KEY_LEFTALT],
            KeyboardModifier::ShiftWin => vec![Key::KEY_LEFTSHIFT, Key::KEY_LEFTMETA],
            KeyboardModifier::AltWin => vec![Key::KEY_LEFTALT, Key::KEY_LEFTMETA],
            KeyboardModifier::CtrlShiftAlt => {
                vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT, Key::KEY_LEFTALT]
            }
            KeyboardModifier::CtrlShiftWin => {
                vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT, Key::KEY_LEFTMETA]
            }
            KeyboardModifier::CtrlAltWin => {
                vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTALT, Key::KEY_LEFTMETA]
            }
            KeyboardModifier::ShiftAltWin => {
                vec![Key::KEY_LEFTSHIFT, Key::KEY_LEFTALT, Key::KEY_LEFTMETA]
            }
            KeyboardModifier::CtrlShiftAltWin => vec![
                Key::KEY_LEFTCTRL,
                Key::KEY_LEFTSHIFT,
                Key::KEY_LEFTALT,
                Key::KEY_LEFTMETA,
            ],
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "ctrl" => KeyboardModifier::Ctrl,
            "shift" => KeyboardModifier::Shift,
            "alt" => KeyboardModifier::Alt,
            "win" => KeyboardModifier::Win,
            "ctrl+shift" | "shift+ctrl" => KeyboardModifier::CtrlShift,
            "ctrl+alt" | "alt+ctrl" => KeyboardModifier::CtrlAlt,
            "ctrl+win" | "win+ctrl" => KeyboardModifier::CtrlWin,
            "shift+alt" | "alt+shift" => KeyboardModifier::ShiftAlt,
            "shift+win" | "win+shift" => KeyboardModifier::ShiftWin,
            "alt+win" | "win+alt" => KeyboardModifier::AltWin,
            "ctrl+shift+alt" | "ctrl+alt+shift" | "shift+ctrl+alt" => {
                KeyboardModifier::CtrlShiftAlt
            }
            "ctrl+shift+win" | "ctrl+win+shift" | "shift+ctrl+win" => {
                KeyboardModifier::CtrlShiftWin
            }
            "ctrl+alt+win" | "ctrl+win+alt" | "alt+ctrl+win" => KeyboardModifier::CtrlAltWin,
            "shift+alt+win" | "shift+win+alt" | "alt+shift+win" => KeyboardModifier::ShiftAltWin,
            "ctrl+shift+alt+win" | "ctrl+shift+win+alt" | "ctrl+alt+shift+win" => {
                KeyboardModifier::CtrlShiftAltWin
            }
            _ => KeyboardModifier::None,
        }
    }

    #[allow(dead_code)]
    pub fn to_display_string(&self) -> String {
        match self {
            KeyboardModifier::None => "".to_string(),
            KeyboardModifier::Ctrl => "Ctrl".to_string(),
            KeyboardModifier::Shift => "Shift".to_string(),
            KeyboardModifier::Alt => "Alt".to_string(),
            KeyboardModifier::Win => "Win".to_string(),
            KeyboardModifier::CtrlShift => "Ctrl+Shift".to_string(),
            KeyboardModifier::CtrlAlt => "Ctrl+Alt".to_string(),
            KeyboardModifier::CtrlWin => "Ctrl+Win".to_string(),
            KeyboardModifier::ShiftAlt => "Shift+Alt".to_string(),
            KeyboardModifier::ShiftWin => "Shift+Win".to_string(),
            KeyboardModifier::AltWin => "Alt+Win".to_string(),
            KeyboardModifier::CtrlShiftAlt => "Ctrl+Shift+Alt".to_string(),
            KeyboardModifier::CtrlShiftWin => "Ctrl+Shift+Win".to_string(),
            KeyboardModifier::CtrlAltWin => "Ctrl+Alt+Win".to_string(),
            KeyboardModifier::ShiftAltWin => "Shift+Alt+Win".to_string(),
            KeyboardModifier::CtrlShiftAltWin => "Ctrl+Shift+Alt+Win".to_string(),
        }
    }
}

pub struct AppState {
    pub is_running: AtomicBool,
    pub is_main_focused: AtomicBool,
    pub is_cps_test_focused: AtomicBool,
    pub is_jiggler_active: AtomicBool,
    pub is_multithread_active: AtomicBool,
    pub minimize_to_tray: AtomicBool,
    pub is_quitting: AtomicBool,
    pub stop_on_custom_position_move: AtomicBool,
    pub cps: AtomicU32,
    pub jiggler_distance: AtomicU32,
    pub jiggler_interval: AtomicU32,
    pub jiggler_pattern: Mutex<JigglerPattern>,
    pub threads_count: AtomicU32,
    pub hotkeys: Mutex<RuntimeHotkeys>,
    pub hotkeys_suspended: AtomicBool,
    pub toggle_hotkey_press_state: StdMutex<Option<ToggleHotkeyPressState>>,

    pub mouse_button: Mutex<MouseButton>,
    pub click_mode: Mutex<ClickMode>,
    pub hold_duration: AtomicU32,
    pub hold_unit: Mutex<HoldUnit>,
    pub repeat_mode: Mutex<RepeatMode>,
    pub repeat_count: AtomicU32,
    pub repeat_unit: Mutex<RepeatUnit>,
    pub position_mode: Mutex<PositionMode>,
    pub coord_x: AtomicI32,
    pub coord_y: AtomicI32,
    pub variation_ms: AtomicU32,

    pub keyboard_key: Mutex<String>,
    pub keyboard_modifiers: Mutex<KeyboardModifier>,
    pub kb_click_mode: Mutex<ClickMode>,
    pub kb_hold_duration: AtomicU32,
    pub kb_hold_unit: Mutex<HoldUnit>,
    pub kb_repeat_mode: Mutex<RepeatMode>,
    pub kb_repeat_count: AtomicU32,
    pub kb_repeat_unit: Mutex<RepeatUnit>,
    pub kb_is_running: AtomicBool,
    pub kb_cps: AtomicU32,
    pub kb_variation_ms: AtomicU32,

    #[cfg(target_os = "linux")]
    pub keyboard_uinput_device: Arc<Mutex<Option<VirtualDevice>>>,

    #[cfg(target_os = "linux")]
    pub uinput_device: Arc<Mutex<Option<VirtualDevice>>>,

    pub macro_engine: MacroEngineState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_running: AtomicBool::new(false),
            is_main_focused: AtomicBool::new(true),
            is_cps_test_focused: AtomicBool::new(false),
            is_jiggler_active: AtomicBool::new(false),
            is_multithread_active: AtomicBool::new(false),
            minimize_to_tray: AtomicBool::new(true),
            is_quitting: AtomicBool::new(false),
            stop_on_custom_position_move: AtomicBool::new(true),
            cps: AtomicU32::new(10),
            jiggler_distance: AtomicU32::new(10),
            jiggler_interval: AtomicU32::new(1000),
            jiggler_pattern: Mutex::new(JigglerPattern::Random),
            threads_count: AtomicU32::new(1),
            hotkeys: Mutex::new(RuntimeHotkeys::default()),
            hotkeys_suspended: AtomicBool::new(false),
            toggle_hotkey_press_state: StdMutex::new(None),

            mouse_button: Mutex::new(MouseButton::Left),
            click_mode: Mutex::new(ClickMode::Press),
            hold_duration: AtomicU32::new(100),
            hold_unit: Mutex::new(HoldUnit::Milliseconds),
            repeat_mode: Mutex::new(RepeatMode::Infinite),
            repeat_count: AtomicU32::new(10),
            repeat_unit: Mutex::new(RepeatUnit::Times),
            position_mode: Mutex::new(PositionMode::Current),
            coord_x: AtomicI32::new(841),
            coord_y: AtomicI32::new(425),
            variation_ms: AtomicU32::new(0),

            keyboard_key: Mutex::new("a".to_string()),
            keyboard_modifiers: Mutex::new(KeyboardModifier::None),
            kb_click_mode: Mutex::new(ClickMode::Press),
            kb_hold_duration: AtomicU32::new(100),
            kb_hold_unit: Mutex::new(HoldUnit::Milliseconds),
            kb_repeat_mode: Mutex::new(RepeatMode::Infinite),
            kb_repeat_count: AtomicU32::new(10),
            kb_repeat_unit: Mutex::new(RepeatUnit::Times),
            kb_is_running: AtomicBool::new(false),
            kb_cps: AtomicU32::new(10),
            kb_variation_ms: AtomicU32::new(0),

            #[cfg(target_os = "linux")]
            keyboard_uinput_device: Arc::new(Mutex::new(None)),

            #[cfg(target_os = "linux")]
            uinput_device: Arc::new(Mutex::new(None)),

            macro_engine: MacroEngineState::default(),
        }
    }
}
