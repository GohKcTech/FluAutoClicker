use enigo::{Button, Coordinate, Enigo, Key, Keyboard, Mouse, Settings};
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::time::sleep;

use super::state::MacroEngineState;
use super::types::{
    MacroAction, MacroActionConfig, MacroKeyboardAction, MacroMouseAction, MacroMoveStyle,
    MacroPlayerState,
};

async fn execute_action(enigo: &mut Enigo, action: &MacroAction) -> Result<(), String> {
    match &action.config {
        MacroActionConfig::Mouse {
            button,
            action: mouse_action,
            position,
        } => {
            if let Some((x, y)) = position {
                enigo
                    .move_mouse(*x, *y, Coordinate::Abs)
                    .map_err(|e| format!("Failed to move mouse: {}", e))?;

                sleep(Duration::from_millis(10)).await;
            }

            match mouse_action {
                MacroMouseAction::Press => {
                    let btn = macro_button_to_enigo(button)?;
                    enigo
                        .button(btn, enigo::Direction::Press)
                        .map_err(|e| format!("Failed to press mouse button: {}", e))?;
                    sleep(Duration::from_millis(50)).await;
                    enigo
                        .button(btn, enigo::Direction::Release)
                        .map_err(|e| format!("Failed to release mouse button: {}", e))?;
                }
                MacroMouseAction::Hold { duration_ms } => {
                    let btn = macro_button_to_enigo(button)?;
                    enigo
                        .button(btn, enigo::Direction::Press)
                        .map_err(|e| format!("Failed to press mouse button: {}", e))?;
                    sleep(Duration::from_millis(*duration_ms as u64)).await;
                    enigo
                        .button(btn, enigo::Direction::Release)
                        .map_err(|e| format!("Failed to release mouse button: {}", e))?;
                }
            }
        }
        MacroActionConfig::Move { x, y, style } => match style {
            MacroMoveStyle::Instant => {
                enigo
                    .move_mouse(*x, *y, Coordinate::Abs)
                    .map_err(|e| format!("Failed to move mouse: {}", e))?;
            }
            MacroMoveStyle::Smooth { duration_ms } => {
                smooth_move_to(enigo, *x, *y, *duration_ms).await?;
            }
        },
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
                        .map_err(|e| format!("Failed to type recorded text: {}", e))?;
                    return Ok(());
                }
            }

            for modifier in modifiers {
                if let Some(key_code) = modifier_to_key(modifier) {
                    enigo
                        .key(key_code, enigo::Direction::Press)
                        .map_err(|e| format!("Failed to press modifier: {}", e))?;
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
                        .map_err(|e| format!("Failed to press key: {}", e))?;
                }
                MacroKeyboardAction::Hold { duration_ms } => {
                    enigo
                        .key(key_code, enigo::Direction::Press)
                        .map_err(|e| format!("Failed to press key: {}", e))?;
                    sleep(Duration::from_millis(*duration_ms as u64)).await;
                    enigo
                        .key(key_code, enigo::Direction::Release)
                        .map_err(|e| format!("Failed to release key: {}", e))?;
                }
            }

            for modifier in modifiers.iter().rev() {
                if let Some(key_code) = modifier_to_key(modifier) {
                    enigo
                        .key(key_code, enigo::Direction::Release)
                        .map_err(|e| format!("Failed to release modifier: {}", e))?;
                }
            }
        }
        MacroActionConfig::Sleep { duration_ms } => {
            sleep(Duration::from_millis(*duration_ms as u64)).await;
        }
    }

    Ok(())
}

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

#[cfg(target_os = "macos")]
fn map_extended_button(_name: &str) -> Result<Button, String> {
    Err("Front and back mouse buttons are not supported on macOS in this build.".to_string())
}

#[cfg(not(target_os = "macos"))]
fn map_extended_button(_name: &str) -> Result<Button, String> {
    Ok(Button::Forward)
}

#[cfg(target_os = "macos")]
fn map_back_button() -> Result<Button, String> {
    Err("Front and back mouse buttons are not supported on macOS in this build.".to_string())
}

#[cfg(not(target_os = "macos"))]
fn map_back_button() -> Result<Button, String> {
    Ok(Button::Back)
}

fn modifier_to_key(modifier: &str) -> Option<Key> {
    match modifier.to_lowercase().as_str() {
        "ctrl" => Some(Key::Control),
        "shift" => Some(Key::Shift),
        "alt" => Some(Key::Alt),
        "win" => Some(Key::Meta),
        _ => None,
    }
}

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
        _ => {}
    }

    if key.len() == 1 {
        let c = key.chars().next().unwrap();
        return Key::Unicode(c);
    }

    Key::Unicode(key.chars().next().unwrap_or('a'))
}

fn str_to_combo_key(key: &str) -> Key {
    str_to_key(key)
}

async fn smooth_move_to(
    enigo: &mut Enigo,
    target_x: i32,
    target_y: i32,
    duration_ms: u32,
) -> Result<(), String> {
    let (start_x, start_y) = enigo
        .location()
        .map_err(|e| format!("Failed to read cursor location: {}", e))?;

    let duration_ms = duration_ms.max(16);
    let step_count = ((duration_ms as f32 / 12.0).ceil() as u32).clamp(2, 120);

    for step in 1..=step_count {
        let progress = step as f32 / step_count as f32;
        let eased = ease_in_out(progress);
        let next_x = start_x + ((target_x - start_x) as f32 * eased).round() as i32;
        let next_y = start_y + ((target_y - start_y) as f32 * eased).round() as i32;

        enigo
            .move_mouse(next_x, next_y, Coordinate::Abs)
            .map_err(|e| format!("Failed to smooth move mouse: {}", e))?;

        sleep(Duration::from_millis(
            (duration_ms / step_count.max(1)) as u64,
        ))
        .await;
    }

    Ok(())
}

fn ease_in_out(progress: f32) -> f32 {
    if progress < 0.5 {
        2.0 * progress * progress
    } else {
        1.0 - ((-2.0 * progress + 2.0).powf(2.0) / 2.0)
    }
}

pub async fn start_playback(
    state: &MacroEngineState,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let actions = state.actions.lock().await;

    if actions.is_empty() {
        let _ = app_handle.emit(
            "macro-status-changed",
            serde_json::json!({
                "state": "stopped",
                "error": "No actions to play"
            }),
        );
        return Err("No actions to play.".to_string());
    }

    *state.player_state.lock().await = MacroPlayerState::Playing;
    state
        .cancel_playback
        .store(false, std::sync::atomic::Ordering::SeqCst);

    let _ = app_handle.emit(
        "macro-status-changed",
        serde_json::json!({
            "state": "playing"
        }),
    );

    let actions_clone = actions.clone();
    let repeat_mode = state.repeat_mode.lock().await.clone();
    let cancel_flag = state.cancel_playback.clone();

    drop(actions);

    tokio::spawn({
        let state = state.clone();
        let app_handle = app_handle.clone();
        let repeat_mode = repeat_mode.clone();
        async move {
            playback_loop(
                &actions_clone,
                &repeat_mode,
                cancel_flag,
                &state,
                app_handle,
            )
            .await;
        }
    });

    Ok(())
}

async fn playback_loop(
    actions: &[MacroAction],
    repeat_mode: &crate::engine::macro_engine::types::MacroRepeatMode,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    state: &MacroEngineState,
    app_handle: tauri::AppHandle,
) {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(error) => {
            let _ = app_handle.emit(
                "macro-status-changed",
                serde_json::json!({
                    "state": "error",
                    "error": format!("Failed to initialize input bridge: {}", error)
                }),
            );
            *state.player_state.lock().await = MacroPlayerState::Stopped;
            return;
        }
    };
    let start_time = Instant::now();

    let max_iterations = match repeat_mode {
        crate::engine::macro_engine::types::MacroRepeatMode::Infinite => None,
        crate::engine::macro_engine::types::MacroRepeatMode::FiniteTimes { count } => {
            Some(*count as u64)
        }
        crate::engine::macro_engine::types::MacroRepeatMode::FiniteSeconds { duration_ms } => {
            let total_action_time = estimate_actions_duration(actions);
            if total_action_time > 0 {
                Some((duration_ms / total_action_time).max(1))
            } else {
                Some(1)
            }
        }
    };

    let mut iteration = 0;

    loop {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        if let Some(max) = max_iterations {
            if iteration >= max {
                break;
            }
        }

        if let crate::engine::macro_engine::types::MacroRepeatMode::FiniteSeconds { duration_ms } =
            repeat_mode
        {
            if start_time.elapsed().as_millis() as u64 >= *duration_ms {
                break;
            }
        }

        for (action_index, action) in actions.iter().enumerate() {
            if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            let _ = app_handle.emit(
                "macro-step-changed",
                serde_json::json!({
                    "action_id": action.id,
                    "action_index": action_index,
                    "total_actions": actions.len(),
                    "iteration": iteration
                }),
            );

            if let Err(e) = execute_action(&mut enigo, action).await {
                eprintln!("Macro playback error: {}", e);
                let _ = app_handle.emit(
                    "macro-status-changed",
                    serde_json::json!({
                        "state": "error",
                        "error": e
                    }),
                );
                cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                break;
            }

            sleep(Duration::from_millis(5)).await;
        }

        iteration += 1;
    }

    *state.player_state.lock().await = MacroPlayerState::Stopped;
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

pub async fn stop_playback(state: &MacroEngineState, app_handle: tauri::AppHandle) {
    state
        .cancel_playback
        .store(true, std::sync::atomic::Ordering::SeqCst);
    *state.player_state.lock().await = MacroPlayerState::Stopped;
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
            MacroActionConfig::Move {
                style: MacroMoveStyle::Smooth { duration_ms },
                ..
            } => {
                total += *duration_ms as u64;
            }
            MacroActionConfig::Move { .. } => {
                total += 10;
            }
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
        }
    }

    total
}
