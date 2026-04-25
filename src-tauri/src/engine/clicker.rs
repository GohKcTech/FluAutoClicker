use crate::engine::state::{
    AppState, ClickMode, HoldUnit, MouseButton, PositionMode, RepeatMode, RepeatUnit,
};
use enigo::{Enigo, Mouse, Settings};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "windows")]
const WINDOWS_MAX_TOTAL_CPS: u32 = 650;

#[cfg(not(target_os = "linux"))]
use enigo::{Button, Coordinate, Direction};

#[cfg(target_os = "linux")]
use evdev::{EventType, InputEvent, Key};

#[cfg(target_os = "linux")]
fn mouse_button_to_key(btn: MouseButton) -> Key {
    match btn {
        MouseButton::Left => Key::BTN_LEFT,
        MouseButton::Right => Key::BTN_RIGHT,
        MouseButton::Middle => Key::BTN_MIDDLE,
        MouseButton::Front => Key::BTN_SIDE,
        MouseButton::Back => Key::BTN_EXTRA,
    }
}

#[cfg(not(target_os = "linux"))]
fn mouse_button_to_enigo(btn: MouseButton) -> Result<Button, String> {
    match btn {
        MouseButton::Left => Ok(Button::Left),
        MouseButton::Right => Ok(Button::Right),
        MouseButton::Middle => Ok(Button::Middle),
        MouseButton::Front => map_extended_button(),
        MouseButton::Back => map_back_button(),
    }
}

#[cfg(target_os = "macos")]
fn map_extended_button() -> Result<Button, String> {
    Err("Front mouse button is not supported on macOS in this build.".to_string())
}

#[cfg(not(target_os = "macos"))]
fn map_extended_button() -> Result<Button, String> {
    Ok(Button::Back)
}

#[cfg(target_os = "macos")]
fn map_back_button() -> Result<Button, String> {
    Err("Back mouse button is not supported on macOS in this build.".to_string())
}

#[cfg(not(target_os = "macos"))]
fn map_back_button() -> Result<Button, String> {
    Ok(Button::Forward)
}

#[cfg(target_os = "linux")]
async fn perform_click(device: &mut evdev::uinput::VirtualDevice, state: &AppState) {
    let btn = *state.mouse_button.lock().await;
    let mode = *state.click_mode.lock().await;
    let key = mouse_button_to_key(btn);

    match mode {
        ClickMode::Press => {
            let _ = device.emit(&[
                InputEvent::new(EventType::KEY, key.0, 1),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                InputEvent::new(EventType::KEY, key.0, 0),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);
        }
        ClickMode::Hold => {
            let hold_duration = state.hold_duration.load(Ordering::SeqCst);
            let unit = *state.hold_unit.lock().await;
            let duration_ms = match unit {
                HoldUnit::Milliseconds => hold_duration,
                HoldUnit::Seconds => hold_duration * 1000,
            };

            let _ = device.emit(&[
                InputEvent::new(EventType::KEY, key.0, 1),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);

            tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms as u64)).await;

            let _ = device.emit(&[
                InputEvent::new(EventType::KEY, key.0, 0),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);
        }
    }
}

#[cfg(not(target_os = "linux"))]
async fn perform_click(enigo: &mut Enigo, state: &AppState) {
    let btn = *state.mouse_button.lock().await;
    let mode = *state.click_mode.lock().await;
    let enigo_btn = match mouse_button_to_enigo(btn) {
        Ok(button) => button,
        Err(_) => return,
    };

    match mode {
        ClickMode::Press => {
            let _ = enigo.button(enigo_btn, Direction::Click);
        }
        ClickMode::Hold => {
            let hold_duration = state.hold_duration.load(Ordering::SeqCst);
            let unit = *state.hold_unit.lock().await;
            let duration_ms = match unit {
                HoldUnit::Milliseconds => hold_duration,
                HoldUnit::Seconds => hold_duration * 1000,
            };

            let _ = enigo.button(enigo_btn, Direction::Press);

            tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms as u64)).await;

            let _ = enigo.button(enigo_btn, Direction::Release);
        }
    }
}

#[cfg(target_os = "linux")]
async fn move_to_position(device: &mut evdev::uinput::VirtualDevice, x: i32, y: i32) {
    let _ = device.emit(&[
        InputEvent::new(EventType::ABSOLUTE, evdev::AbsoluteAxisType::ABS_X.0, x),
        InputEvent::new(EventType::ABSOLUTE, evdev::AbsoluteAxisType::ABS_Y.0, y),
        InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
    ]);
}

#[cfg(not(target_os = "linux"))]
async fn move_to_position(enigo: &mut Enigo, x: i32, y: i32) {
    let _ = enigo.move_mouse(x, y, Coordinate::Abs);
}

fn current_cursor_position() -> Result<(i32, i32), String> {
    let enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize input bridge: {e}"))?;
    enigo
        .location()
        .map_err(|e| format!("Failed to read cursor position: {e}"))
}

fn should_stop_on_custom_position_move(
    enabled: bool,
    primed: bool,
    current_position: Option<(i32, i32)>,
    target_position: (i32, i32),
) -> bool {
    enabled && primed && current_position.is_some_and(|position| position != target_position)
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

pub async fn click_task(state: Arc<AppState>, app: AppHandle) {
    #[cfg(not(target_os = "linux"))]
    let mut enigo = Enigo::new(&Settings::default()).expect("Enigo init failed");

    let mut click_count: u32 = 0;
    let mut start_time = std::time::Instant::now();
    let mut custom_position_primed = false;
    let mut last_custom_target: Option<(i32, i32)> = None;

    loop {
        if state.is_running.load(Ordering::SeqCst) {
            if state.is_main_focused.load(Ordering::SeqCst)
                && !state.is_cps_test_focused.load(Ordering::SeqCst)
            {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                continue;
            }

            let repeat_mode = *state.repeat_mode.lock().await;
            if repeat_mode == RepeatMode::Finite {
                let repeat_count = state.repeat_count.load(Ordering::SeqCst);
                let repeat_unit = *state.repeat_unit.lock().await;

                match repeat_unit {
                    RepeatUnit::Times => {
                        if click_count >= repeat_count {
                            state.is_running.store(false, Ordering::SeqCst);
                            let _ =
                                app.emit("status-changed", serde_json::json!({ "running": false }));
                            click_count = 0;
                            continue;
                        }
                    }
                    RepeatUnit::Seconds => {
                        let elapsed = start_time.elapsed().as_secs();
                        if elapsed >= repeat_count as u64 {
                            state.is_running.store(false, Ordering::SeqCst);
                            let _ =
                                app.emit("status-changed", serde_json::json!({ "running": false }));
                            start_time = std::time::Instant::now();
                            continue;
                        }
                    }
                }
            }

            let cps = state.cps.load(Ordering::SeqCst);
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

            let variation_ms = state.variation_ms.load(Ordering::SeqCst);
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

            let position_mode = *state.position_mode.lock().await;
            if position_mode == PositionMode::Custom {
                let x = state.coord_x.load(Ordering::SeqCst);
                let y = state.coord_y.load(Ordering::SeqCst);
                let target_position = (x, y);

                if last_custom_target != Some(target_position) {
                    custom_position_primed = false;
                    last_custom_target = Some(target_position);
                }

                let stop_on_move = state.stop_on_custom_position_move.load(Ordering::SeqCst);
                let current_position = if stop_on_move && custom_position_primed {
                    current_cursor_position().ok()
                } else {
                    None
                };

                if should_stop_on_custom_position_move(
                    stop_on_move,
                    custom_position_primed,
                    current_position,
                    target_position,
                ) {
                    state.is_running.store(false, Ordering::SeqCst);
                    let _ = app.emit("status-changed", serde_json::json!({ "running": false }));
                    custom_position_primed = false;
                    click_count = 0;
                    start_time = std::time::Instant::now();
                    continue;
                }

                #[cfg(target_os = "linux")]
                {
                    let mut device_guard = state.uinput_device.lock().await;
                    if let Some(ref mut device) = *device_guard {
                        move_to_position(device, x, y).await;
                    } else {
                        drop(device_guard);
                        let mut dg = state.uinput_device.lock().await;
                        *dg = crate::engine::uinput::setup_uinput();
                        if let Some(ref mut device) = *dg {
                            move_to_position(device, x, y).await;
                        }
                    }
                }

                #[cfg(not(target_os = "linux"))]
                {
                    move_to_position(&mut enigo, x, y).await;
                }

                custom_position_primed = true;
            } else {
                custom_position_primed = false;
                last_custom_target = None;
            }

            #[cfg(target_os = "linux")]
            {
                let mut device_guard = state.uinput_device.lock().await;
                if let Some(ref mut device) = *device_guard {
                    perform_click(device, &state).await;
                } else {
                    drop(device_guard);
                    let mut dg = state.uinput_device.lock().await;
                    *dg = crate::engine::uinput::setup_uinput();
                    if let Some(ref mut device) = *dg {
                        perform_click(device, &state).await;
                    }
                }
            }

            #[cfg(not(target_os = "linux"))]
            {
                perform_click(&mut enigo, &state).await;
            }

            if repeat_mode == RepeatMode::Finite {
                let repeat_unit = *state.repeat_unit.lock().await;
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
            custom_position_primed = false;
            last_custom_target = None;
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::should_stop_on_custom_position_move;

    #[test]
    fn does_not_stop_until_custom_position_is_primed() {
        assert!(!should_stop_on_custom_position_move(
            true,
            false,
            Some((120, 240)),
            (100, 200),
        ));
    }

    #[test]
    fn stops_when_cursor_leaves_custom_position() {
        assert!(should_stop_on_custom_position_move(
            true,
            true,
            Some((120, 240)),
            (100, 200),
        ));
    }

    #[test]
    fn ignores_cursor_movement_when_feature_is_disabled() {
        assert!(!should_stop_on_custom_position_move(
            false,
            true,
            Some((120, 240)),
            (100, 200),
        ));
    }
}
