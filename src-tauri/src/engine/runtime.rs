use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

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

impl StopPolicy {
    pub fn validate(self) -> Result<(), RuntimeError> {
        match self {
            Self::UntilStopped | Self::RepeatCount(1..) | Self::DurationMs(1..) => Ok(()),
            Self::RepeatCount(0) => Err(RuntimeError::InvalidCount),
            Self::DurationMs(0) => Err(RuntimeError::InvalidDuration),
        }
    }
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

#[derive(Clone, Debug)]
pub struct RuntimeModel {
    phase: RuntimePhase,
    selected_mode: AppMode,
    completed_cycles: u64,
    last_error: Option<String>,
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

impl RuntimeModel {
    pub fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            phase: self.phase,
            selected_mode: self.selected_mode,
            completed_cycles: self.completed_cycles,
            last_error: self.last_error.clone(),
        }
    }

    pub fn set_selected_mode(&mut self, mode: AppMode) -> Result<(), RuntimeError> {
        if self.phase != RuntimePhase::Idle {
            return Err(RuntimeError::Busy(self.phase));
        }

        self.selected_mode = mode;
        Ok(())
    }

    pub fn start(&mut self, mode: AppMode, stop_policy: StopPolicy) -> Result<(), RuntimeError> {
        if self.phase != RuntimePhase::Idle {
            return Err(RuntimeError::Busy(self.phase));
        }
        stop_policy.validate()?;

        self.selected_mode = mode;
        self.completed_cycles = 0;
        self.last_error = None;
        self.phase = match mode {
            AppMode::Mouse => RuntimePhase::RunningMouse,
            AppMode::Keyboard => RuntimePhase::RunningKeyboard,
            AppMode::Macro => RuntimePhase::PlayingMacro,
        };
        Ok(())
    }

    pub fn begin_recording(&mut self) -> Result<(), RuntimeError> {
        if self.phase != RuntimePhase::Idle {
            return Err(RuntimeError::Busy(self.phase));
        }
        if self.selected_mode != AppMode::Macro {
            return Err(RuntimeError::InvalidTransition);
        }

        self.completed_cycles = 0;
        self.last_error = None;
        self.phase = RuntimePhase::Recording;
        Ok(())
    }

    pub fn record_cycle_completed(&mut self) -> Result<(), RuntimeError> {
        if !matches!(
            self.phase,
            RuntimePhase::RunningMouse | RuntimePhase::RunningKeyboard | RuntimePhase::PlayingMacro
        ) {
            return Err(RuntimeError::InvalidTransition);
        }

        self.completed_cycles += 1;
        Ok(())
    }

    pub fn begin_stop(&mut self) -> Result<(), RuntimeError> {
        if !matches!(
            self.phase,
            RuntimePhase::Recording
                | RuntimePhase::RunningMouse
                | RuntimePhase::RunningKeyboard
                | RuntimePhase::PlayingMacro
        ) {
            return Err(RuntimeError::InvalidTransition);
        }

        self.phase = RuntimePhase::Stopping;
        Ok(())
    }

    pub fn finish_stop(&mut self) -> Result<(), RuntimeError> {
        if self.phase != RuntimePhase::Stopping {
            return Err(RuntimeError::InvalidTransition);
        }

        self.phase = RuntimePhase::Idle;
        Ok(())
    }

    pub fn fail_execution(
        &mut self,
        message: impl Into<String>,
    ) -> Result<RuntimeSnapshot, RuntimeError> {
        if !matches!(
            self.phase,
            RuntimePhase::Recording
                | RuntimePhase::RunningMouse
                | RuntimePhase::RunningKeyboard
                | RuntimePhase::PlayingMacro
                | RuntimePhase::Stopping
        ) {
            return Err(RuntimeError::InvalidTransition);
        }

        self.last_error = Some(message.into());
        self.phase = RuntimePhase::Error;
        Ok(self.snapshot())
    }

    pub fn recover_idle(&mut self) -> Result<RuntimeSnapshot, RuntimeError> {
        if self.phase != RuntimePhase::Error {
            return Err(RuntimeError::InvalidTransition);
        }

        self.phase = RuntimePhase::Idle;
        Ok(self.snapshot())
    }
}

pub struct RuntimeCoordinator {
    model: Mutex<RuntimeModel>,
    cancellation_generation: AtomicU64,
}

impl Default for RuntimeCoordinator {
    fn default() -> Self {
        Self::new(RuntimeModel::default())
    }
}

impl RuntimeCoordinator {
    pub fn new(model: RuntimeModel) -> Self {
        Self {
            model: Mutex::new(model),
            cancellation_generation: AtomicU64::new(0),
        }
    }

    pub fn current_generation(&self) -> u64 {
        self.cancellation_generation.load(Ordering::SeqCst)
    }

    pub async fn snapshot(&self) -> RuntimeSnapshot {
        self.model.lock().await.snapshot()
    }

    pub async fn set_selected_mode(&self, mode: AppMode) -> Result<RuntimeSnapshot, RuntimeError> {
        let mut model = self.model.lock().await;
        model.set_selected_mode(mode)?;
        Ok(model.snapshot())
    }

    pub async fn begin_recording(&self) -> Result<RuntimeSnapshot, RuntimeError> {
        let mut model = self.model.lock().await;
        model.begin_recording()?;
        self.cancellation_generation.fetch_add(1, Ordering::SeqCst);
        Ok(model.snapshot())
    }

    pub async fn record_cycle_completed(&self) -> Result<RuntimeSnapshot, RuntimeError> {
        let mut model = self.model.lock().await;
        model.record_cycle_completed()?;
        Ok(model.snapshot())
    }

    pub async fn start(
        &self,
        mode: AppMode,
        stop_policy: StopPolicy,
    ) -> Result<RuntimeSnapshot, RuntimeError> {
        let mut model = self.model.lock().await;
        model.start(mode, stop_policy)?;
        self.cancellation_generation.fetch_add(1, Ordering::SeqCst);
        Ok(model.snapshot())
    }

    pub async fn begin_stop(&self) -> Result<RuntimeSnapshot, RuntimeError> {
        let mut model = self.model.lock().await;
        model.begin_stop()?;
        Ok(model.snapshot())
    }

    pub async fn finish_stop(&self) -> Result<RuntimeSnapshot, RuntimeError> {
        let mut model = self.model.lock().await;
        model.finish_stop()?;
        Ok(model.snapshot())
    }

    pub async fn fail_execution(
        &self,
        message: impl Into<String>,
    ) -> Result<(RuntimeSnapshot, RuntimeSnapshot), RuntimeError> {
        let mut model = self.model.lock().await;
        let error_snapshot = model.fail_execution(message)?;
        let idle_snapshot = model.recover_idle()?;
        Ok((error_snapshot, idle_snapshot))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_is_idle_after_restoring_selected_mode() {
        let mut runtime = RuntimeModel::default();
        runtime.set_selected_mode(AppMode::Keyboard).unwrap();

        assert_eq!(
            runtime.snapshot(),
            RuntimeSnapshot {
                phase: RuntimePhase::Idle,
                selected_mode: AppMode::Keyboard,
                completed_cycles: 0,
                last_error: None,
            }
        );
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
    fn validates_stop_policy() {
        assert_eq!(
            StopPolicy::RepeatCount(0).validate(),
            Err(RuntimeError::InvalidCount)
        );
        assert_eq!(
            StopPolicy::DurationMs(0).validate(),
            Err(RuntimeError::InvalidDuration)
        );
    }

    #[test]
    fn recording_and_playback_are_mutually_exclusive() {
        let mut playback = RuntimeModel::default();
        playback
            .start(AppMode::Macro, StopPolicy::UntilStopped)
            .unwrap();
        let playback_snapshot = playback.snapshot();
        assert_eq!(
            playback.begin_recording(),
            Err(RuntimeError::Busy(RuntimePhase::PlayingMacro))
        );
        assert_eq!(playback.snapshot(), playback_snapshot);

        let mut recording = RuntimeModel::default();
        recording.set_selected_mode(AppMode::Macro).unwrap();
        recording.begin_recording().unwrap();
        let recording_snapshot = recording.snapshot();
        assert_eq!(
            recording.start(AppMode::Macro, StopPolicy::UntilStopped),
            Err(RuntimeError::Busy(RuntimePhase::Recording))
        );
        assert_eq!(recording.snapshot(), recording_snapshot);
    }

    #[test]
    fn transitions_running_to_stopping_to_idle() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Keyboard, StopPolicy::RepeatCount(2))
            .unwrap();

        runtime.begin_stop().unwrap();
        assert_eq!(runtime.snapshot().phase, RuntimePhase::Stopping);
        runtime.finish_stop().unwrap();
        assert_eq!(runtime.snapshot().phase, RuntimePhase::Idle);
    }

    #[test]
    fn invalid_transition_does_not_mutate_state() {
        let mut runtime = RuntimeModel::default();
        runtime.set_selected_mode(AppMode::Macro).unwrap();
        let snapshot = runtime.snapshot();

        assert_eq!(runtime.finish_stop(), Err(RuntimeError::InvalidTransition));
        assert_eq!(runtime.snapshot(), snapshot);
    }

    #[test]
    fn execution_error_returns_to_idle_until_next_successful_start() {
        let mut runtime = RuntimeModel::default();
        runtime
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .unwrap();

        let error_snapshot = runtime.fail_execution("input disconnected").unwrap();
        assert_eq!(error_snapshot.phase, RuntimePhase::Error);
        assert_eq!(
            error_snapshot.last_error.as_deref(),
            Some("input disconnected")
        );

        let idle_snapshot = runtime.recover_idle().unwrap();
        assert_eq!(idle_snapshot.phase, RuntimePhase::Idle);
        assert_eq!(
            idle_snapshot.last_error.as_deref(),
            Some("input disconnected")
        );

        runtime
            .start(AppMode::Keyboard, StopPolicy::UntilStopped)
            .unwrap();
        assert_eq!(runtime.snapshot().last_error, None);
    }

    #[tokio::test]
    async fn coordinator_advances_generation_only_for_successful_starts() {
        let coordinator = RuntimeCoordinator::default();
        assert_eq!(coordinator.current_generation(), 0);

        let first = coordinator
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .await
            .unwrap();
        assert_eq!(first.phase, RuntimePhase::RunningMouse);
        assert_eq!(coordinator.current_generation(), 1);

        assert_eq!(
            coordinator
                .start(AppMode::Keyboard, StopPolicy::UntilStopped)
                .await,
            Err(RuntimeError::Busy(RuntimePhase::RunningMouse))
        );
        assert_eq!(coordinator.current_generation(), 1);

        coordinator.begin_stop().await.unwrap();
        coordinator.finish_stop().await.unwrap();
        let second = coordinator
            .start(AppMode::Keyboard, StopPolicy::RepeatCount(2))
            .await
            .unwrap();
        assert_eq!(second.phase, RuntimePhase::RunningKeyboard);
        assert_eq!(coordinator.current_generation(), 2);
    }

    #[tokio::test]
    async fn coordinator_returns_error_then_idle_snapshots() {
        let coordinator = RuntimeCoordinator::default();
        coordinator
            .start(AppMode::Macro, StopPolicy::UntilStopped)
            .await
            .unwrap();

        let (error_snapshot, idle_snapshot) = coordinator
            .fail_execution("playback input disconnected")
            .await
            .unwrap();
        assert_eq!(error_snapshot.phase, RuntimePhase::Error);
        assert_eq!(
            error_snapshot.last_error.as_deref(),
            Some("playback input disconnected")
        );
        assert_eq!(idle_snapshot.phase, RuntimePhase::Idle);
        assert_eq!(
            idle_snapshot.last_error.as_deref(),
            Some("playback input disconnected")
        );
        assert_eq!(coordinator.snapshot().await, idle_snapshot);
    }

    #[tokio::test]
    async fn coordinator_records_only_in_macro_mode_and_advances_generation_on_success() {
        let wrong_mode = RuntimeCoordinator::default();
        let wrong_mode_snapshot = wrong_mode.snapshot().await;
        assert_eq!(
            wrong_mode.begin_recording().await,
            Err(RuntimeError::InvalidTransition)
        );
        assert_eq!(wrong_mode.current_generation(), 0);
        assert_eq!(wrong_mode.snapshot().await, wrong_mode_snapshot);

        let coordinator = RuntimeCoordinator::default();
        coordinator.set_selected_mode(AppMode::Macro).await.unwrap();
        assert_eq!(coordinator.current_generation(), 0);

        let recording = coordinator.begin_recording().await.unwrap();
        assert_eq!(recording.phase, RuntimePhase::Recording);
        assert_eq!(coordinator.current_generation(), 1);

        let recording_snapshot = coordinator.snapshot().await;
        assert_eq!(
            coordinator.begin_recording().await,
            Err(RuntimeError::Busy(RuntimePhase::Recording))
        );
        assert_eq!(coordinator.current_generation(), 1);
        assert_eq!(coordinator.snapshot().await, recording_snapshot);
    }

    #[tokio::test]
    async fn successful_recording_clears_stale_error_and_cycle_count() {
        let coordinator = RuntimeCoordinator::default();
        coordinator
            .start(AppMode::Mouse, StopPolicy::UntilStopped)
            .await
            .unwrap();
        coordinator.record_cycle_completed().await.unwrap();
        coordinator
            .fail_execution("input disconnected")
            .await
            .unwrap();
        assert_eq!(
            coordinator.snapshot().await.last_error.as_deref(),
            Some("input disconnected")
        );

        coordinator.set_selected_mode(AppMode::Macro).await.unwrap();
        let recording = coordinator.begin_recording().await.unwrap();
        assert_eq!(recording.phase, RuntimePhase::Recording);
        assert_eq!(recording.last_error, None);
        assert_eq!(recording.completed_cycles, 0);
        assert_eq!(coordinator.snapshot().await.last_error, None);
    }

    #[tokio::test]
    async fn record_cycle_completed_is_constrained_to_active_execution_phases() {
        let coordinator = RuntimeCoordinator::default();
        assert_eq!(
            coordinator.record_cycle_completed().await,
            Err(RuntimeError::InvalidTransition)
        );

        let started = coordinator
            .start(AppMode::Keyboard, StopPolicy::UntilStopped)
            .await
            .unwrap();
        assert_eq!(started.completed_cycles, 0);

        let after_one = coordinator.record_cycle_completed().await.unwrap();
        assert_eq!(after_one.completed_cycles, 1);
        let after_two = coordinator.record_cycle_completed().await.unwrap();
        assert_eq!(after_two.completed_cycles, 2);

        coordinator.begin_stop().await.unwrap();
        assert_eq!(
            coordinator.record_cycle_completed().await,
            Err(RuntimeError::InvalidTransition)
        );
        assert_eq!(coordinator.snapshot().await.completed_cycles, 2);
    }

    #[tokio::test]
    async fn app_state_uses_an_idle_runtime_coordinator() {
        let app_state = crate::engine::state::AppState::default();

        assert_eq!(
            app_state.runtime_coordinator.snapshot().await.phase,
            RuntimePhase::Idle
        );
    }
}
