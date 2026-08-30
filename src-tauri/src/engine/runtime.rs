use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppMode {
    Mouse,
    Keyboard,
    Macro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePhase {
    Idle,
    Recording,
    RunningMouse,
    RunningKeyboard,
    PlayingMacro,
    Stopping,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum StopPolicy {
    UntilStopped,
    RepeatCount(u32),
    DurationMs(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    Busy(RuntimePhase),
    InvalidTransition,
    InvalidCount,
    InvalidDuration,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RuntimeSnapshot {
    pub phase: RuntimePhase,
    pub selected_mode: AppMode,
    pub completed_cycles: u64,
    pub last_error: Option<String>,
}

pub struct RuntimeModel {
    pub phase: RuntimePhase,
    pub selected_mode: AppMode,
    pub completed_cycles: u64,
    pub last_error: Option<String>,
}

impl Default for RuntimeModel {
    fn default() -> Self {
        Self {
            phase: RuntimePhase::Idle,
            selected_mode: AppMode::Mouse,
            completed_cycles: 0,
            last_error: None,
        }
    }
}

impl StopPolicy {
    pub fn validate(&self) -> Result<(), RuntimeError> {
        match self {
            StopPolicy::UntilStopped => Ok(()),
            StopPolicy::RepeatCount(0) => Err(RuntimeError::InvalidCount),
            StopPolicy::RepeatCount(_) => Ok(()),
            StopPolicy::DurationMs(0) => Err(RuntimeError::InvalidDuration),
            StopPolicy::DurationMs(_) => Ok(()),
        }
    }
}

impl RuntimeModel {
    pub fn start(&mut self, mode: AppMode, policy: StopPolicy) -> Result<u64, RuntimeError> {
        policy.validate()?;

        if self.phase != RuntimePhase::Idle {
            return Err(RuntimeError::Busy(self.phase));
        }

        self.phase = match mode {
            AppMode::Mouse => RuntimePhase::RunningMouse,
            AppMode::Keyboard => RuntimePhase::RunningKeyboard,
            AppMode::Macro => RuntimePhase::PlayingMacro,
        };
        self.selected_mode = mode;
        self.completed_cycles = 0;
        self.last_error = None;

        Ok(self.completed_cycles + 1)
    }

    pub fn start_recording(&mut self) -> Result<u64, RuntimeError> {
        if self.phase != RuntimePhase::Idle {
            return Err(RuntimeError::Busy(self.phase));
        }

        if self.selected_mode != AppMode::Macro {
            return Err(RuntimeError::InvalidTransition);
        }

        self.phase = RuntimePhase::Recording;
        self.completed_cycles = 0;
        self.last_error = None;

        Ok(self.completed_cycles + 1)
    }

    pub fn begin_stop(&mut self) -> Result<u64, RuntimeError> {
        if self.phase == RuntimePhase::Idle {
            return Err(RuntimeError::InvalidTransition);
        }

        self.phase = RuntimePhase::Stopping;

        Ok(self.completed_cycles)
    }

    pub fn finish_stop(&mut self) -> Result<u64, RuntimeError> {
        if self.phase != RuntimePhase::Stopping {
            return Err(RuntimeError::InvalidTransition);
        }

        self.phase = RuntimePhase::Idle;

        Ok(self.completed_cycles)
    }

    pub fn fail(&mut self, error: String) -> RuntimeSnapshot {
        let snapshot = RuntimeSnapshot {
            phase: RuntimePhase::Error,
            selected_mode: self.selected_mode,
            completed_cycles: self.completed_cycles,
            last_error: Some(error),
        };
        self.phase = RuntimePhase::Error;
        self.last_error = Some(snapshot.last_error.clone().unwrap_or_default());
        snapshot
    }

    pub fn recover_idle(&mut self) -> Result<u64, RuntimeError> {
        if self.phase != RuntimePhase::Error {
            return Err(RuntimeError::InvalidTransition);
        }

        let cycles = self.completed_cycles;
        self.phase = RuntimePhase::Idle;

        Ok(cycles)
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        if self.phase == RuntimePhase::Idle {
            self.selected_mode = mode;
        }
    }

    pub fn increment_cycle(&mut self) -> u64 {
        self.completed_cycles += 1;
        self.completed_cycles
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            phase: self.phase,
            selected_mode: self.selected_mode,
            completed_cycles: self.completed_cycles,
            last_error: self.last_error.clone(),
        }
    }
}

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RuntimeCoordinator {
    model: Arc<Mutex<RuntimeModel>>,
    generation: AtomicU64,
}

impl Default for RuntimeCoordinator {
    fn default() -> Self {
        Self {
            model: Arc::new(Mutex::new(RuntimeModel::default())),
            generation: AtomicU64::new(0),
        }
    }
}

impl RuntimeCoordinator {
    pub async fn start(&self, mode: AppMode, policy: StopPolicy) -> Result<u64, RuntimeError> {
        let mut model = self.model.lock().await;
        let result = model.start(mode, policy)?;
        self.generation.fetch_add(1, Ordering::SeqCst);
        Ok(result)
    }

    pub async fn start_recording(&self) -> Result<u64, RuntimeError> {
        let mut model = self.model.lock().await;
        let result = model.start_recording()?;
        self.generation.fetch_add(1, Ordering::SeqCst);
        Ok(result)
    }

    pub async fn begin_stop(&self) -> Result<u64, RuntimeError> {
        let mut model = self.model.lock().await;
        model.begin_stop()
    }

    pub async fn finish_stop(&self) -> Result<u64, RuntimeError> {
        let mut model = self.model.lock().await;
        model.finish_stop()
    }

    pub async fn fail(&self, error: String) -> RuntimeSnapshot {
        let mut model = self.model.lock().await;
        model.fail(error)
    }

    pub async fn recover_idle(&self) -> Result<u64, RuntimeError> {
        let mut model = self.model.lock().await;
        model.recover_idle()
    }

    pub async fn set_mode(&self, mode: AppMode) {
        let mut model = self.model.lock().await;
        model.set_mode(mode);
    }

    pub async fn increment_cycle(&self) -> u64 {
        let mut model = self.model.lock().await;
        model.increment_cycle()
    }

    pub async fn snapshot(&self) -> RuntimeSnapshot {
        let model = self.model.lock().await;
        model.snapshot()
    }

    pub fn current_generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_idle_with_mouse_and_zero_cycles() {
        let runtime = RuntimeModel::default();
        assert_eq!(runtime.phase, RuntimePhase::Idle);
        assert_eq!(runtime.selected_mode, AppMode::Mouse);
        assert_eq!(runtime.completed_cycles, 0);
        assert_eq!(runtime.last_error, None);
    }

    #[test]
    fn mode_change_does_not_start_execution() {
        let mut runtime = RuntimeModel::default();
        runtime.set_mode(AppMode::Keyboard);
        assert_eq!(runtime.phase, RuntimePhase::Idle);
        assert_eq!(runtime.selected_mode, AppMode::Keyboard);
    }

    #[test]
    fn validates_stop_policy_rejects_zero_count() {
        assert_eq!(
            StopPolicy::RepeatCount(0).validate(),
            Err(RuntimeError::InvalidCount)
        );
    }

    #[test]
    fn validates_stop_policy_rejects_zero_duration() {
        assert_eq!(
            StopPolicy::DurationMs(0).validate(),
            Err(RuntimeError::InvalidDuration)
        );
    }

    #[test]
    fn validates_stop_policy_accepts_positive_count() {
        assert!(StopPolicy::RepeatCount(1).validate().is_ok());
    }

    #[test]
    fn validates_stop_policy_accepts_positive_duration() {
        assert!(StopPolicy::DurationMs(1).validate().is_ok());
    }

    #[test]
    fn validates_stop_policy_accepts_until_stopped() {
        assert!(StopPolicy::UntilStopped.validate().is_ok());
    }

    #[test]
    fn start_from_idle_clears_error_and_resets_cycles() {
        let mut runtime = RuntimeModel::default();
        runtime.last_error = Some("previous error".to_string());
        runtime.completed_cycles = 5;

        let gen = runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();

        assert_eq!(runtime.last_error, None);
        assert_eq!(runtime.completed_cycles, 0);
        assert_eq!(gen, 1);
    }

    #[test]
    fn rejects_reentrant_start() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();
        assert_eq!(
            runtime.start(AppMode::Keyboard, StopPolicy::RepeatCount(2)),
            Err(RuntimeError::Busy(RuntimePhase::RunningMouse))
        );
    }

    #[test]
    fn start_with_mouse_selects_running_mouse_phase() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();
        assert_eq!(runtime.phase, RuntimePhase::RunningMouse);
    }

    #[test]
    fn start_with_keyboard_selects_running_keyboard_phase() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Keyboard, StopPolicy::UntilStopped)
            .unwrap();
        assert_eq!(runtime.phase, RuntimePhase::RunningKeyboard);
    }

    #[test]
    fn start_with_macro_selects_playing_macro_phase() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Macro, StopPolicy::UntilStopped)
            .unwrap();
        assert_eq!(runtime.phase, RuntimePhase::PlayingMacro);
    }

    #[test]
    fn recording_starts_only_from_idle_for_macro_mode() {
        let mut runtime = RuntimeModel::default();
        runtime.selected_mode = AppMode::Macro;

        let gen = runtime.start_recording().unwrap();

        assert_eq!(runtime.phase, RuntimePhase::Recording);
        assert_eq!(gen, 1);
    }

    #[test]
    fn recording_rejected_for_non_macro_mode() {
        let mut runtime = RuntimeModel::default();
        runtime.selected_mode = AppMode::Mouse;

        assert_eq!(
            runtime.start_recording(),
            Err(RuntimeError::InvalidTransition)
        );
    }

    #[test]
    fn recording_rejected_when_already_running() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();

        assert_eq!(
            runtime.start_recording(),
            Err(RuntimeError::Busy(RuntimePhase::RunningMouse))
        );
    }

    #[test]
    fn running_and_recording_are_mutually_exclusive() {
        let mut runtime = RuntimeModel::default();

        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();
        assert_eq!(
            runtime.start_recording(),
            Err(RuntimeError::Busy(RuntimePhase::RunningMouse))
        );

        let mut runtime2 = RuntimeModel::default();
        runtime2.selected_mode = AppMode::Macro;
        runtime2.start_recording().unwrap();
        assert_eq!(
            runtime2.start(AppMode::Mouse, StopPolicy::UntilStopped),
            Err(RuntimeError::Busy(RuntimePhase::Recording))
        );
    }

    #[test]
    fn begin_stop_from_running() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();

        let cycles = runtime.begin_stop().unwrap();

        assert_eq!(runtime.phase, RuntimePhase::Stopping);
        assert_eq!(cycles, 0);
    }

    #[test]
    fn begin_stop_from_recording() {
        let mut runtime = RuntimeModel::default();
        runtime.selected_mode = AppMode::Macro;
        runtime.start_recording().unwrap();

        let cycles = runtime.begin_stop().unwrap();

        assert_eq!(runtime.phase, RuntimePhase::Stopping);
        assert_eq!(cycles, 0);
    }

    #[test]
    fn begin_stop_rejected_from_idle() {
        let mut runtime = RuntimeModel::default();

        assert_eq!(runtime.begin_stop(), Err(RuntimeError::InvalidTransition));
    }

    #[test]
    fn finish_stop_changes_stopping_to_idle() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();
        runtime.begin_stop().unwrap();

        let cycles = runtime.finish_stop().unwrap();

        assert_eq!(runtime.phase, RuntimePhase::Idle);
        assert_eq!(cycles, 0);
    }

    #[test]
    fn finish_stop_rejected_from_idle() {
        let mut runtime = RuntimeModel::default();

        assert_eq!(runtime.finish_stop(), Err(RuntimeError::InvalidTransition));
    }

    #[test]
    fn fail_records_error_and_enters_error_phase() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();

        let snapshot = runtime.fail("execution failed".to_string());

        assert_eq!(runtime.phase, RuntimePhase::Error);
        assert_eq!(runtime.last_error, Some("execution failed".to_string()));
        assert_eq!(snapshot.phase, RuntimePhase::Error);
        assert_eq!(snapshot.last_error, Some("execution failed".to_string()));
    }

    #[test]
    fn recover_idle_from_error_clears_phase_but_keeps_error() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();
        runtime.fail("execution failed".to_string());

        let cycles = runtime.recover_idle().unwrap();

        assert_eq!(runtime.phase, RuntimePhase::Idle);
        assert_eq!(runtime.last_error, Some("execution failed".to_string()));
        assert_eq!(cycles, 0);
    }

    #[test]
    fn recover_idle_rejected_from_idle() {
        let mut runtime = RuntimeModel::default();

        assert_eq!(runtime.recover_idle(), Err(RuntimeError::InvalidTransition));
    }

    #[test]
    fn snapshot_reflects_phase_mode_cycles_and_error() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Keyboard, StopPolicy::UntilStopped)
            .unwrap();
        runtime.increment_cycle();
        runtime.increment_cycle();

        let snapshot = runtime.snapshot();

        assert_eq!(snapshot.phase, RuntimePhase::RunningKeyboard);
        assert_eq!(snapshot.selected_mode, AppMode::Keyboard);
        assert_eq!(snapshot.completed_cycles, 2);
        assert_eq!(snapshot.last_error, None);
    }

    #[test]
    fn invalid_transition_does_not_mutate_state() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();

        let _ = runtime.start(AppMode::Keyboard, StopPolicy::RepeatCount(1));

        assert_eq!(runtime.phase, RuntimePhase::RunningMouse);
        assert_eq!(runtime.selected_mode, AppMode::Mouse);
    }

    #[test]
    fn increment_cycle_increments_completed_cycles() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();

        let cycle1 = runtime.increment_cycle();
        let cycle2 = runtime.increment_cycle();

        assert_eq!(cycle1, 1);
        assert_eq!(cycle2, 2);
        assert_eq!(runtime.completed_cycles, 2);
    }
}
