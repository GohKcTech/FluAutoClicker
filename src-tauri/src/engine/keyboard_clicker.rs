use crate::engine::state::{
    AppState, ClickMode, HoldUnit, KeyboardModifier, RepeatMode, RepeatUnit,
};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "windows")]
const WINDOWS_MAX_TOTAL_CPS: u32 = 650;

#[cfg(not(target_os = "linux"))]
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

#[cfg(target_os = "linux")]
use evdev::{EventType, InputEvent, Key};

#[cfg(target_os = "linux")]
fn string_to_key(key_str: &str) -> Option<Key> {
    match key_str.to_lowercase().as_str() {
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

        "kp0" => Some(Key::KEY_KP0),
        "kp1" => Some(Key::KEY_KP1),
        "kp2" => Some(Key::KEY_KP2),
        "kp3" => Some(Key::KEY_KP3),
        "kp4" => Some(Key::KEY_KP4),
        "kp5" => Some(Key::KEY_KP5),
        "kp6" => Some(Key::KEY_KP6),
        "kp7" => Some(Key::KEY_KP7),
        "kp8" => Some(Key::KEY_KP8),
        "kp9" => Some(Key::KEY_KP9),
        "kpenter" | "numpad_enter" => Some(Key::KEY_KPENTER),
        "kpplus" | "kp+" | "numpad_+" => Some(Key::KEY_KPPLUS),
        "kpminus" | "kp-" | "numpad_-" => Some(Key::KEY_KPMINUS),
        "kpasterisk" | "kp*" | "numpad_*" => Some(Key::KEY_KPASTERISK),
        "kpdot" | "kp." | "numpad_." => Some(Key::KEY_KPDOT),
        "kpslash" | "kp/" | "numpad_/" => Some(Key::KEY_KPSLASH),
        "numlock" | "num_lock" => Some(Key::KEY_NUMLOCK),

        "capslock" | "caps" => Some(Key::KEY_CAPSLOCK),

        "ctrl" | "control" => Some(Key::KEY_LEFTCTRL),
        "shift" => Some(Key::KEY_LEFTSHIFT),
        "alt" => Some(Key::KEY_LEFTALT),
        "win" | "meta" | "super" => Some(Key::KEY_LEFTMETA),
        _ => None,
    }
}

#[cfg(not(target_os = "linux"))]
fn string_to_enigo_key(key_str: &str) -> Key {
    match key_str.to_lowercase().as_str() {
        "a" => Key::A,
        "b" => Key::B,
        "c" => Key::C,
        "d" => Key::D,
        "e" => Key::E,
        "f" => Key::F,
        "g" => Key::G,
        "h" => Key::H,
        "i" => Key::I,
        "j" => Key::J,
        "k" => Key::K,
        "l" => Key::L,
        "m" => Key::M,
        "n" => Key::N,
        "o" => Key::O,
        "p" => Key::P,
        "q" => Key::Q,
        "r" => Key::R,
        "s" => Key::S,
        "t" => Key::T,
        "u" => Key::U,
        "v" => Key::V,
        "w" => Key::W,
        "x" => Key::X,
        "y" => Key::Y,
        "z" => Key::Z,
        "0" => Key::Unicode('0'),
        "1" => Key::Unicode('1'),
        "2" => Key::Unicode('2'),
        "3" => Key::Unicode('3'),
        "4" => Key::Unicode('4'),
        "5" => Key::Unicode('5'),
        "6" => Key::Unicode('6'),
        "7" => Key::Unicode('7'),
        "8" => Key::Unicode('8'),
        "9" => Key::Unicode('9'),
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        "escape" | "esc" => Key::Escape,
        "space" => Key::Space,
        "enter" | "return" => Key::Return,
        "tab" => Key::Tab,
        "backspace" | "back" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "insert" | "ins" => Key::Insert,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" | "page_up" | "pgup" => Key::PageUp,
        "pagedown" | "page_down" | "pgdn" => Key::PageDown,
        "arrowup" | "up" => Key::UpArrow,
        "arrowdown" | "down" => Key::DownArrow,
        "arrowleft" | "left" => Key::LeftArrow,
        "arrowright" | "right" => Key::RightArrow,
        "capslock" | "caps" => Key::CapsLock,
        "ctrl" | "control" => Key::Control,
        "shift" => Key::Shift,
        "alt" => Key::Alt,
        "win" | "meta" | "super" => Key::Meta,
        _ => Key::Unicode(key_str.chars().next().unwrap_or(' ')),
    }
}

#[cfg(not(target_os = "linux"))]
fn modifier_to_enigo_keys(modifier: KeyboardModifier) -> Vec<Key> {
    match modifier {
        KeyboardModifier::None => vec![],
        KeyboardModifier::Ctrl => vec![Key::Control],
        KeyboardModifier::Shift => vec![Key::Shift],
        KeyboardModifier::Alt => vec![Key::Alt],
        KeyboardModifier::Win => vec![Key::Meta],
        KeyboardModifier::CtrlShift => vec![Key::Control, Key::Shift],
        KeyboardModifier::CtrlAlt => vec![Key::Control, Key::Alt],
        KeyboardModifier::CtrlWin => vec![Key::Control, Key::Meta],
        KeyboardModifier::ShiftAlt => vec![Key::Shift, Key::Alt],
        KeyboardModifier::ShiftWin => vec![Key::Shift, Key::Meta],
        KeyboardModifier::AltWin => vec![Key::Alt, Key::Meta],
        KeyboardModifier::CtrlShiftAlt => vec![Key::Control, Key::Shift, Key::Alt],
        KeyboardModifier::CtrlShiftWin => vec![Key::Control, Key::Shift, Key::Meta],
        KeyboardModifier::CtrlAltWin => vec![Key::Control, Key::Alt, Key::Meta],
        KeyboardModifier::ShiftAltWin => vec![Key::Shift, Key::Alt, Key::Meta],
        KeyboardModifier::CtrlShiftAltWin => vec![Key::Control, Key::Shift, Key::Alt, Key::Meta],
    }
}

#[cfg(target_os = "linux")]
async fn perform_keyboard_press(device: &mut evdev::uinput::VirtualDevice, state: &AppState) {
    let key_str = state.keyboard_key.lock().await.clone();
    let modifiers = state.keyboard_modifiers.lock().await.clone();
    let mode = *state.kb_click_mode.lock().await;

    let key = match string_to_key(&key_str) {
        Some(k) => k,
        None => return,
    };

    let modifier_keys = modifiers.to_keys();

    match mode {
        ClickMode::Press => {
            for &mod_key in &modifier_keys {
                let _ = device.emit(&[
                    InputEvent::new(EventType::KEY, mod_key.0, 1),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ]);
            }

            let _ = device.emit(&[
                InputEvent::new(EventType::KEY, key.0, 1),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                InputEvent::new(EventType::KEY, key.0, 0),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);

            for &mod_key in modifier_keys.iter().rev() {
                let _ = device.emit(&[
                    InputEvent::new(EventType::KEY, mod_key.0, 0),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ]);
            }
        }
        ClickMode::Hold => {
            let hold_duration = state.kb_hold_duration.load(Ordering::SeqCst);
            let unit = *state.kb_hold_unit.lock().await;
            let duration_ms = match unit {
                HoldUnit::Milliseconds => hold_duration,
                HoldUnit::Seconds => hold_duration * 1000,
            };

            for &mod_key in &modifier_keys {
                let _ = device.emit(&[
                    InputEvent::new(EventType::KEY, mod_key.0, 1),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ]);
            }

            let _ = device.emit(&[
                InputEvent::new(EventType::KEY, key.0, 1),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);

            tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms as u64)).await;

            let _ = device.emit(&[
                InputEvent::new(EventType::KEY, key.0, 0),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);

            for &mod_key in modifier_keys.iter().rev() {
                let _ = device.emit(&[
                    InputEvent::new(EventType::KEY, mod_key.0, 0),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ]);
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
async fn perform_keyboard_press(enigo: &mut Enigo, state: &AppState) {
    let key_str = state.keyboard_key.lock().await.clone();
    let modifiers = state.keyboard_modifiers.lock().await.clone();
    let mode = *state.kb_click_mode.lock().await;

    let key = string_to_enigo_key(&key_str);
    let enigo_mod_keys = modifier_to_enigo_keys(modifiers);

    match mode {
        ClickMode::Press => {
            for &mod_key in &enigo_mod_keys {
                let _ = enigo.key(mod_key, Direction::Press);
            }

            let _ = enigo.key(key, Direction::Click);

            for &mod_key in enigo_mod_keys.iter().rev() {
                let _ = enigo.key(mod_key, Direction::Release);
            }
        }
        ClickMode::Hold => {
            let hold_duration = state.kb_hold_duration.load(Ordering::SeqCst);
            let unit = *state.kb_hold_unit.lock().await;
            let duration_ms = match unit {
                HoldUnit::Milliseconds => hold_duration,
                HoldUnit::Seconds => hold_duration * 1000,
            };

            for &mod_key in &enigo_mod_keys {
                let _ = enigo.key(mod_key, Direction::Press);
            }

            let _ = enigo.key(key, Direction::Press);

            tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms as u64)).await;

            let _ = enigo.key(key, Direction::Release);

            for &mod_key in enigo_mod_keys.iter().rev() {
                let _ = enigo.key(mod_key, Direction::Release);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn wait_interval(interval_us: u64) {
    let target = std::time::Instant::now() + std::time::Duration::from_micros(interval_us);
    while let Some(remaining) = target.checked_duration_since(std::time::Instant::now()) {
        if remaining > std::time::Duration::from_micros(250) {
            std::thread::yield_now();
        } else {
            std::hint::spin_loop();
        }
    }
}

#[cfg(not(target_os = "windows"))]
async fn wait_interval(interval_us: u64) {
    if interval_us < 1000 {
        std::thread::sleep(std::time::Duration::from_micros(interval_us));
    } else {
        tokio::time::sleep(tokio::time::Duration::from_micros(interval_us)).await;
    }
}

pub async fn keyboard_clicker_task(state: Arc<AppState>, app: AppHandle) {
    #[cfg(not(target_os = "linux"))]
    let mut enigo = Enigo::new(&Settings::default()).expect("Enigo init failed");

    let mut click_count: u32 = 0;
    let mut start_time = std::time::Instant::now();

    loop {
        if state.kb_is_running.load(Ordering::SeqCst) {
            if state.is_main_focused.load(Ordering::SeqCst)
                && !state.is_cps_test_focused.load(Ordering::SeqCst)
            {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                continue;
            }

            let repeat_mode = *state.kb_repeat_mode.lock().await;
            if repeat_mode == RepeatMode::Finite {
                let repeat_count = state.kb_repeat_count.load(Ordering::SeqCst);
                let repeat_unit = *state.kb_repeat_unit.lock().await;

                match repeat_unit {
                    RepeatUnit::Times => {
                        if click_count >= repeat_count {
                            state.kb_is_running.store(false, Ordering::SeqCst);
                            let _ = app.emit(
                                "keyboard-status-changed",
                                serde_json::json!({ "running": false }),
                            );
                            click_count = 0;
                            continue;
                        }
                    }
                    RepeatUnit::Seconds => {
                        let elapsed = start_time.elapsed().as_secs();
                        if elapsed >= repeat_count as u64 {
                            state.kb_is_running.store(false, Ordering::SeqCst);
                            let _ = app.emit(
                                "keyboard-status-changed",
                                serde_json::json!({ "running": false }),
                            );
                            start_time = std::time::Instant::now();
                            continue;
                        }
                    }
                }
            }

            let cps = state.kb_cps.load(Ordering::SeqCst);
            let multithread_active = state.is_multithread_active.load(Ordering::SeqCst);
            let threads = if multithread_active {
                state.threads_count.load(Ordering::SeqCst).max(1)
            } else {
                1
            };

            #[cfg(target_os = "windows")]
            let total_target_cps = {
                let requested = if cps == 0 {
                    1000u32.saturating_mul(threads)
                } else {
                    cps.saturating_mul(threads)
                };
                if multithread_active {
                    requested.max(1)
                } else {
                    requested.clamp(1, WINDOWS_MAX_TOTAL_CPS)
                }
            };

            #[cfg(not(target_os = "windows"))]
            let total_target_cps = if cps == 0 {
                1000u32.saturating_mul(threads)
            } else {
                cps.saturating_mul(threads)
            }
            .max(1);

            let interval_us = (1_000_000u32 / total_target_cps).max(200);

            let variation_ms = state.kb_variation_ms.load(Ordering::SeqCst);
            let mut final_interval_us = if variation_ms > 0 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let variation_us = rng.gen_range(0..=variation_ms * 1000) as u64;

                if rng.gen_bool(0.5) {
                    (interval_us as u64 + variation_us) as u32
                } else {
                    (interval_us as u64).saturating_sub(variation_us) as u32
                }
            } else {
                interval_us
            };

            #[cfg(target_os = "windows")]
            {
                if !multithread_active {
                    let min_interval_us = 1_000_000u32 / WINDOWS_MAX_TOTAL_CPS;
                    final_interval_us = final_interval_us.max(min_interval_us);
                }
            }

            #[cfg(target_os = "linux")]
            {
                let mut device_guard = state.keyboard_uinput_device.lock().await;
                if let Some(ref mut device) = *device_guard {
                    perform_keyboard_press(device, &state).await;
                } else {
                    drop(device_guard);
                    let mut dg = state.keyboard_uinput_device.lock().await;
                    *dg = crate::engine::keyboard_uinput::setup_keyboard_uinput();
                    if let Some(ref mut device) = *dg {
                        perform_keyboard_press(device, &state).await;
                    }
                }
            }

            #[cfg(not(target_os = "linux"))]
            {
                perform_keyboard_press(&mut enigo, &state).await;
            }

            if repeat_mode == RepeatMode::Finite {
                let repeat_unit = *state.kb_repeat_unit.lock().await;
                if repeat_unit == RepeatUnit::Times {
                    click_count += 1;
                }
            }

            #[cfg(target_os = "windows")]
            wait_interval(final_interval_us as u64);

            #[cfg(not(target_os = "windows"))]
            wait_interval(final_interval_us as u64).await;
        } else {
            click_count = 0;
            start_time = std::time::Instant::now();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}
