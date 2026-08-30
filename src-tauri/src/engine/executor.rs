use std::time::Duration;

use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

/// Cooperative cancellation signal shared between a runtime coordinator and
/// the executor loop currently running. Cancelling is event-driven: waiters
/// wake immediately instead of polling.
#[derive(Clone, Default)]
pub struct Cancellation(CancellationToken);

impl Cancellation {
    pub fn new() -> Self {
        Self(CancellationToken::new())
    }

    pub fn cancel(&self) {
        self.0.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitOutcome {
    Completed,
    Cancelled,
    DeadlineReached,
}

/// Waits for `duration`, waking early on cancellation or on reaching
/// `deadline`, whichever comes first. Uses `tokio::select!` rather than a
/// polling loop so cancellation is observed immediately, even under paused
/// simulated time in tests.
pub async fn interruptible_wait(
    duration: Duration,
    cancel: Cancellation,
    deadline: Option<Instant>,
) -> WaitOutcome {
    let sleep = tokio::time::sleep(duration);
    tokio::pin!(sleep);

    match deadline {
        Some(deadline) => {
            let deadline_sleep = tokio::time::sleep_until(deadline);
            tokio::pin!(deadline_sleep);

            tokio::select! {
                biased;
                _ = cancel.0.cancelled() => WaitOutcome::Cancelled,
                _ = &mut deadline_sleep => WaitOutcome::DeadlineReached,
                _ = &mut sleep => WaitOutcome::Completed,
            }
        }
        None => {
            tokio::select! {
                biased;
                _ = cancel.0.cancelled() => WaitOutcome::Cancelled,
                _ = &mut sleep => WaitOutcome::Completed,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn long_wait_returns_on_cancel() {
        let cancel = Cancellation::new();
        let waiter = tokio::spawn(interruptible_wait(
            Duration::from_secs(60),
            cancel.clone(),
            None,
        ));
        cancel.cancel();
        assert_eq!(waiter.await.unwrap(), WaitOutcome::Cancelled);
    }

    #[tokio::test(start_paused = true)]
    async fn wait_completes_normally_when_not_cancelled() {
        let cancel = Cancellation::new();
        let outcome = interruptible_wait(Duration::from_millis(10), cancel, None).await;
        assert_eq!(outcome, WaitOutcome::Completed);
    }

    #[tokio::test(start_paused = true)]
    async fn deadline_wins_when_it_is_sooner_than_the_requested_duration() {
        let cancel = Cancellation::new();
        let deadline = Instant::now() + Duration::from_millis(5);
        let outcome = interruptible_wait(Duration::from_secs(60), cancel, Some(deadline)).await;
        assert_eq!(outcome, WaitOutcome::DeadlineReached);
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_wins_over_an_already_elapsed_deadline() {
        let cancel = Cancellation::new();
        cancel.cancel();
        let deadline = Instant::now() + Duration::from_millis(5);
        let outcome = interruptible_wait(Duration::from_secs(60), cancel, Some(deadline)).await;
        assert_eq!(outcome, WaitOutcome::Cancelled);
    }
}
