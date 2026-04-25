use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MacroMouseButton {
    Left,
    Middle,
    Right,
    Front,
    Back,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MacroMouseAction {
    Press,
    Hold { duration_ms: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MacroMoveStyle {
    Instant,
    Smooth { duration_ms: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MacroActionConfig {
    Mouse {
        button: MacroMouseButton,
        action: MacroMouseAction,
        position: Option<(i32, i32)>,
    },
    Move {
        x: i32,
        y: i32,
        style: MacroMoveStyle,
    },
    Keyboard {
        key: String,
        #[serde(default)]
        text: Option<String>,
        modifiers: Vec<String>,
        action: MacroKeyboardAction,
    },
    Sleep {
        duration_ms: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MacroKeyboardAction {
    Press,
    Hold { duration_ms: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MacroAction {
    pub id: u64,
    pub config: MacroActionConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MacroRepeatMode {
    Infinite,
    FiniteTimes { count: u32 },
    FiniteSeconds { duration_ms: u64 },
}

impl Default for MacroRepeatMode {
    fn default() -> Self {
        MacroRepeatMode::Infinite
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct MacroRecordingOptions {
    pub record_mouse_clicks: bool,
    pub record_mouse_moves: bool,
    pub record_keyboard: bool,
    pub record_delays: bool,
    pub record_click_position: bool,
}

impl Default for MacroRecordingOptions {
    fn default() -> Self {
        Self {
            record_mouse_clicks: true,
            record_mouse_moves: true,
            record_keyboard: true,
            record_delays: true,
            record_click_position: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MacroPlayerState {
    Stopped,
    Playing,
    Recording,
}

impl Default for MacroPlayerState {
    fn default() -> Self {
        MacroPlayerState::Stopped
    }
}
