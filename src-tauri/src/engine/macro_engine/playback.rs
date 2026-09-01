#[cfg(not(target_os = "linux"))]
use enigo::{Axis, Button, Coordinate, Enigo, Key, Keyboard, Mouse, Settings};
#[cfg(target_os = "linux")]
use enigo::{Enigo, Mouse, Settings};
#[cfg(target_os = "linux")]
use evdev::{EventType, InputEvent, Key, RelativeAxisType};
use std::time::Duration;
use tauri::Emitter;
use tokio::time::sleep;

use crate::engine::clicker::{ExecutionError, RunEnd};
use crate::engine::executor::{interruptible_wait, Cancellation, WaitOutcome};
use crate::engine::input::{InputError, InputMouseButton, InputSession, InputSink, InputToken};
use crate::engine::runtime::{MacroRunControl, StopPolicy};
use crate::engine::state::AppState;

use super::types::{
    MacroAction, MacroActionConfig, MacroKeyboardAction, MacroMouseAction, MacroMouseButton,
    MacroMoveStyle, MacroPlayerState,
};

#[cfg(not(target_os = "linux"))]
async fn execute_action(
    enigo: &mut Enigo,
    action: &MacroAction,
    multiplier: f64,
) -> Result<(), String> {
    match &action.config {
        MacroActionConfig::Mouse {
            button,
            action: mouse_action,
            position,
        } => {
            if let Some((x, y)) = position {
                enigo
                    .move_mouse(*x, *y, Coordinate::Abs)
                    .map_err(|e| format!("Could not move the mouse. Details: {}", e))?;

                sleep(Duration::from_millis(
                    (10.0 / multiplier).round().max(1.0) as u64
                ))
                .await;
            }

            match mouse_action {
                MacroMouseAction::Press => {
                    let btn = macro_button_to_enigo(button)?;
                    enigo
                        .button(btn, enigo::Direction::Press)
                        .map_err(|e| format!("Could not press the mouse button. Details: {}", e))?;
                    sleep(Duration::from_millis(
                        (50.0 / multiplier).round().max(1.0) as u64
                    ))
                    .await;
                    enigo.button(btn, enigo::Direction::Release).map_err(|e| {
                        format!("Could not release the mouse button. Details: {}", e)
                    })?;
                }
                MacroMouseAction::Hold { duration_ms } => {
                    let btn = macro_button_to_enigo(button)?;
                    enigo
                        .button(btn, enigo::Direction::Press)
                        .map_err(|e| format!("Could not press the mouse button. Details: {}", e))?;
                    sleep(Duration::from_millis(
                        (*duration_ms as f64 / multiplier).round() as u64,
                    ))
                    .await;
                    enigo.button(btn, enigo::Direction::Release).map_err(|e| {
                        format!("Could not release the mouse button. Details: {}", e)
                    })?;
                }
                MacroMouseAction::Down => {
                    let btn = macro_button_to_enigo(button)?;
                    enigo
                        .button(btn, enigo::Direction::Press)
                        .map_err(|e| format!("Could not press the mouse button. Details: {}", e))?;
                }
                MacroMouseAction::Up => {
                    let btn = macro_button_to_enigo(button)?;
                    enigo.button(btn, enigo::Direction::Release).map_err(|e| {
                        format!("Could not release the mouse button. Details: {}", e)
                    })?;
                }
            }
        }
        MacroActionConfig::Move { x, y, style } => match style {
            MacroMoveStyle::Instant => {
                enigo
                    .move_mouse(*x, *y, Coordinate::Abs)
                    .map_err(|e| format!("Could not move the mouse. Details: {}", e))?;
            }
            MacroMoveStyle::Linear { duration_ms } => {
                linear_move_to(
                    enigo,
                    *x,
                    *y,
                    (*duration_ms as f64 / multiplier).round() as u32,
                )
                .await?;
            }
            MacroMoveStyle::Smooth { path, duration_ms } => {
                play_smooth_path(
                    enigo,
                    path,
                    (*duration_ms as f64 / multiplier).round() as u32,
                )
                .await?;
            }
        },
        MacroActionConfig::RawMove { points } => {
            play_raw_move(enigo, points, multiplier).await?;
        }
        MacroActionConfig::Keyboard {
            key,
            text,
            modifiers,
            action,
        } => {
            if let Some(recorded_text) = text.as_deref() {
                if modifiers.is_empty() && matches!(action, MacroKeyboardAction::Press) {
                    enigo
                        .text(recorded_text)
                        .map_err(|e| format!("Could not type the recorded text. Details: {}", e))?;
                    return Ok(());
                }
            }

            for modifier in modifiers {
                if let Some(key_code) = modifier_to_key(modifier) {
                    enigo
                        .key(key_code, enigo::Direction::Press)
                        .map_err(|e| format!("Could not hold a shortcut key. Details: {}", e))?;
                }
            }

            let key_code = if modifiers.is_empty() {
                str_to_key(key)
            } else {
                str_to_combo_key(key)
            };

            match action {
                MacroKeyboardAction::Press => {
                    enigo
                        .key(key_code, enigo::Direction::Click)
                        .map_err(|e| format!("Could not press the key. Details: {}", e))?;
                }
                MacroKeyboardAction::Hold { duration_ms } => {
                    enigo
                        .key(key_code, enigo::Direction::Press)
                        .map_err(|e| format!("Could not press the key. Details: {}", e))?;
                    sleep(Duration::from_millis(
                        (*duration_ms as f64 / multiplier).round() as u64,
                    ))
                    .await;
                    enigo
                        .key(key_code, enigo::Direction::Release)
                        .map_err(|e| format!("Could not release the key. Details: {}", e))?;
                }
                MacroKeyboardAction::Down => {
                    enigo
                        .key(key_code, enigo::Direction::Press)
                        .map_err(|e| format!("Could not press the key. Details: {}", e))?;
                }
                MacroKeyboardAction::Up => {
                    enigo
                        .key(key_code, enigo::Direction::Release)
                        .map_err(|e| format!("Could not release the key. Details: {}", e))?;
                }
            }

            for modifier in modifiers.iter().rev() {
                if let Some(key_code) = modifier_to_key(modifier) {
                    enigo
                        .key(key_code, enigo::Direction::Release)
                        .map_err(|e| format!("Could not release a shortcut key. Details: {}", e))?;
                }
            }
        }
        MacroActionConfig::Sleep { duration_ms } => {
            sleep(Duration::from_millis(
                (*duration_ms as f64 / multiplier).round() as u64,
            ))
            .await;
        }
        MacroActionConfig::Scroll { clicks } => {
            enigo
                .scroll(-*clicks, Axis::Vertical)
                .map_err(|e| format!("Could not scroll mouse wheel. Details: {}", e))?;
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
struct LinuxPlaybackBackend {
    mouse: evdev::uinput::VirtualDevice,
    keyboard: evdev::uinput::VirtualDevice,
}

#[cfg(target_os = "linux")]
impl LinuxPlaybackBackend {
    fn new() -> Result<Self, String> {
        let mouse = crate::engine::uinput::setup_uinput().ok_or_else(uinput_error)?;
        let keyboard =
            crate::engine::keyboard_uinput::setup_keyboard_uinput().ok_or_else(uinput_error)?;

        Ok(Self { mouse, keyboard })
    }

    fn emit_key(
        device: &mut evdev::uinput::VirtualDevice,
        key: Key,
        pressed: bool,
    ) -> Result<(), String> {
        device
            .emit(&[
                InputEvent::new(EventType::KEY, key.0, i32::from(pressed)),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ])
            .map_err(|e| format!("Could not emit input event. Details: {e}"))
    }

    fn move_mouse(&mut self, x: i32, y: i32) -> Result<(), String> {
        self.mouse
            .emit(&[
                InputEvent::new(EventType::ABSOLUTE, evdev::AbsoluteAxisType::ABS_X.0, x),
                InputEvent::new(EventType::ABSOLUTE, evdev::AbsoluteAxisType::ABS_Y.0, y),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ])
            .map_err(|e| format!("Could not move the mouse. Details: {e}"))
    }

    async fn click_mouse(
        &mut self,
        button: &crate::engine::macro_engine::types::MacroMouseButton,
        action: &MacroMouseAction,
        multiplier: f64,
    ) -> Result<(), String> {
        let key = macro_button_to_evdev(button);
        match action {
            MacroMouseAction::Press => {
                Self::emit_key(&mut self.mouse, key, true)?;
                sleep(Duration::from_millis(
                    (50.0 / multiplier).round().max(1.0) as u64
                ))
                .await;
                Self::emit_key(&mut self.mouse, key, false)
            }
            MacroMouseAction::Hold { duration_ms } => {
                Self::emit_key(&mut self.mouse, key, true)?;
                sleep(Duration::from_millis(
                    (*duration_ms as f64 / multiplier).round() as u64,
                ))
                .await;
                Self::emit_key(&mut self.mouse, key, false)
            }
            MacroMouseAction::Down => Self::emit_key(&mut self.mouse, key, true),
            MacroMouseAction::Up => Self::emit_key(&mut self.mouse, key, false),
        }
    }

    fn press_keyboard_key(&mut self, key: Key) -> Result<(), String> {
        Self::emit_key(&mut self.keyboard, key, true)?;
        Self::emit_key(&mut self.keyboard, key, false)
    }

    fn scroll_mouse(&mut self, clicks: i32) -> Result<(), String> {
        self.mouse
            .emit(&[
                InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_WHEEL.0, -clicks),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ])
            .map_err(|e| format!("Could not scroll mouse. Details: {e}"))
    }
}

#[cfg(target_os = "linux")]
fn uinput_error() -> String {
    "Could not initialize Linux virtual input devices. Make sure /dev/uinput exists and is writable for your user, then try again.".to_string()
}

#[cfg(target_os = "linux")]
async fn execute_action(
    backend: &mut LinuxPlaybackBackend,
    action: &MacroAction,
    multiplier: f64,
) -> Result<(), String> {
    match &action.config {
        MacroActionConfig::Mouse {
            button,
            action: mouse_action,
            position,
        } => {
            if let Some((x, y)) = position {
                backend.move_mouse(*x, *y)?;
                sleep(Duration::from_millis(
                    (10.0 / multiplier).round().max(1.0) as u64
                ))
                .await;
            }

            backend
                .click_mouse(button, mouse_action, multiplier)
                .await?;
        }
        MacroActionConfig::Move { x, y, style } => match style {
            MacroMoveStyle::Instant => backend.move_mouse(*x, *y)?,
            MacroMoveStyle::Linear { duration_ms } => {
                linear_move_to(
                    backend,
                    *x,
                    *y,
                    (*duration_ms as f64 / multiplier).round() as u32,
                )
                .await?;
            }
            MacroMoveStyle::Smooth { path, duration_ms } => {
                play_smooth_path(
                    backend,
                    path,
                    (*duration_ms as f64 / multiplier).round() as u32,
                )
                .await?;
            }
        },
        MacroActionConfig::RawMove { points } => {
            play_raw_move(backend, points, multiplier).await?;
        }
        MacroActionConfig::Keyboard {
            key,
            text,
            modifiers,
            action,
        } => {
            if let Some(recorded_text) = text.as_deref() {
                if modifiers.is_empty() && matches!(action, MacroKeyboardAction::Press) {
                    type_text(backend, recorded_text).await?;
                    return Ok(());
                }
            }

            let modifier_keys = modifiers_to_evdev(modifiers)?;
            for &modifier_key in &modifier_keys {
                LinuxPlaybackBackend::emit_key(&mut backend.keyboard, modifier_key, true)?;
            }

            let key_code = str_to_evdev_key(key).ok_or_else(|| {
                format!("The key `{key}` is not supported by Linux macro playback.")
            })?;

            match action {
                MacroKeyboardAction::Press => {
                    backend.press_keyboard_key(key_code)?;
                }
                MacroKeyboardAction::Hold { duration_ms } => {
                    LinuxPlaybackBackend::emit_key(&mut backend.keyboard, key_code, true)?;
                    sleep(Duration::from_millis(
                        (*duration_ms as f64 / multiplier).round() as u64,
                    ))
                    .await;
                    LinuxPlaybackBackend::emit_key(&mut backend.keyboard, key_code, false)?;
                }
                MacroKeyboardAction::Down => {
                    LinuxPlaybackBackend::emit_key(&mut backend.keyboard, key_code, true)?;
                }
                MacroKeyboardAction::Up => {
                    LinuxPlaybackBackend::emit_key(&mut backend.keyboard, key_code, false)?;
                }
            }

            for &modifier_key in modifier_keys.iter().rev() {
                LinuxPlaybackBackend::emit_key(&mut backend.keyboard, modifier_key, false)?;
            }
        }
        MacroActionConfig::Sleep { duration_ms } => {
            sleep(Duration::from_millis(
                (*duration_ms as f64 / multiplier).round() as u64,
            ))
            .await;
        }
        MacroActionConfig::Scroll { clicks } => {
            backend.scroll_mouse(*clicks)?;
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn macro_button_to_enigo(
    button: &crate::engine::macro_engine::types::MacroMouseButton,
) -> Result<Button, String> {
    match button {
        crate::engine::macro_engine::types::MacroMouseButton::Left => Ok(Button::Left),
        crate::engine::macro_engine::types::MacroMouseButton::Middle => Ok(Button::Middle),
        crate::engine::macro_engine::types::MacroMouseButton::Right => Ok(Button::Right),
        crate::engine::macro_engine::types::MacroMouseButton::Front => map_extended_button("front"),
        crate::engine::macro_engine::types::MacroMouseButton::Back => map_back_button(),
    }
}

#[cfg(target_os = "linux")]
fn macro_button_to_evdev(button: &crate::engine::macro_engine::types::MacroMouseButton) -> Key {
    match button {
        crate::engine::macro_engine::types::MacroMouseButton::Left => Key::BTN_LEFT,
        crate::engine::macro_engine::types::MacroMouseButton::Middle => Key::BTN_MIDDLE,
        crate::engine::macro_engine::types::MacroMouseButton::Right => Key::BTN_RIGHT,
        crate::engine::macro_engine::types::MacroMouseButton::Front => Key::BTN_SIDE,
        crate::engine::macro_engine::types::MacroMouseButton::Back => Key::BTN_EXTRA,
    }
}

#[cfg(not(target_os = "linux"))]
#[cfg(target_os = "macos")]
fn map_extended_button(_name: &str) -> Result<Button, String> {
    Err("Front and back mouse buttons are not supported on macOS in this build.".to_string())
}

#[cfg(not(target_os = "linux"))]
#[cfg(not(target_os = "macos"))]
fn map_extended_button(_name: &str) -> Result<Button, String> {
    Ok(Button::Forward)
}

#[cfg(not(target_os = "linux"))]
#[cfg(target_os = "macos")]
fn map_back_button() -> Result<Button, String> {
    Err("Front and back mouse buttons are not supported on macOS in this build.".to_string())
}

#[cfg(not(target_os = "linux"))]
#[cfg(not(target_os = "macos"))]
fn map_back_button() -> Result<Button, String> {
    Ok(Button::Back)
}

#[cfg(not(target_os = "linux"))]
fn modifier_to_key(modifier: &str) -> Option<Key> {
    match modifier.to_lowercase().as_str() {
        "ctrl" => Some(Key::Control),
        "shift" => Some(Key::Shift),
        "alt" => Some(Key::Alt),
        "win" => Some(Key::Meta),
        _ => None,
    }
}

#[cfg(not(target_os = "linux"))]
fn str_to_key(key: &str) -> Key {
    match key.to_lowercase().as_str() {
        "a" => return Key::A,
        "b" => return Key::B,
        "c" => return Key::C,
        "d" => return Key::D,
        "e" => return Key::E,
        "f" => return Key::F,
        "g" => return Key::G,
        "h" => return Key::H,
        "i" => return Key::I,
        "j" => return Key::J,
        "k" => return Key::K,
        "l" => return Key::L,
        "m" => return Key::M,
        "n" => return Key::N,
        "o" => return Key::O,
        "p" => return Key::P,
        "q" => return Key::Q,
        "r" => return Key::R,
        "s" => return Key::S,
        "t" => return Key::T,
        "u" => return Key::U,
        "v" => return Key::V,
        "w" => return Key::W,
        "x" => return Key::X,
        "y" => return Key::Y,
        "z" => return Key::Z,
        "0" => return Key::Num0,
        "1" => return Key::Num1,
        "2" => return Key::Num2,
        "3" => return Key::Num3,
        "4" => return Key::Num4,
        "5" => return Key::Num5,
        "6" => return Key::Num6,
        "7" => return Key::Num7,
        "8" => return Key::Num8,
        "9" => return Key::Num9,
        "space" => return Key::Space,
        "enter" => return Key::Return,
        "tab" => return Key::Tab,
        "backspace" => return Key::Backspace,
        "escape" => return Key::Escape,
        "delete" => return Key::Delete,
        "insert" => return Key::Insert,
        "home" => return Key::Home,
        "end" => return Key::End,
        "pageup" => return Key::PageUp,
        "pagedown" => return Key::PageDown,
        "up" => return Key::UpArrow,
        "down" => return Key::DownArrow,
        "left" => return Key::LeftArrow,
        "right" => return Key::RightArrow,
        "f1" => return Key::F1,
        "f2" => return Key::F2,
        "f3" => return Key::F3,
        "f4" => return Key::F4,
        "f5" => return Key::F5,
        "f6" => return Key::F6,
        "f7" => return Key::F7,
        "f8" => return Key::F8,
        "f9" => return Key::F9,
        "f10" => return Key::F10,
        "f11" => return Key::F11,
        "f12" => return Key::F12,
        "numpad0" => return Key::Num0,
        "numpad1" => return Key::Num1,
        "numpad2" => return Key::Num2,
        "numpad3" => return Key::Num3,
        "numpad4" => return Key::Num4,
        "numpad5" => return Key::Num5,
        "numpad6" => return Key::Num6,
        "numpad7" => return Key::Num7,
        "numpad8" => return Key::Num8,
        "numpad9" => return Key::Num9,
        "ctrl" | "control" => return Key::Control,
        "shift" => return Key::Shift,
        "alt" => return Key::Alt,
        "win" | "meta" | "super" => return Key::Meta,
        _ => {}
    }

    if key.len() == 1 {
        let c = key.chars().next().unwrap();
        return Key::Unicode(c);
    }

    Key::Unicode(key.chars().next().unwrap_or('a'))
}

#[cfg(not(target_os = "linux"))]
fn str_to_combo_key(key: &str) -> Key {
    str_to_key(key)
}

#[cfg(target_os = "linux")]
fn str_to_evdev_key(key: &str) -> Option<Key> {
    match key.to_lowercase().as_str() {
        "ctrl" | "control" | "controlleft" | "ctrlleft" => Some(Key::KEY_LEFTCTRL),
        "controlright" | "ctrlright" => Some(Key::KEY_RIGHTCTRL),
        "shift" | "shiftleft" => Some(Key::KEY_LEFTSHIFT),
        "shiftright" => Some(Key::KEY_RIGHTSHIFT),
        "alt" | "altleft" => Some(Key::KEY_LEFTALT),
        "altright" | "altgr" => Some(Key::KEY_RIGHTALT),
        "win" | "meta" | "super" | "metaleft" | "winleft" | "superleft" => Some(Key::KEY_LEFTMETA),
        "metaright" | "winright" | "superright" => Some(Key::KEY_RIGHTMETA),
        "a" => Some(Key::KEY_A),
        "b" => Some(Key::KEY_B),
        "c" => Some(Key::KEY_C),
        "d" => Some(Key::KEY_D),
        "e" => Some(Key::KEY_E),
        "f" => Some(Key::KEY_F),
        "g" => Some(Key::KEY_G),
        "h" => Some(Key::KEY_H),
        "i" => Some(Key::KEY_I),
        "j" => Some(Key::KEY_J),
        "k" => Some(Key::KEY_K),
        "l" => Some(Key::KEY_L),
        "m" => Some(Key::KEY_M),
        "n" => Some(Key::KEY_N),
        "o" => Some(Key::KEY_O),
        "p" => Some(Key::KEY_P),
        "q" => Some(Key::KEY_Q),
        "r" => Some(Key::KEY_R),
        "s" => Some(Key::KEY_S),
        "t" => Some(Key::KEY_T),
        "u" => Some(Key::KEY_U),
        "v" => Some(Key::KEY_V),
        "w" => Some(Key::KEY_W),
        "x" => Some(Key::KEY_X),
        "y" => Some(Key::KEY_Y),
        "z" => Some(Key::KEY_Z),
        "0" => Some(Key::KEY_0),
        "1" => Some(Key::KEY_1),
        "2" => Some(Key::KEY_2),
        "3" => Some(Key::KEY_3),
        "4" => Some(Key::KEY_4),
        "5" => Some(Key::KEY_5),
        "6" => Some(Key::KEY_6),
        "7" => Some(Key::KEY_7),
        "8" => Some(Key::KEY_8),
        "9" => Some(Key::KEY_9),
        "f1" => Some(Key::KEY_F1),
        "f2" => Some(Key::KEY_F2),
        "f3" => Some(Key::KEY_F3),
        "f4" => Some(Key::KEY_F4),
        "f5" => Some(Key::KEY_F5),
        "f6" => Some(Key::KEY_F6),
        "f7" => Some(Key::KEY_F7),
        "f8" => Some(Key::KEY_F8),
        "f9" => Some(Key::KEY_F9),
        "f10" => Some(Key::KEY_F10),
        "f11" => Some(Key::KEY_F11),
        "f12" => Some(Key::KEY_F12),
        "escape" | "esc" => Some(Key::KEY_ESC),
        "space" => Some(Key::KEY_SPACE),
        "enter" | "return" => Some(Key::KEY_ENTER),
        "tab" => Some(Key::KEY_TAB),
        "backspace" | "back" => Some(Key::KEY_BACKSPACE),
        "delete" | "del" => Some(Key::KEY_DELETE),
        "insert" | "ins" => Some(Key::KEY_INSERT),
        "home" => Some(Key::KEY_HOME),
        "end" => Some(Key::KEY_END),
        "pageup" | "page_up" | "pgup" => Some(Key::KEY_PAGEUP),
        "pagedown" | "page_down" | "pgdn" => Some(Key::KEY_PAGEDOWN),
        "arrowup" | "up" => Some(Key::KEY_UP),
        "arrowdown" | "down" => Some(Key::KEY_DOWN),
        "arrowleft" | "left" => Some(Key::KEY_LEFT),
        "arrowright" | "right" => Some(Key::KEY_RIGHT),
        "grave" | "`" => Some(Key::KEY_GRAVE),
        "minus" | "-" => Some(Key::KEY_MINUS),
        "equal" | "=" => Some(Key::KEY_EQUAL),
        "leftbrace" | "[" => Some(Key::KEY_LEFTBRACE),
        "rightbrace" | "]" => Some(Key::KEY_RIGHTBRACE),
        "backslash" | "\\" => Some(Key::KEY_BACKSLASH),
        "semicolon" | ";" => Some(Key::KEY_SEMICOLON),
        "apostrophe" | "'" => Some(Key::KEY_APOSTROPHE),
        "comma" | "," => Some(Key::KEY_COMMA),
        "dot" | "." => Some(Key::KEY_DOT),
        "slash" | "/" => Some(Key::KEY_SLASH),
        "kp0" | "numpad0" => Some(Key::KEY_KP0),
        "kp1" | "numpad1" => Some(Key::KEY_KP1),
        "kp2" | "numpad2" => Some(Key::KEY_KP2),
        "kp3" | "numpad3" => Some(Key::KEY_KP3),
        "kp4" | "numpad4" => Some(Key::KEY_KP4),
        "kp5" | "numpad5" => Some(Key::KEY_KP5),
        "kp6" | "numpad6" => Some(Key::KEY_KP6),
        "kp7" | "numpad7" => Some(Key::KEY_KP7),
        "kp8" | "numpad8" => Some(Key::KEY_KP8),
        "kp9" | "numpad9" => Some(Key::KEY_KP9),
        "kpenter" | "numpad_enter" => Some(Key::KEY_KPENTER),
        "kpplus" | "kp+" | "numpad_+" => Some(Key::KEY_KPPLUS),
        "kpminus" | "kp-" | "numpad_-" => Some(Key::KEY_KPMINUS),
        "kpasterisk" | "kp*" | "numpad_*" => Some(Key::KEY_KPASTERISK),
        "kpdot" | "kp." | "numpad_." => Some(Key::KEY_KPDOT),
        "kpslash" | "kp/" | "numpad_/" => Some(Key::KEY_KPSLASH),
        "numlock" | "num_lock" => Some(Key::KEY_NUMLOCK),
        "capslock" | "caps" => Some(Key::KEY_CAPSLOCK),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn modifiers_to_evdev(modifiers: &[String]) -> Result<Vec<Key>, String> {
    let mut keys = Vec::new();
    for modifier in modifiers {
        match modifier.to_lowercase().as_str() {
            "ctrl" | "control" => keys.push(Key::KEY_LEFTCTRL),
            "shift" => keys.push(Key::KEY_LEFTSHIFT),
            "alt" => keys.push(Key::KEY_LEFTALT),
            "win" | "meta" | "super" => keys.push(Key::KEY_LEFTMETA),
            "" => {}
            other => {
                return Err(format!(
                    "The modifier `{other}` is not supported by Linux macro playback."
                ))
            }
        }
    }
    Ok(keys)
}

#[cfg(target_os = "linux")]
fn char_to_evdev(ch: char) -> Option<(Key, bool)> {
    let lower = ch.to_ascii_lowercase();
    let needs_shift = ch.is_ascii_uppercase()
        || matches!(
            ch,
            '!' | '@'
                | '#'
                | '$'
                | '%'
                | '^'
                | '&'
                | '*'
                | '('
                | ')'
                | '_'
                | '+'
                | '{'
                | '}'
                | '|'
                | ':'
                | '"'
                | '<'
                | '>'
                | '?'
        );

    let key = match lower {
        'a'..='z' | '0'..='9' => str_to_evdev_key(&lower.to_string())?,
        ' ' => Key::KEY_SPACE,
        '\n' => Key::KEY_ENTER,
        '\t' => Key::KEY_TAB,
        '!' => Key::KEY_1,
        '@' => Key::KEY_2,
        '#' => Key::KEY_3,
        '$' => Key::KEY_4,
        '%' => Key::KEY_5,
        '^' => Key::KEY_6,
        '&' => Key::KEY_7,
        '*' => Key::KEY_8,
        '(' => Key::KEY_9,
        ')' => Key::KEY_0,
        '`' | '~' => Key::KEY_GRAVE,
        '-' | '_' => Key::KEY_MINUS,
        '=' | '+' => Key::KEY_EQUAL,
        '[' | '{' => Key::KEY_LEFTBRACE,
        ']' | '}' => Key::KEY_RIGHTBRACE,
        '\\' | '|' => Key::KEY_BACKSLASH,
        ';' | ':' => Key::KEY_SEMICOLON,
        '\'' | '"' => Key::KEY_APOSTROPHE,
        ',' | '<' => Key::KEY_COMMA,
        '.' | '>' => Key::KEY_DOT,
        '/' | '?' => Key::KEY_SLASH,
        _ => return None,
    };

    Some((key, needs_shift))
}

#[cfg(target_os = "linux")]
async fn type_text(backend: &mut LinuxPlaybackBackend, text: &str) -> Result<(), String> {
    for ch in text.chars() {
        let (key, needs_shift) = char_to_evdev(ch).ok_or_else(|| {
            format!("The character `{ch}` is not supported by Linux macro playback.")
        })?;
        if needs_shift {
            LinuxPlaybackBackend::emit_key(&mut backend.keyboard, Key::KEY_LEFTSHIFT, true)?;
        }
        backend.press_keyboard_key(key)?;
        if needs_shift {
            LinuxPlaybackBackend::emit_key(&mut backend.keyboard, Key::KEY_LEFTSHIFT, false)?;
        }
        sleep(Duration::from_millis(3)).await;
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
async fn linear_move_to(
    enigo: &mut Enigo,
    target_x: i32,
    target_y: i32,
    duration_ms: u32,
) -> Result<(), String> {
    let (start_x, start_y) = enigo
        .location()
        .map_err(|e| format!("Could not read the cursor position. Details: {}", e))?;

    let duration_ms = duration_ms.max(16);
    let step_count = ((duration_ms as f32 / 12.0).ceil() as u32).clamp(2, 120);

    for step in 1..=step_count {
        let progress = step as f32 / step_count as f32;
        let next_x = start_x + ((target_x - start_x) as f32 * progress).round() as i32;
        let next_y = start_y + ((target_y - start_y) as f32 * progress).round() as i32;

        enigo
            .move_mouse(next_x, next_y, Coordinate::Abs)
            .map_err(|e| format!("Could not move the mouse. Details: {}", e))?;

        sleep(Duration::from_millis(
            (duration_ms / step_count.max(1)) as u64,
        ))
        .await;
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
async fn play_smooth_path(
    enigo: &mut Enigo,
    path: &[(i32, i32)],
    duration_ms: u32,
) -> Result<(), String> {
    if path.is_empty() {
        return Ok(());
    }
    if path.len() == 1 {
        let (x, y) = path[0];
        enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| format!("Could not move mouse to ({}, {}): {}", x, y, e))?;
        return Ok(());
    }

    let mut segment_lengths = Vec::with_capacity(path.len() - 1);
    let mut total_length = 0.0f32;
    for i in 1..path.len() {
        let dx = (path[i].0 - path[i - 1].0) as f32;
        let dy = (path[i].1 - path[i - 1].1) as f32;
        let len = (dx * dx + dy * dy).sqrt();
        segment_lengths.push(len);
        total_length += len;
    }

    if total_length <= 0.0 {
        let (x, y) = path[0];
        enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| format!("Could not move mouse to ({}, {}): {}", x, y, e))?;
        return Ok(());
    }

    let duration_ms = duration_ms.max(1);

    for i in 1..path.len() {
        let (start_x, start_y) = path[i - 1];
        let (target_x, target_y) = path[i];
        let segment_duration =
            (duration_ms as f32 * segment_lengths[i - 1] / total_length).max(8.0) as u32;
        let step_count = ((segment_duration as f32 / 8.0).ceil() as u32).clamp(2, 60);

        for step in 1..=step_count {
            let progress = step as f32 / step_count as f32;
            let x = start_x + ((target_x - start_x) as f32 * progress).round() as i32;
            let y = start_y + ((target_y - start_y) as f32 * progress).round() as i32;
            enigo
                .move_mouse(x, y, Coordinate::Abs)
                .map_err(|e| format!("Could not move the mouse. Details: {}", e))?;
            sleep(Duration::from_millis(
                (segment_duration / step_count.max(1)) as u64,
            ))
            .await;
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
async fn play_raw_move(
    enigo: &mut Enigo,
    points: &[(i32, i32, u64)],
    multiplier: f64,
) -> Result<(), String> {
    if points.is_empty() {
        return Ok(());
    }
    let mut prev_ts = points[0].2;
    for &(x, y, ts) in points {
        enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| format!("Could not move the mouse. Details: {}", e))?;
        if ts > prev_ts {
            let delay_ms = ((ts - prev_ts) as f64 / multiplier).round() as u64;
            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
        prev_ts = ts;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
async fn linear_move_to(
    backend: &mut LinuxPlaybackBackend,
    target_x: i32,
    target_y: i32,
    duration_ms: u32,
) -> Result<(), String> {
    let (start_x, start_y) = current_cursor_position().unwrap_or((target_x, target_y));
    let duration_ms = duration_ms.max(16);
    let step_count = ((duration_ms as f32 / 12.0).ceil() as u32).clamp(2, 120);

    for step in 1..=step_count {
        let progress = step as f32 / step_count as f32;
        let next_x = start_x + ((target_x - start_x) as f32 * progress).round() as i32;
        let next_y = start_y + ((target_y - start_y) as f32 * progress).round() as i32;

        backend.move_mouse(next_x, next_y)?;

        sleep(Duration::from_millis(
            (duration_ms / step_count.max(1)) as u64,
        ))
        .await;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
async fn play_smooth_path(
    backend: &mut LinuxPlaybackBackend,
    path: &[(i32, i32)],
    duration_ms: u32,
) -> Result<(), String> {
    if path.is_empty() {
        return Ok(());
    }
    if path.len() == 1 {
        let (x, y) = path[0];
        backend.move_mouse(x, y)?;
        return Ok(());
    }

    let mut segment_lengths = Vec::with_capacity(path.len() - 1);
    let mut total_length = 0.0f32;
    for i in 1..path.len() {
        let dx = (path[i].0 - path[i - 1].0) as f32;
        let dy = (path[i].1 - path[i - 1].1) as f32;
        let len = (dx * dx + dy * dy).sqrt();
        segment_lengths.push(len);
        total_length += len;
    }

    if total_length <= 0.0 {
        let (x, y) = path[0];
        backend.move_mouse(x, y)?;
        return Ok(());
    }

    let duration_ms = duration_ms.max(1);

    for i in 1..path.len() {
        let (start_x, start_y) = path[i - 1];
        let (target_x, target_y) = path[i];
        let segment_duration =
            (duration_ms as f32 * segment_lengths[i - 1] / total_length).max(8.0) as u32;
        let step_count = ((segment_duration as f32 / 8.0).ceil() as u32).clamp(2, 60);

        for step in 1..=step_count {
            let progress = step as f32 / step_count as f32;
            let x = start_x + ((target_x - start_x) as f32 * progress).round() as i32;
            let y = start_y + ((target_y - start_y) as f32 * progress).round() as i32;
            backend.move_mouse(x, y)?;
            sleep(Duration::from_millis(
                (segment_duration / step_count.max(1)) as u64,
            ))
            .await;
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
async fn play_raw_move(
    backend: &mut LinuxPlaybackBackend,
    points: &[(i32, i32, u64)],
    multiplier: f64,
) -> Result<(), String> {
    if points.is_empty() {
        return Ok(());
    }
    let mut prev_ts = points[0].2;
    for &(x, y, ts) in points {
        backend.move_mouse(x, y)?;
        if ts > prev_ts {
            let delay_ms = ((ts - prev_ts) as f64 / multiplier).round() as u64;
            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
        prev_ts = ts;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn current_cursor_position() -> Result<(i32, i32), String> {
    let enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Could not initialize cursor reader. Details: {e}"))?;
    enigo
        .location()
        .map_err(|e| format!("Could not read the cursor position. Details: {e}"))
}

fn stop_policy_deadline(stop_policy: StopPolicy) -> Option<tokio::time::Instant> {
    match stop_policy {
        StopPolicy::DurationMs(ms) => Some(tokio::time::Instant::now() + Duration::from_millis(ms)),
        StopPolicy::UntilStopped | StopPolicy::RepeatCount(_) => None,
    }
}

fn stop_policy_repeat_count(stop_policy: StopPolicy) -> Option<u32> {
    match stop_policy {
        StopPolicy::RepeatCount(count) => Some(count),
        StopPolicy::UntilStopped | StopPolicy::DurationMs(_) => None,
    }
}

async fn wait_or_end(
    duration: Duration,
    cancellation: &Cancellation,
    deadline: Option<tokio::time::Instant>,
) -> Option<RunEnd> {
    match interruptible_wait(duration, cancellation.clone(), deadline).await {
        WaitOutcome::Completed => None,
        WaitOutcome::Cancelled => Some(RunEnd::Cancelled),
        WaitOutcome::DeadlineReached => Some(RunEnd::Deadline),
    }
}

fn input_mouse_button(button: &MacroMouseButton) -> InputMouseButton {
    match button {
        MacroMouseButton::Left => InputMouseButton::Left,
        MacroMouseButton::Middle => InputMouseButton::Middle,
        MacroMouseButton::Right => InputMouseButton::Right,
        MacroMouseButton::Front => InputMouseButton::Front,
        MacroMouseButton::Back => InputMouseButton::Back,
    }
}

fn scaled_duration(duration_ms: u32, multiplier: f64) -> Duration {
    Duration::from_millis((duration_ms as f64 / multiplier).round().max(1.0) as u64)
}

fn recorded_text_key(character: char) -> Option<(&'static str, bool)> {
    let key = match character {
        'a'..='z' | 'A'..='Z' => {
            let lower = character.to_ascii_lowercase();
            return Some((
                match lower {
                    'a' => "a",
                    'b' => "b",
                    'c' => "c",
                    'd' => "d",
                    'e' => "e",
                    'f' => "f",
                    'g' => "g",
                    'h' => "h",
                    'i' => "i",
                    'j' => "j",
                    'k' => "k",
                    'l' => "l",
                    'm' => "m",
                    'n' => "n",
                    'o' => "o",
                    'p' => "p",
                    'q' => "q",
                    'r' => "r",
                    's' => "s",
                    't' => "t",
                    'u' => "u",
                    'v' => "v",
                    'w' => "w",
                    'x' => "x",
                    'y' => "y",
                    'z' => "z",
                    _ => unreachable!(),
                },
                character.is_ascii_uppercase(),
            ));
        }
        '0' | ')' => "0",
        '1' | '!' => "1",
        '2' | '@' => "2",
        '3' | '#' => "3",
        '4' | '$' => "4",
        '5' | '%' => "5",
        '6' | '^' => "6",
        '7' | '&' => "7",
        '8' | '*' => "8",
        '9' | '(' => "9",
        ' ' => "space",
        '\n' => "enter",
        '\t' => "tab",
        '`' | '~' => "grave",
        '-' | '_' => "minus",
        '=' | '+' => "equal",
        '[' | '{' => "leftbrace",
        ']' | '}' => "rightbrace",
        '\\' | '|' => "backslash",
        ';' | ':' => "semicolon",
        '\'' | '"' => "apostrophe",
        ',' | '<' => "comma",
        '.' | '>' => "dot",
        '/' | '?' => "slash",
        _ => return None,
    };
    Some((
        key,
        matches!(
            character,
            ')' | '!'
                | '@'
                | '#'
                | '$'
                | '%'
                | '^'
                | '&'
                | '*'
                | '('
                | '~'
                | '_'
                | '+'
                | '{'
                | '}'
                | '|'
                | ':'
                | '"'
                | '<'
                | '>'
                | '?'
        ),
    ))
}

async fn stepped_move<S: InputSink>(
    session: &mut InputSession<S>,
    start: (i32, i32),
    target: (i32, i32),
    duration: Duration,
    cancellation: &Cancellation,
    deadline: Option<tokio::time::Instant>,
) -> Result<Option<RunEnd>, ExecutionError> {
    let duration_ms = duration.as_millis().max(16) as u64;
    let steps = ((duration_ms as f64 / 12.0).ceil() as u32).clamp(2, 120);
    let per_step = Duration::from_millis((duration_ms / steps as u64).max(1));
    for step in 1..=steps {
        if let Some(end) = cooperative_end_before_injection(cancellation, deadline).await {
            return Ok(Some(end));
        }
        let progress = step as f64 / steps as f64;
        session.move_to(
            start.0 + ((target.0 - start.0) as f64 * progress).round() as i32,
            start.1 + ((target.1 - start.1) as f64 * progress).round() as i32,
        )?;
        if let Some(end) = wait_or_end(per_step, cancellation, deadline).await {
            return Ok(Some(end));
        }
    }
    Ok(None)
}

fn end_before_injection(
    cancellation: &Cancellation,
    deadline: Option<tokio::time::Instant>,
) -> Option<RunEnd> {
    if cancellation.is_cancelled() {
        Some(RunEnd::Cancelled)
    } else if deadline.is_some_and(|deadline| tokio::time::Instant::now() >= deadline) {
        Some(RunEnd::Deadline)
    } else {
        None
    }
}

/// Yielding at every injection boundary prevents a long sequence of zero-delay
/// actions from starving the task that advances a deadline or requests stop.
async fn cooperative_end_before_injection(
    cancellation: &Cancellation,
    deadline: Option<tokio::time::Instant>,
) -> Option<RunEnd> {
    if let Some(end) = end_before_injection(cancellation, deadline) {
        return Some(end);
    }
    tokio::task::yield_now().await;
    end_before_injection(cancellation, deadline)
}

async fn execute_with_sink<S: InputSink>(
    session: &mut InputSession<S>,
    action: &MacroAction,
    multiplier: f64,
    cancellation: &Cancellation,
    deadline: Option<tokio::time::Instant>,
) -> Result<Option<RunEnd>, ExecutionError> {
    if let Some(end) = end_before_injection(cancellation, deadline) {
        return Ok(Some(end));
    }
    match &action.config {
        MacroActionConfig::Mouse {
            button,
            action: mouse_action,
            position,
        } => {
            if let Some((x, y)) = position {
                session.move_to(*x, *y)?;
                if let Some(end) =
                    wait_or_end(scaled_duration(10, multiplier), cancellation, deadline).await
                {
                    return Ok(Some(end));
                }
            }

            let token = InputToken::Mouse(input_mouse_button(button));
            match mouse_action {
                MacroMouseAction::Press => {
                    session.press(token.clone())?;
                    if let Some(end) =
                        wait_or_end(scaled_duration(50, multiplier), cancellation, deadline).await
                    {
                        return Ok(Some(end));
                    }
                    session.release(&token)?;
                }
                MacroMouseAction::Hold { duration_ms } => {
                    session.press(token.clone())?;
                    if let Some(end) = wait_or_end(
                        scaled_duration(*duration_ms, multiplier),
                        cancellation,
                        deadline,
                    )
                    .await
                    {
                        return Ok(Some(end));
                    }
                    session.release(&token)?;
                }
                MacroMouseAction::Down => session.press(token)?,
                MacroMouseAction::Up => session.release(&token)?,
            }
        }
        MacroActionConfig::Move { x, y, style } => match style {
            MacroMoveStyle::Instant => session.move_to(*x, *y)?,
            MacroMoveStyle::Linear { duration_ms } => {
                let start = session.position()?;
                if let Some(end) = stepped_move(
                    session,
                    start,
                    (*x, *y),
                    scaled_duration(*duration_ms, multiplier),
                    cancellation,
                    deadline,
                )
                .await?
                {
                    return Ok(Some(end));
                }
            }
            MacroMoveStyle::Smooth { path, duration_ms } => {
                if path.is_empty() {
                    session.move_to(*x, *y)?;
                } else {
                    let mut total_length = 0.0;
                    for segment in path.windows(2) {
                        let dx = (segment[1].0 - segment[0].0) as f64;
                        let dy = (segment[1].1 - segment[0].1) as f64;
                        total_length += (dx * dx + dy * dy).sqrt();
                    }
                    if total_length == 0.0 {
                        session.move_to(path[0].0, path[0].1)?;
                    } else {
                        for segment in path.windows(2) {
                            let dx = (segment[1].0 - segment[0].0) as f64;
                            let dy = (segment[1].1 - segment[0].1) as f64;
                            let length = (dx * dx + dy * dy).sqrt();
                            let segment_duration = scaled_duration(
                                ((*duration_ms as f64 * length / total_length).max(8.0)) as u32,
                                multiplier,
                            );
                            if let Some(end) = stepped_move(
                                session,
                                segment[0],
                                segment[1],
                                segment_duration,
                                cancellation,
                                deadline,
                            )
                            .await?
                            {
                                return Ok(Some(end));
                            }
                        }
                    }
                }
            }
        },
        // RawMove is a legacy format: it must remain executable after loading
        // old macro files even though command APIs no longer accept new ones.
        MacroActionConfig::RawMove { points } => {
            let mut previous_timestamp = points.first().map(|point| point.2).unwrap_or(0);
            for (x, y, timestamp) in points {
                if let Some(end) = cooperative_end_before_injection(cancellation, deadline).await {
                    return Ok(Some(end));
                }
                session.move_to(*x, *y)?;
                let delay_ms = timestamp.saturating_sub(previous_timestamp);
                if delay_ms > 0 {
                    let delay =
                        Duration::from_millis((delay_ms as f64 / multiplier).round() as u64);
                    if let Some(end) = wait_or_end(delay, cancellation, deadline).await {
                        return Ok(Some(end));
                    }
                }
                previous_timestamp = *timestamp;
            }
        }
        MacroActionConfig::Keyboard {
            key,
            text,
            modifiers,
            action: keyboard_action,
        } => {
            if let Some(recorded_text) = text.as_deref() {
                if modifiers.is_empty() && matches!(keyboard_action, MacroKeyboardAction::Press) {
                    for character in recorded_text.chars() {
                        if let Some(end) =
                            cooperative_end_before_injection(cancellation, deadline).await
                        {
                            return Ok(Some(end));
                        }
                        #[cfg(not(target_os = "linux"))]
                        if !character.is_ascii() {
                            // Preserve Enigo's legacy Unicode-key behavior on
                            // platforms where it is available. Linux stays on
                            // the physical evdev path below.
                            let token = InputToken::Key(character.to_string());
                            session.press(token.clone())?;
                            session.release(&token)?;
                            continue;
                        }
                        let (key, needs_shift) = recorded_text_key(character).ok_or_else(|| {
                            ExecutionError::UnsupportedKey(format!(
                                "The character `{character}` is not supported by macro playback."
                            ))
                        })?;
                        let shift = InputToken::Key("shift".to_string());
                        if needs_shift {
                            session.press(shift.clone())?;
                        }
                        let token = InputToken::Key(key.to_string());
                        session.press(token.clone())?;
                        session.release(&token)?;
                        if needs_shift {
                            session.release(&shift)?;
                        }
                    }
                    return Ok(None);
                }
            }

            let modifier_tokens: Vec<InputToken> =
                modifiers.iter().cloned().map(InputToken::Key).collect();
            for token in &modifier_tokens {
                session.press(token.clone())?;
            }

            let key_token = InputToken::Key(key.clone());
            match keyboard_action {
                MacroKeyboardAction::Press => {
                    session.press(key_token.clone())?;
                    session.release(&key_token)?;
                }
                MacroKeyboardAction::Hold { duration_ms } => {
                    session.press(key_token.clone())?;
                    if let Some(end) = wait_or_end(
                        scaled_duration(*duration_ms, multiplier),
                        cancellation,
                        deadline,
                    )
                    .await
                    {
                        return Ok(Some(end));
                    }
                    session.release(&key_token)?;
                }
                MacroKeyboardAction::Down => session.press(key_token)?,
                MacroKeyboardAction::Up => session.release(&key_token)?,
            }

            for token in modifier_tokens.iter().rev() {
                session.release(token)?;
            }
        }
        MacroActionConfig::Sleep { duration_ms } => {
            if let Some(end) = wait_or_end(
                scaled_duration(*duration_ms, multiplier),
                cancellation,
                deadline,
            )
            .await
            {
                return Ok(Some(end));
            }
        }
        MacroActionConfig::Scroll { clicks } => session.scroll(*clicks)?,
    }

    Ok(None)
}

/// Executes a macro through an injected input sink. Inputs are owned by the
/// session for the whole macro run, so cancellation, deadlines, and send
/// failures always release every key/button this run pressed in reverse order.
pub async fn run_macro<S: InputSink>(
    actions: &[MacroAction],
    stop_policy: StopPolicy,
    cancellation: Cancellation,
    sink: S,
    multiplier: f64,
) -> Result<RunEnd, ExecutionError> {
    let deadline = stop_policy_deadline(stop_policy);
    run_macro_with_deadline(
        actions,
        stop_policy,
        cancellation,
        deadline,
        sink,
        multiplier,
    )
    .await
}

async fn run_macro_with_deadline<S: InputSink>(
    actions: &[MacroAction],
    stop_policy: StopPolicy,
    cancellation: Cancellation,
    deadline: Option<tokio::time::Instant>,
    sink: S,
    multiplier: f64,
) -> Result<RunEnd, ExecutionError> {
    let repeat_count = stop_policy_repeat_count(stop_policy);
    let mut session = InputSession::new(sink);
    let mut completed_iterations = 0;

    loop {
        if let Some(count) = repeat_count {
            if completed_iterations >= count {
                return Ok(RunEnd::Completed);
            }
        }

        for (index, action) in actions.iter().enumerate() {
            if let Some(end) = cooperative_end_before_injection(&cancellation, deadline).await {
                return Ok(end);
            }
            if let Some(end) =
                execute_with_sink(&mut session, action, multiplier, &cancellation, deadline).await?
            {
                return Ok(end);
            }

            if let Some(next_action) = actions.get(index + 1) {
                let wait_ms = if action.timestamp_ms > 0
                    && next_action.timestamp_ms > 0
                    && next_action.timestamp_ms >= action.timestamp_ms
                {
                    next_action.timestamp_ms - action.timestamp_ms
                } else {
                    5
                };
                let delay =
                    Duration::from_millis((wait_ms as f64 / multiplier).round().max(1.0) as u64);
                if let Some(end) = wait_or_end(delay, &cancellation, deadline).await {
                    return Ok(end);
                }
            }
        }

        // A repeat becomes observable only after the final action in the list
        // has completed; cancelled/deadline-cut iterations never increment it.
        completed_iterations += 1;

        // Instant-only macros otherwise never await, which would prevent the
        // runtime from observing a deadline or cancellation between repeats.
        tokio::task::yield_now().await;
    }
}

#[cfg(not(target_os = "linux"))]
struct EnigoMacroSink {
    enigo: Enigo,
}

#[cfg(not(target_os = "linux"))]
impl EnigoMacroSink {
    fn new() -> Result<Self, String> {
        Enigo::new(&Settings::default())
            .map(|enigo| Self { enigo })
            .map_err(|error| error.to_string())
    }
}

#[cfg(not(target_os = "linux"))]
impl InputSink for EnigoMacroSink {
    fn key_down(&mut self, key: &str) -> Result<(), InputError> {
        self.enigo
            .key(str_to_key(key), enigo::Direction::Press)
            .map_err(|error| InputError::new(error.to_string()))
    }

    fn key_up(&mut self, key: &str) -> Result<(), InputError> {
        self.enigo
            .key(str_to_key(key), enigo::Direction::Release)
            .map_err(|error| InputError::new(error.to_string()))
    }

    fn mouse_down(&mut self, button: InputMouseButton) -> Result<(), InputError> {
        self.enigo
            .button(input_button_to_enigo(button)?, enigo::Direction::Press)
            .map_err(|error| InputError::new(error.to_string()))
    }

    fn mouse_up(&mut self, button: InputMouseButton) -> Result<(), InputError> {
        self.enigo
            .button(input_button_to_enigo(button)?, enigo::Direction::Release)
            .map_err(|error| InputError::new(error.to_string()))
    }

    fn position(&mut self) -> Result<(i32, i32), InputError> {
        self.enigo
            .location()
            .map_err(|error| InputError::new(error.to_string()))
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), InputError> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|error| InputError::new(error.to_string()))
    }

    fn scroll(&mut self, clicks: i32) -> Result<(), InputError> {
        self.enigo
            .scroll(-clicks, Axis::Vertical)
            .map_err(|error| InputError::new(error.to_string()))
    }
}

#[cfg(not(target_os = "linux"))]
fn input_button_to_enigo(button: InputMouseButton) -> Result<Button, InputError> {
    let button = match button {
        InputMouseButton::Left => MacroMouseButton::Left,
        InputMouseButton::Middle => MacroMouseButton::Middle,
        InputMouseButton::Right => MacroMouseButton::Right,
        InputMouseButton::Front => MacroMouseButton::Front,
        InputMouseButton::Back => MacroMouseButton::Back,
    };
    macro_button_to_enigo(&button).map_err(InputError::new)
}

#[cfg(target_os = "linux")]
struct LinuxMacroSink {
    backend: LinuxPlaybackBackend,
}

#[cfg(target_os = "linux")]
impl LinuxMacroSink {
    fn new() -> Result<Self, String> {
        LinuxPlaybackBackend::new().map(|backend| Self { backend })
    }
}

#[cfg(target_os = "linux")]
impl InputSink for LinuxMacroSink {
    fn key_down(&mut self, key: &str) -> Result<(), InputError> {
        let key = str_to_evdev_key(key)
            .ok_or_else(|| InputError::new(format!("Unsupported Linux macro key `{key}`")))?;
        LinuxPlaybackBackend::emit_key(&mut self.backend.keyboard, key, true)
            .map_err(InputError::new)
    }

    fn key_up(&mut self, key: &str) -> Result<(), InputError> {
        let key = str_to_evdev_key(key)
            .ok_or_else(|| InputError::new(format!("Unsupported Linux macro key `{key}`")))?;
        LinuxPlaybackBackend::emit_key(&mut self.backend.keyboard, key, false)
            .map_err(InputError::new)
    }

    fn mouse_down(&mut self, button: InputMouseButton) -> Result<(), InputError> {
        LinuxPlaybackBackend::emit_key(&mut self.backend.mouse, input_button_to_evdev(button), true)
            .map_err(InputError::new)
    }

    fn mouse_up(&mut self, button: InputMouseButton) -> Result<(), InputError> {
        LinuxPlaybackBackend::emit_key(
            &mut self.backend.mouse,
            input_button_to_evdev(button),
            false,
        )
        .map_err(InputError::new)
    }

    fn position(&mut self) -> Result<(i32, i32), InputError> {
        current_cursor_position().map_err(InputError::new)
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), InputError> {
        self.backend.move_mouse(x, y).map_err(InputError::new)
    }

    fn scroll(&mut self, clicks: i32) -> Result<(), InputError> {
        self.backend.scroll_mouse(clicks).map_err(InputError::new)
    }
}

#[cfg(target_os = "linux")]
fn input_button_to_evdev(button: InputMouseButton) -> Key {
    match button {
        InputMouseButton::Left => Key::BTN_LEFT,
        InputMouseButton::Middle => Key::BTN_MIDDLE,
        InputMouseButton::Right => Key::BTN_RIGHT,
        InputMouseButton::Front => Key::BTN_SIDE,
        InputMouseButton::Back => Key::BTN_EXTRA,
    }
}

async fn begin_playback_runtime(
    state: &AppState,
    stop_policy: StopPolicy,
) -> Result<MacroRunControl, String> {
    state
        .runtime_coordinator
        .start_macro(stop_policy)
        .await
        .map_err(|error| format!("Could not start macro playback: {error:?}"))
}

async fn finish_successful_playback_runtime(state: &AppState) {
    let _ = state.runtime_coordinator.finish_macro().await;
}

pub async fn start_playback(
    state: std::sync::Arc<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let macro_state = &state.macro_engine;
    let actions = macro_state.actions.lock().await;

    if actions.is_empty() {
        let _ = app_handle.emit(
            "macro-status-changed",
            serde_json::json!({
                "state": "stopped",
                "error": "Add at least one macro action before starting playback."
            }),
        );
        return Err("Add at least one macro action before starting playback.".to_string());
    }

    let mut player_state_guard = macro_state.player_state.lock().await;
    if *player_state_guard == MacroPlayerState::Playing {
        return Ok(());
    }
    let actions_clone = actions.clone();
    let repeat_mode = macro_state.repeat_mode.lock().await.clone();
    let stop_policy = repeat_mode.stop_policy();

    drop(actions);

    let control = begin_playback_runtime(&state, stop_policy).await?;

    *player_state_guard = MacroPlayerState::Playing;
    drop(player_state_guard);

    let _ = app_handle.emit(
        "macro-status-changed",
        serde_json::json!({
            "state": "playing"
        }),
    );

    tokio::spawn({
        let state = state.clone();
        let app_handle = app_handle.clone();
        async move {
            playback_loop(&actions_clone, stop_policy, control, &state, app_handle).await;
        }
    });

    Ok(())
}

async fn playback_loop(
    actions: &[MacroAction],
    stop_policy: StopPolicy,
    control: MacroRunControl,
    state: &AppState,
    app_handle: tauri::AppHandle,
) {
    #[cfg(not(target_os = "linux"))]
    let sink = match EnigoMacroSink::new() {
        Ok(sink) => sink,
        Err(error) => {
            finish_playback(
                state,
                Err(ExecutionError::PermissionMismatch(format!(
                    "The app could not control your mouse or keyboard. Check system permissions and try again. Details: {error}"
                ))),
                app_handle,
            )
            .await;
            return;
        }
    };

    #[cfg(target_os = "linux")]
    let sink = match LinuxMacroSink::new() {
        Ok(sink) => sink,
        Err(error) => {
            finish_playback(
                state,
                Err(ExecutionError::PermissionMismatch(error)),
                app_handle,
            )
            .await;
            return;
        }
    };
    let multiplier = *state.macro_engine.speed_multiplier.lock().await;
    let result = run_macro_with_deadline(
        actions,
        stop_policy,
        control.cancellation(),
        control.deadline(),
        sink,
        multiplier,
    )
    .await;
    finish_playback(state, result, app_handle).await;
}

async fn finish_playback(
    state: &AppState,
    result: Result<RunEnd, ExecutionError>,
    app_handle: tauri::AppHandle,
) {
    match result {
        Ok(_) => {
            finish_successful_playback_runtime(state).await;
        }
        Err(error) => {
            let _ = state
                .runtime_coordinator
                .fail_macro(format!("{error:?}"))
                .await;
            let _ = app_handle.emit(
                "macro-status-changed",
                serde_json::json!({
                    "state": "error",
                    "error": format!("Macro playback stopped because an action could not be completed. {error:?}")
                }),
            );
        }
    }

    *state.macro_engine.player_state.lock().await = MacroPlayerState::Stopped;
    let _ = app_handle.emit(
        "macro-step-changed",
        serde_json::json!({
            "action_id": serde_json::Value::Null
        }),
    );
    let _ = app_handle.emit(
        "macro-status-changed",
        serde_json::json!({
            "state": "stopped"
        }),
    );
}

pub async fn stop_playback(state: &AppState, _app_handle: tauri::AppHandle) {
    let _ = state.runtime_coordinator.cancel_macro().await;
}

fn estimate_actions_duration(actions: &[MacroAction]) -> u64 {
    let mut total = 0u64;

    for action in actions {
        match &action.config {
            MacroActionConfig::Mouse {
                action: MacroMouseAction::Hold { duration_ms },
                ..
            } => {
                total += *duration_ms as u64 + 50;
            }
            MacroActionConfig::Mouse { .. } => {
                total += 60;
            }
            MacroActionConfig::Move { style, .. } => match style {
                MacroMoveStyle::Instant => {
                    total += 10;
                }
                MacroMoveStyle::Linear { duration_ms } => {
                    total += *duration_ms as u64;
                }
                MacroMoveStyle::Smooth { duration_ms, .. } => {
                    total += *duration_ms as u64;
                }
            },
            MacroActionConfig::Keyboard {
                action: MacroKeyboardAction::Hold { duration_ms },
                ..
            } => {
                total += *duration_ms as u64 + 50;
            }
            MacroActionConfig::Keyboard { .. } => {
                total += 60;
            }
            MacroActionConfig::Sleep { duration_ms } => {
                total += *duration_ms as u64;
            }
            MacroActionConfig::Scroll { .. } => {
                total += 50;
            }
            MacroActionConfig::RawMove { points } => {
                if points.len() >= 2 {
                    total += points.last().map(|p| p.2).unwrap_or(0)
                        - points.first().map(|p| p.2).unwrap_or(0);
                }
                total += 10;
            }
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::clicker::{ExecutionError, RunEnd};
    use crate::engine::executor::Cancellation;
    use crate::engine::input::test_support::{InputCall, SharedFakeSink};
    use crate::engine::input::InputToken;
    use crate::engine::runtime::StopPolicy;

    fn action(id: u64, config: MacroActionConfig) -> MacroAction {
        MacroAction {
            id,
            timestamp_ms: 0,
            config,
        }
    }

    fn key_down(key: &str) -> MacroAction {
        action(
            1,
            MacroActionConfig::Keyboard {
                key: key.to_string(),
                text: None,
                modifiers: Vec::new(),
                action: MacroKeyboardAction::Down,
            },
        )
    }

    fn sleep_ms(duration_ms: u32) -> MacroAction {
        action(2, MacroActionConfig::Sleep { duration_ms })
    }

    #[tokio::test(start_paused = true)]
    async fn finite_repeats_count_only_completed_action_lists() {
        let actions = vec![action(
            1,
            MacroActionConfig::Keyboard {
                key: "A".to_string(),
                text: None,
                modifiers: Vec::new(),
                action: MacroKeyboardAction::Press,
            },
        )];
        let sink = SharedFakeSink::default();

        let end = run_macro(
            &actions,
            StopPolicy::RepeatCount(2),
            Cancellation::new(),
            sink.clone(),
            1.0,
        )
        .await
        .unwrap();

        assert_eq!(end, RunEnd::Completed);
        assert_eq!(sink.complete_cycles(&InputToken::Key("A".to_string())), 2);
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn deadline_stops_instant_actions_before_another_iteration_injects_input() {
        let actions = vec![action(
            1,
            MacroActionConfig::Keyboard {
                key: "A".to_string(),
                text: None,
                modifiers: Vec::new(),
                action: MacroKeyboardAction::Press,
            },
        )];
        let sink = SharedFakeSink::default();
        let run = tokio::spawn({
            let actions = actions.clone();
            let sink = sink.clone();
            async move {
                run_macro(
                    &actions,
                    StopPolicy::DurationMs(5),
                    Cancellation::new(),
                    sink,
                    1.0,
                )
                .await
            }
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(5)).await;

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Deadline);
        assert_eq!(sink.complete_cycles(&InputToken::Key("A".to_string())), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn deadline_prevents_a_long_instant_action_list_from_injecting_after_expiry() {
        let actions = (0..128)
            .map(|id| key_down(&format!("Key{id}")))
            .collect::<Vec<_>>();
        let sink = SharedFakeSink::default();
        let run = tokio::spawn({
            let sink = sink.clone();
            async move {
                run_macro(
                    &actions,
                    StopPolicy::DurationMs(5),
                    Cancellation::new(),
                    sink,
                    1.0,
                )
                .await
            }
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(5)).await;

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Deadline);
        assert!(sink.calls().is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn deadline_prevents_remaining_recorded_text_characters_from_injecting() {
        let actions = vec![action(
            1,
            MacroActionConfig::Keyboard {
                key: "ignored".to_string(),
                text: Some("A".repeat(128)),
                modifiers: Vec::new(),
                action: MacroKeyboardAction::Press,
            },
        )];
        let sink = SharedFakeSink::default();
        let run = tokio::spawn({
            let sink = sink.clone();
            async move {
                run_macro(
                    &actions,
                    StopPolicy::DurationMs(5),
                    Cancellation::new(),
                    sink,
                    1.0,
                )
                .await
            }
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(5)).await;

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Deadline);
        assert!(sink.calls().is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn deadline_prevents_same_timestamp_raw_move_points_from_injecting() {
        let actions = vec![action(
            1,
            MacroActionConfig::RawMove {
                points: (0..128).map(|x| (x, 0, 0)).collect(),
            },
        )];
        let sink = SharedFakeSink::default();
        let run = tokio::spawn({
            let sink = sink.clone();
            async move {
                run_macro(
                    &actions,
                    StopPolicy::DurationMs(5),
                    Cancellation::new(),
                    sink,
                    1.0,
                )
                .await
            }
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(5)).await;

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Deadline);
        assert!(sink.calls().is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn recorded_text_keeps_shifted_characters_and_spaces_as_physical_input() {
        let actions = vec![action(
            1,
            MacroActionConfig::Keyboard {
                key: "ignored".to_string(),
                text: Some("A !".to_string()),
                modifiers: Vec::new(),
                action: MacroKeyboardAction::Press,
            },
        )];
        let sink = SharedFakeSink::default();

        assert_eq!(
            run_macro(
                &actions,
                StopPolicy::RepeatCount(1),
                Cancellation::new(),
                sink.clone(),
                1.0,
            )
            .await
            .unwrap(),
            RunEnd::Completed
        );
        assert_eq!(
            sink.calls(),
            vec![
                InputCall::Down(InputToken::Key("shift".to_string())),
                InputCall::Down(InputToken::Key("a".to_string())),
                InputCall::Up(InputToken::Key("a".to_string())),
                InputCall::Up(InputToken::Key("shift".to_string())),
                InputCall::Down(InputToken::Key("space".to_string())),
                InputCall::Up(InputToken::Key("space".to_string())),
                InputCall::Down(InputToken::Key("shift".to_string())),
                InputCall::Down(InputToken::Key("1".to_string())),
                InputCall::Up(InputToken::Key("1".to_string())),
                InputCall::Up(InputToken::Key("shift".to_string())),
            ]
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_sink_accepts_canonical_text_and_shortcut_modifier_tokens() {
        assert_eq!(super::str_to_evdev_key("shift"), Some(Key::KEY_LEFTSHIFT));
        assert_eq!(super::str_to_evdev_key("ctrl"), Some(Key::KEY_LEFTCTRL));
        assert_eq!(super::str_to_evdev_key("alt"), Some(Key::KEY_LEFTALT));
        assert_eq!(super::str_to_evdev_key("super"), Some(Key::KEY_LEFTMETA));
        assert_eq!(super::str_to_evdev_key("enter"), Some(Key::KEY_ENTER));
        assert_eq!(super::str_to_evdev_key("tab"), Some(Key::KEY_TAB));
    }

    #[tokio::test(start_paused = true)]
    async fn smooth_moves_interpolate_between_recorded_path_points() {
        let actions = vec![action(
            1,
            MacroActionConfig::Move {
                x: 10,
                y: 0,
                style: MacroMoveStyle::Smooth {
                    path: vec![(0, 0), (10, 0)],
                    duration_ms: 16,
                },
            },
        )];
        let sink = SharedFakeSink::default();

        assert_eq!(
            run_macro(
                &actions,
                StopPolicy::RepeatCount(1),
                Cancellation::new(),
                sink.clone(),
                1.0,
            )
            .await
            .unwrap(),
            RunEnd::Completed
        );
        assert_eq!(
            sink.calls(),
            vec![InputCall::MoveTo(5, 0), InputCall::MoveTo(10, 0)]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_during_linear_move_stops_before_the_final_step() {
        let actions = vec![action(
            1,
            MacroActionConfig::Move {
                x: 100,
                y: 0,
                style: MacroMoveStyle::Linear { duration_ms: 120 },
            },
        )];
        let sink = SharedFakeSink::default();
        let cancellation = Cancellation::new();
        let run = tokio::spawn({
            let actions = actions.clone();
            let sink = sink.clone();
            let cancellation = cancellation.clone();
            async move { run_macro(&actions, StopPolicy::UntilStopped, cancellation, sink, 1.0).await }
        });

        tokio::task::yield_now().await;
        tokio::task::yield_now().await;
        tokio::task::yield_now().await;
        cancellation.cancel();

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Cancelled);
        assert_eq!(sink.calls(), vec![InputCall::MoveTo(10, 0)]);
    }

    #[tokio::test(start_paused = true)]
    async fn deadline_cuts_off_macro_mid_iteration_and_releases_key() {
        let actions = vec![
            key_down("ShiftLeft"),
            sleep_ms(60_000),
            action(
                3,
                MacroActionConfig::Keyboard {
                    key: "ShiftLeft".to_string(),
                    text: None,
                    modifiers: Vec::new(),
                    action: MacroKeyboardAction::Up,
                },
            ),
        ];
        let sink = SharedFakeSink::default();
        let run = tokio::spawn({
            let actions = actions.clone();
            let sink = sink.clone();
            async move {
                run_macro(
                    &actions,
                    StopPolicy::DurationMs(500),
                    Cancellation::new(),
                    sink,
                    1.0,
                )
                .await
            }
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(500)).await;

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Deadline);
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_during_sleep_returns_without_waiting_for_the_full_delay() {
        let sink = SharedFakeSink::default();
        let cancel = Cancellation::new();
        let actions = vec![sleep_ms(60_000)];
        let run = tokio::spawn({
            let cancel = cancel.clone();
            let sink = sink.clone();
            async move { run_macro(&actions, StopPolicy::UntilStopped, cancel, sink, 1.0).await }
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(5)).await;
        tokio::task::yield_now().await;
        cancel.cancel();

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Cancelled);
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_during_mouse_hold_releases_the_button() {
        let actions = vec![action(
            1,
            MacroActionConfig::Mouse {
                button: super::super::types::MacroMouseButton::Left,
                action: MacroMouseAction::Hold {
                    duration_ms: 60_000,
                },
                position: None,
            },
        )];
        let sink = SharedFakeSink::default();
        let cancel = Cancellation::new();
        let run = tokio::spawn({
            let actions = actions.clone();
            let cancel = cancel.clone();
            let sink = sink.clone();
            async move { run_macro(&actions, StopPolicy::UntilStopped, cancel, sink, 1.0).await }
        });

        tokio::task::yield_now().await;
        cancel.cancel();

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Cancelled);
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_during_keyboard_hold_releases_the_key() {
        let actions = vec![action(
            1,
            MacroActionConfig::Keyboard {
                key: "ShiftLeft".to_string(),
                text: None,
                modifiers: Vec::new(),
                action: MacroKeyboardAction::Hold {
                    duration_ms: 60_000,
                },
            },
        )];
        let sink = SharedFakeSink::default();
        let cancel = Cancellation::new();
        let run = tokio::spawn({
            let actions = actions.clone();
            let cancel = cancel.clone();
            let sink = sink.clone();
            async move { run_macro(&actions, StopPolicy::UntilStopped, cancel, sink, 1.0).await }
        });

        tokio::task::yield_now().await;
        cancel.cancel();

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Cancelled);
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn send_failure_releases_inputs_already_pressed_by_the_macro() {
        let sink = SharedFakeSink::default();
        sink.fail_on_attempt(1);
        let result = run_macro(
            &[key_down("ControlLeft"), key_down("ShiftLeft")],
            StopPolicy::UntilStopped,
            Cancellation::new(),
            sink.clone(),
            1.0,
        )
        .await;

        assert_eq!(
            result,
            Err(ExecutionError::SendFailed("forced failure".to_string()))
        );
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn legacy_raw_move_actions_still_play_back() {
        let actions = vec![action(
            1,
            MacroActionConfig::RawMove {
                points: vec![(10, 20, 0), (30, 40, 10)],
            },
        )];
        let sink = SharedFakeSink::default();
        let run = tokio::spawn({
            let actions = actions.clone();
            let sink = sink.clone();
            async move {
                run_macro(
                    &actions,
                    StopPolicy::RepeatCount(1),
                    Cancellation::new(),
                    sink,
                    1.0,
                )
                .await
            }
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(10)).await;

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Completed);
        assert_eq!(
            sink.calls(),
            vec![InputCall::MoveTo(10, 20), InputCall::MoveTo(30, 40)]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_releases_held_inputs_in_reverse_press_order() {
        let sink = SharedFakeSink::default();
        let cancel = Cancellation::new();
        let actions = vec![
            key_down("ControlLeft"),
            key_down("ShiftLeft"),
            sleep_ms(60_000),
        ];
        let run = tokio::spawn({
            let cancel = cancel.clone();
            let sink = sink.clone();
            async move { run_macro(&actions, StopPolicy::UntilStopped, cancel, sink, 1.0).await }
        });

        for _ in 0..3 {
            tokio::task::yield_now().await;
        }
        tokio::time::advance(Duration::from_millis(5)).await;
        for _ in 0..3 {
            tokio::task::yield_now().await;
        }
        assert_eq!(sink.calls().len(), 2);
        cancel.cancel();

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Cancelled);
        assert_eq!(
            sink.calls(),
            vec![
                InputCall::Down(InputToken::Key("ControlLeft".to_string())),
                InputCall::Down(InputToken::Key("ShiftLeft".to_string())),
                InputCall::Up(InputToken::Key("ShiftLeft".to_string())),
                InputCall::Up(InputToken::Key("ControlLeft".to_string())),
            ]
        );
    }

    #[tokio::test]
    async fn macro_start_uses_the_app_coordinator_and_respects_other_running_modes() {
        let state = AppState::default();
        state
            .runtime_coordinator
            .start(
                crate::engine::runtime::AppMode::Mouse,
                StopPolicy::UntilStopped,
            )
            .await
            .unwrap();

        assert!(begin_playback_runtime(&state, StopPolicy::UntilStopped)
            .await
            .unwrap_err()
            .contains("Busy(RunningMouse)"));
    }

    #[tokio::test]
    async fn successful_macro_finish_returns_the_app_coordinator_to_idle_once() {
        let state = AppState::default();
        begin_playback_runtime(&state, StopPolicy::RepeatCount(1))
            .await
            .unwrap();

        finish_successful_playback_runtime(&state).await;

        assert_eq!(
            state.runtime_coordinator.snapshot().await.phase,
            crate::engine::runtime::RuntimePhase::Idle
        );
        assert_eq!(
            state.runtime_coordinator.finish_stop().await,
            Err(crate::engine::runtime::RuntimeError::InvalidTransition)
        );
    }
}
