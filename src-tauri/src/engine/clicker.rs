use crate::engine::executor::{interruptible_wait, Cancellation, WaitOutcome};
use crate::engine::input::{InputError, InputMouseButton, InputSession, InputSink, InputToken};
use crate::engine::runtime::StopPolicy;
use crate::engine::state::{
    AppState, ClickMode, HoldUnit, JigglerPattern, MouseButton, PositionMode, RepeatMode,
    RepeatUnit,
};
use enigo::{Enigo, Mouse, Settings};
use rand::SeedableRng;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::time::Instant;

/// Stable failure categories for real input injection, so the UI can
/// explain e.g. that a higher-integrity foreground window may require
/// launching this tool manually at the same integrity level. This process
/// never triggers elevation automatically.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionError {
    PermissionMismatch(String),
    UnsupportedKey(String),
    SendFailed(String),
}

impl From<InputError> for ExecutionError {
    fn from(err: InputError) -> Self {
        ExecutionError::SendFailed(err.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunEnd {
    Completed,
    Cancelled,
    Deadline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseHoldMode {
    Press,
    Hold(Duration),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MouseRunConfig {
    pub button: InputMouseButton,
    pub hold: MouseHoldMode,
    pub interval: Duration,
}

fn stop_policy_deadline(stop_policy: StopPolicy) -> Option<Instant> {
    match stop_policy {
        StopPolicy::DurationMs(ms) => Some(Instant::now() + Duration::from_millis(ms)),
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
    cancel: &Cancellation,
    deadline: Option<Instant>,
) -> Option<RunEnd> {
    match interruptible_wait(duration, cancel.clone(), deadline).await {
        WaitOutcome::Cancelled => Some(RunEnd::Cancelled),
        WaitOutcome::DeadlineReached => Some(RunEnd::Deadline),
        WaitOutcome::Completed => None,
    }
}

/// Runs one mouse-click executor to completion. One complete cycle is a
/// single down/up pair (or a held down for `MouseHoldMode::Hold`); the
/// session guarantees the button is released on every return path,
/// including a cancelled or deadline-cut hold.
pub async fn run_mouse<S: InputSink>(
    config: MouseRunConfig,
    stop_policy: StopPolicy,
    cancel: Cancellation,
    sink: S,
) -> Result<RunEnd, ExecutionError> {
    let deadline = stop_policy_deadline(stop_policy);
    let repeat_count = stop_policy_repeat_count(stop_policy);
    let mut session = InputSession::new(sink);
    let token = InputToken::Mouse(config.button.clone());
    let mut completed: u32 = 0;

    loop {
        if let Some(count) = repeat_count {
            if completed >= count {
                return Ok(RunEnd::Completed);
            }
        }

        session.press(token.clone())?;

        if let MouseHoldMode::Hold(duration) = config.hold {
            if let Some(end) = wait_or_end(duration, &cancel, deadline).await {
                return Ok(end);
            }
        }

        session.release(&token)?;
        completed += 1;

        if let Some(end) = wait_or_end(config.interval, &cancel, deadline).await {
            return Ok(end);
        }
    }
}

#[cfg(target_os = "linux")]
const MIN_INTERVAL_US: u32 = 50;

#[cfg(not(target_os = "linux"))]
const MIN_INTERVAL_US: u32 = 650;

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

#[cfg(all(not(target_os = "macos"), not(target_os = "linux")))]
fn map_extended_button() -> Result<Button, String> {
    Ok(Button::Back)
}

#[cfg(target_os = "macos")]
fn map_back_button() -> Result<Button, String> {
    Err("Back mouse button is not supported on macOS in this build.".to_string())
}

#[cfg(all(not(target_os = "macos"), not(target_os = "linux")))]
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

/// Adapts a live `Enigo` handle to `InputSink` so the mouse click cycle can
/// go through `InputSession`'s guaranteed-release guard and, for a held
/// click, through `interruptible_wait` instead of an uncancellable sleep.
/// Keyboard input is out of scope for this sink; the mouse cycle never
/// calls those methods.
#[cfg(not(target_os = "linux"))]
struct EnigoMouseSink<'a> {
    enigo: &'a mut Enigo,
}

#[cfg(not(target_os = "linux"))]
fn input_button_to_state(button: InputMouseButton) -> MouseButton {
    match button {
        InputMouseButton::Left => MouseButton::Left,
        InputMouseButton::Middle => MouseButton::Middle,
        InputMouseButton::Right => MouseButton::Right,
        InputMouseButton::Front => MouseButton::Front,
        InputMouseButton::Back => MouseButton::Back,
    }
}

#[cfg(not(target_os = "linux"))]
impl<'a> InputSink for EnigoMouseSink<'a> {
    fn key_down(&mut self, _key: &str) -> Result<(), InputError> {
        Err(InputError::new(
            "keyboard input is not supported by the mouse sink",
        ))
    }

    fn key_up(&mut self, _key: &str) -> Result<(), InputError> {
        Err(InputError::new(
            "keyboard input is not supported by the mouse sink",
        ))
    }

    fn mouse_down(&mut self, button: InputMouseButton) -> Result<(), InputError> {
        let enigo_btn =
            mouse_button_to_enigo(input_button_to_state(button)).map_err(InputError::new)?;
        self.enigo
            .button(enigo_btn, Direction::Press)
            .map_err(|e| InputError::new(e.to_string()))
    }

    fn mouse_up(&mut self, button: InputMouseButton) -> Result<(), InputError> {
        let enigo_btn =
            mouse_button_to_enigo(input_button_to_state(button)).map_err(InputError::new)?;
        self.enigo
            .button(enigo_btn, Direction::Release)
            .map_err(|e| InputError::new(e.to_string()))
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), InputError> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| InputError::new(e.to_string()))
    }

    fn scroll(&mut self, clicks: i32) -> Result<(), InputError> {
        self.enigo
            .scroll(clicks, enigo::Axis::Vertical)
            .map_err(|e| InputError::new(e.to_string()))
    }
}

#[cfg(not(target_os = "linux"))]
fn to_input_mouse_button(button: MouseButton) -> InputMouseButton {
    match button {
        MouseButton::Left => InputMouseButton::Left,
        MouseButton::Middle => InputMouseButton::Middle,
        MouseButton::Right => InputMouseButton::Right,
        MouseButton::Front => InputMouseButton::Front,
        MouseButton::Back => InputMouseButton::Back,
    }
}

#[cfg(not(target_os = "linux"))]
async fn perform_click(enigo: &mut Enigo, state: &AppState) {
    let btn = *state.mouse_button.lock().await;
    let mode = *state.click_mode.lock().await;
    if mouse_button_to_enigo(btn).is_err() {
        return;
    }
    let token = InputToken::Mouse(to_input_mouse_button(btn));
    let mut session = InputSession::new(EnigoMouseSink { enigo });

    match mode {
        ClickMode::Press => {
            if session.press(token.clone()).is_ok() {
                let _ = session.release(&token);
            }
        }
        ClickMode::Hold => {
            let hold_duration = state.hold_duration.load(Ordering::SeqCst);
            let unit = *state.hold_unit.lock().await;
            let duration_ms = match unit {
                HoldUnit::Milliseconds => hold_duration,
                HoldUnit::Seconds => hold_duration * 1000,
            };

            if session.press(token.clone()).is_err() {
                return;
            }

            let cancel = state.mouse_cancel.lock().unwrap().clone();
            let outcome =
                interruptible_wait(Duration::from_millis(duration_ms as u64), cancel, None).await;

            // Whether the hold completed, was cancelled, or hit a deadline,
            // release now if still held; if this scope exits early on a
            // future refactor, InputSession's Drop guard covers it too.
            let _ = session.release(&token);
            let _ = outcome;
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

fn random_point_in_circle(rng: &mut impl rand::Rng, radius: i32) -> (i32, i32) {
    loop {
        let x = rng.gen_range(-radius..=radius);
        let y = rng.gen_range(-radius..=radius);
        if x * x + y * y <= radius * radius {
            return (x, y);
        }
    }
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
    if interval_us == 0 {
        return;
    }

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
    let mut ozone_rng = rand::rngs::SmallRng::from_entropy();
    let mut ozone_target: Option<(i32, i32)> = None;
    let mut ozone_next_move_at = std::time::Instant::now();

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
                            state.ozone_anchor_ready.store(false, Ordering::SeqCst);
                            state
                                .ozone_wait_for_click_anchor
                                .store(false, Ordering::SeqCst);
                            ozone_target = None;
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
                            state.ozone_anchor_ready.store(false, Ordering::SeqCst);
                            state
                                .ozone_wait_for_click_anchor
                                .store(false, Ordering::SeqCst);
                            ozone_target = None;
                            let _ =
                                app.emit("status-changed", serde_json::json!({ "running": false }));
                            start_time = std::time::Instant::now();
                            continue;
                        }
                    }
                }
            }

            let cps = state.cps.load(Ordering::SeqCst);

            #[cfg(target_os = "linux")]
            let total_target_cps = if cps == 0 { u32::MAX } else { cps }.max(1);

            #[cfg(not(target_os = "linux"))]
            let total_target_cps = if cps == 0 { 1000 } else { cps }.max(1);

            let interval_us = (1_000_000u32 / total_target_cps).max(MIN_INTERVAL_US);

            let variation_ms = state.variation_ms.load(Ordering::SeqCst);
            let final_interval_us = if variation_ms > 0 {
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
            }
            .max(MIN_INTERVAL_US);

            let ozone_active = state.is_jiggler_active.load(Ordering::SeqCst)
                && *state.jiggler_pattern.lock().await == JigglerPattern::OZone;

            if ozone_active {
                if !state.ozone_anchor_ready.load(Ordering::SeqCst) {
                    ozone_target = None;
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    continue;
                }

                let should_move_ozone =
                    ozone_target.is_none() || std::time::Instant::now() >= ozone_next_move_at;

                if should_move_ozone {
                    let radius = (state.jiggler_distance.load(Ordering::SeqCst) as i32).max(1);
                    let center_x = state.ozone_center_x.load(Ordering::SeqCst);
                    let center_y = state.ozone_center_y.load(Ordering::SeqCst);
                    let (offset_x, offset_y) = random_point_in_circle(&mut ozone_rng, radius);
                    let target = (center_x + offset_x, center_y + offset_y);
                    ozone_target = Some(target);
                    ozone_next_move_at = std::time::Instant::now()
                        + std::time::Duration::from_millis(
                            state.jiggler_interval.load(Ordering::SeqCst).max(100) as u64,
                        );
                }

                let Some((x, y)) = ozone_target else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    continue;
                };

                #[cfg(target_os = "linux")]
                if should_move_ozone {
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
                if should_move_ozone {
                    move_to_position(&mut enigo, x, y).await;
                }

                custom_position_primed = false;
                last_custom_target = None;
            } else {
                ozone_target = None;
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
                        state.ozone_anchor_ready.store(false, Ordering::SeqCst);
                        state
                            .ozone_wait_for_click_anchor
                            .store(false, Ordering::SeqCst);
                        ozone_target = None;
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
            ozone_target = None;
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::input::test_support::SharedFakeSink;

    fn press(button: InputMouseButton, interval_ms: u64) -> MouseRunConfig {
        MouseRunConfig {
            button,
            hold: MouseHoldMode::Press,
            interval: Duration::from_millis(interval_ms),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn mouse_count_means_complete_down_up_cycles() {
        let sink = SharedFakeSink::default();
        let end = run_mouse(
            press(InputMouseButton::Left, 10),
            StopPolicy::RepeatCount(3),
            Cancellation::new(),
            sink.clone(),
        )
        .await
        .unwrap();

        assert_eq!(end, RunEnd::Completed);
        assert_eq!(
            sink.complete_cycles(&InputToken::Mouse(InputMouseButton::Left)),
            3
        );
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn mouse_duration_interrupts_a_long_hold() {
        let sink = SharedFakeSink::default();
        let config = MouseRunConfig {
            button: InputMouseButton::Left,
            hold: MouseHoldMode::Hold(Duration::from_secs(30)),
            interval: Duration::from_millis(10),
        };
        let run = tokio::spawn(run_mouse(
            config,
            StopPolicy::DurationMs(1_000),
            Cancellation::new(),
            sink.clone(),
        ));
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(1)).await;

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Deadline);
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_during_the_interval_stops_before_the_next_cycle() {
        let sink = SharedFakeSink::default();
        let cancel = Cancellation::new();
        let run = tokio::spawn(run_mouse(
            press(InputMouseButton::Left, 60_000),
            StopPolicy::UntilStopped,
            cancel.clone(),
            sink.clone(),
        ));
        tokio::task::yield_now().await;
        cancel.cancel();

        assert_eq!(run.await.unwrap().unwrap(), RunEnd::Cancelled);
        assert_eq!(
            sink.complete_cycles(&InputToken::Mouse(InputMouseButton::Left)),
            1
        );
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn a_send_failure_on_the_first_down_never_enters_the_held_list() {
        let sink = SharedFakeSink::default();
        sink.fail_on_attempt(0);

        let result = run_mouse(
            press(InputMouseButton::Left, 10),
            StopPolicy::UntilStopped,
            Cancellation::new(),
            sink.clone(),
        )
        .await;

        assert_eq!(
            result,
            Err(ExecutionError::SendFailed("forced failure".to_string()))
        );
        assert!(sink.calls().is_empty());
        assert!(sink.no_inputs_held());
    }

    #[tokio::test(start_paused = true)]
    async fn a_send_failure_on_release_still_stops_the_run_cleanly() {
        let sink = SharedFakeSink::default();
        // Attempt 0 is the Down that succeeds; attempt 1 is the Up that fails.
        sink.fail_on_attempt(1);

        let result = run_mouse(
            press(InputMouseButton::Left, 10),
            StopPolicy::UntilStopped,
            Cancellation::new(),
            sink.clone(),
        )
        .await;

        assert_eq!(
            result,
            Err(ExecutionError::SendFailed("forced failure".to_string()))
        );
        // The sink rejected the physical release, so its own call log has
        // an unmatched Down; a lost physical release can't be retried, so
        // the executor does not attempt this button again after returning.
        assert!(!sink.no_inputs_held());
    }

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
