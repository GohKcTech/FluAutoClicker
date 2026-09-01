#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InputMouseButton {
    Left,
    Middle,
    Right,
    Front,
    Back,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InputToken {
    Key(String),
    Mouse(InputMouseButton),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputError(pub String);

impl InputError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

pub trait InputSink: Send {
    fn key_down(&mut self, key: &str) -> Result<(), InputError>;
    fn key_up(&mut self, key: &str) -> Result<(), InputError>;
    fn mouse_down(&mut self, button: InputMouseButton) -> Result<(), InputError>;
    fn mouse_up(&mut self, button: InputMouseButton) -> Result<(), InputError>;
    /// Emits text through the platform text-input path. Text has no held
    /// state, so it deliberately does not participate in session cleanup.
    fn text(&mut self, _text: &str) -> Result<(), InputError> {
        Err(InputError::new(
            "text input is not supported by this input sink",
        ))
    }
    fn position(&mut self) -> Result<(i32, i32), InputError> {
        Err(InputError::new(
            "cursor position is not supported by this input sink",
        ))
    }
    fn move_to(&mut self, x: i32, y: i32) -> Result<(), InputError>;
    fn scroll(&mut self, clicks: i32) -> Result<(), InputError>;
}

fn press_token(sink: &mut dyn InputSink, token: &InputToken) -> Result<(), InputError> {
    match token {
        InputToken::Key(key) => sink.key_down(key),
        InputToken::Mouse(button) => sink.mouse_down(button.clone()),
    }
}

fn release_token(sink: &mut dyn InputSink, token: &InputToken) -> Result<(), InputError> {
    match token {
        InputToken::Key(key) => sink.key_up(key),
        InputToken::Mouse(button) => sink.mouse_up(button.clone()),
    }
}

/// Tracks inputs that this process has successfully pressed, in press order,
/// so `release_all` can release only what it owns, in reverse order.
#[derive(Default)]
pub struct HeldInputs {
    held: Vec<InputToken>,
}

impl HeldInputs {
    pub fn press(&mut self, sink: &mut dyn InputSink, token: InputToken) -> Result<(), InputError> {
        if self.held.contains(&token) {
            return Ok(());
        }

        press_token(sink, &token)?;
        self.held.push(token);
        Ok(())
    }

    pub fn release(
        &mut self,
        sink: &mut dyn InputSink,
        token: &InputToken,
    ) -> Result<(), InputError> {
        let Some(position) = self.held.iter().rposition(|held| held == token) else {
            return Err(InputError::new("release of a token that is not held"));
        };

        let result = release_token(sink, token);
        if result.is_ok() {
            self.held.remove(position);
        }
        result
    }

    /// Releases every held token in reverse press order. A failed release
    /// does not stop the remaining releases; the token is still considered
    /// released, since this process cannot retry a lost physical state.
    pub fn release_all(&mut self, sink: &mut dyn InputSink) {
        while let Some(token) = self.held.pop() {
            let _ = release_token(sink, &token);
        }
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.held.is_empty()
    }
}

/// Owns an `InputSink` alongside the inputs this process has pressed through
/// it. `Drop` always releases everything still held, including across an
/// unwinding panic, so a caller never needs to remember cleanup on every
/// return path.
pub struct InputSession<S: InputSink> {
    sink: S,
    held: HeldInputs,
}

impl<S: InputSink> InputSession<S> {
    pub fn new(sink: S) -> Self {
        Self {
            sink,
            held: HeldInputs::default(),
        }
    }

    pub fn press(&mut self, token: InputToken) -> Result<(), InputError> {
        self.held.press(&mut self.sink, token)
    }

    pub fn release(&mut self, token: &InputToken) -> Result<(), InputError> {
        self.held.release(&mut self.sink, token)
    }

    pub fn text(&mut self, text: &str) -> Result<(), InputError> {
        self.sink.text(text)
    }

    pub fn move_to(&mut self, x: i32, y: i32) -> Result<(), InputError> {
        self.sink.move_to(x, y)
    }

    pub fn position(&mut self) -> Result<(i32, i32), InputError> {
        self.sink.position()
    }

    pub fn scroll(&mut self, clicks: i32) -> Result<(), InputError> {
        self.sink.scroll(clicks)
    }
}

impl<S: InputSink> Drop for InputSession<S> {
    fn drop(&mut self) {
        self.held.release_all(&mut self.sink);
    }
}

/// Test-only `InputSink` shared across engine test modules (clicker,
/// keyboard_clicker, macro playback): records every call behind a shared
/// lock so a test can hold a clone for assertions while another clone runs
/// inside the executor under test, and can be told to fail on a specific
/// call attempt to exercise cleanup paths.
#[cfg(test)]
pub(crate) mod test_support {
    use super::{InputError, InputMouseButton, InputSink, InputToken};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) enum InputCall {
        Down(InputToken),
        Up(InputToken),
        Text(String),
        MoveTo(i32, i32),
        Scroll(i32),
    }

    #[derive(Default)]
    struct SharedFakeSinkState {
        calls: Vec<InputCall>,
        position: (i32, i32),
        attempts: usize,
        fail_on: Option<usize>,
    }

    #[derive(Clone, Default)]
    pub(crate) struct SharedFakeSink {
        state: Arc<Mutex<SharedFakeSinkState>>,
    }

    impl SharedFakeSink {
        pub(crate) fn fail_on_attempt(&self, attempt: usize) {
            self.state.lock().unwrap().fail_on = Some(attempt);
        }

        pub(crate) fn calls(&self) -> Vec<InputCall> {
            self.state.lock().unwrap().calls.clone()
        }

        /// True when every recorded Down for a token has a matching Up,
        /// i.e. nothing is left physically held.
        pub(crate) fn no_inputs_held(&self) -> bool {
            let calls = self.state.lock().unwrap();
            let mut net: std::collections::HashMap<InputToken, i32> =
                std::collections::HashMap::new();
            for call in &calls.calls {
                match call {
                    InputCall::Down(token) => *net.entry(token.clone()).or_default() += 1,
                    InputCall::Up(token) => *net.entry(token.clone()).or_default() -= 1,
                    InputCall::Text(_) | InputCall::MoveTo(..) | InputCall::Scroll(..) => {}
                }
            }
            net.values().all(|count| *count == 0)
        }

        /// Number of complete Down-then-Up cycles recorded for `token`.
        pub(crate) fn complete_cycles(&self, token: &InputToken) -> usize {
            self.state
                .lock()
                .unwrap()
                .calls
                .iter()
                .filter(|call| matches!(call, InputCall::Up(t) if t == token))
                .count()
        }

        fn record(&self, call: InputCall) -> Result<(), InputError> {
            let mut state = self.state.lock().unwrap();
            let attempt = state.attempts;
            state.attempts += 1;
            if state.fail_on == Some(attempt) {
                return Err(InputError::new("forced failure"));
            }
            state.calls.push(call);
            Ok(())
        }
    }

    impl InputSink for SharedFakeSink {
        fn key_down(&mut self, key: &str) -> Result<(), InputError> {
            self.record(InputCall::Down(InputToken::Key(key.to_string())))
        }

        fn key_up(&mut self, key: &str) -> Result<(), InputError> {
            self.record(InputCall::Up(InputToken::Key(key.to_string())))
        }

        fn mouse_down(&mut self, button: InputMouseButton) -> Result<(), InputError> {
            self.record(InputCall::Down(InputToken::Mouse(button)))
        }

        fn mouse_up(&mut self, button: InputMouseButton) -> Result<(), InputError> {
            self.record(InputCall::Up(InputToken::Mouse(button)))
        }

        fn text(&mut self, text: &str) -> Result<(), InputError> {
            self.record(InputCall::Text(text.to_string()))
        }

        fn position(&mut self) -> Result<(i32, i32), InputError> {
            Ok(self.state.lock().unwrap().position)
        }

        fn move_to(&mut self, x: i32, y: i32) -> Result<(), InputError> {
            self.record(InputCall::MoveTo(x, y))?;
            self.state.lock().unwrap().position = (x, y);
            Ok(())
        }

        fn scroll(&mut self, clicks: i32) -> Result<(), InputError> {
            self.record(InputCall::Scroll(clicks))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum InputCall {
        Down(InputToken),
        Up(InputToken),
    }

    #[derive(Default)]
    struct FakeSink {
        calls: Vec<InputCall>,
        attempts: usize,
        fail_on: Option<usize>,
    }

    impl FakeSink {
        fn record_down(&mut self, token: InputToken) -> Result<(), InputError> {
            self.record(InputCall::Down(token))
        }

        fn record_up(&mut self, token: InputToken) -> Result<(), InputError> {
            self.record(InputCall::Up(token))
        }

        fn record(&mut self, call: InputCall) -> Result<(), InputError> {
            let attempt = self.attempts;
            self.attempts += 1;
            if self.fail_on == Some(attempt) {
                return Err(InputError::new("forced failure"));
            }
            self.calls.push(call);
            Ok(())
        }
    }

    impl InputSink for FakeSink {
        fn key_down(&mut self, key: &str) -> Result<(), InputError> {
            self.record_down(InputToken::Key(key.to_string()))
        }

        fn key_up(&mut self, key: &str) -> Result<(), InputError> {
            self.record_up(InputToken::Key(key.to_string()))
        }

        fn mouse_down(&mut self, button: InputMouseButton) -> Result<(), InputError> {
            self.record_down(InputToken::Mouse(button))
        }

        fn mouse_up(&mut self, button: InputMouseButton) -> Result<(), InputError> {
            self.record_up(InputToken::Mouse(button))
        }

        fn move_to(&mut self, _x: i32, _y: i32) -> Result<(), InputError> {
            Ok(())
        }

        fn scroll(&mut self, _clicks: i32) -> Result<(), InputError> {
            Ok(())
        }
    }

    fn key(name: &str) -> InputToken {
        InputToken::Key(name.to_string())
    }

    #[test]
    fn releases_only_successfully_held_inputs_in_reverse_order() {
        let mut sink = FakeSink::default();
        let mut held = HeldInputs::default();
        held.press(&mut sink, key("ControlLeft")).unwrap();
        held.press(&mut sink, key("A")).unwrap();
        held.release_all(&mut sink);

        assert_eq!(
            sink.calls,
            vec![
                InputCall::Down(key("ControlLeft")),
                InputCall::Down(key("A")),
                InputCall::Up(key("A")),
                InputCall::Up(key("ControlLeft")),
            ]
        );
        assert!(held.is_empty());
    }

    #[test]
    fn duplicate_down_is_a_no_op_and_does_not_double_press() {
        let mut sink = FakeSink::default();
        let mut held = HeldInputs::default();
        held.press(&mut sink, key("A")).unwrap();
        held.press(&mut sink, key("A")).unwrap();

        assert_eq!(sink.calls, vec![InputCall::Down(key("A"))]);

        held.release_all(&mut sink);
        assert_eq!(
            sink.calls,
            vec![InputCall::Down(key("A")), InputCall::Up(key("A"))]
        );
    }

    #[test]
    fn unmatched_release_is_rejected_without_calling_the_sink() {
        let mut sink = FakeSink::default();
        let mut held = HeldInputs::default();

        assert_eq!(
            held.release(&mut sink, &key("A")),
            Err(InputError::new("release of a token that is not held"))
        );
        assert!(sink.calls.is_empty());
    }

    #[test]
    fn a_failed_down_does_not_enter_the_held_list() {
        let mut sink = FakeSink {
            fail_on: Some(0),
            ..Default::default()
        };
        let mut held = HeldInputs::default();

        assert_eq!(
            held.press(&mut sink, key("A")),
            Err(InputError::new("forced failure"))
        );
        assert!(held.is_empty());

        held.release_all(&mut sink);
        assert!(sink.calls.is_empty());
    }

    #[test]
    fn release_all_continues_after_one_up_error() {
        let mut sink = FakeSink::default();
        let mut held = HeldInputs::default();
        held.press(&mut sink, key("ControlLeft")).unwrap();
        held.press(&mut sink, key("A")).unwrap();

        // Calls so far: Down(Ctrl)=0, Down(A)=1. The next call, Up(A), is index 2.
        sink.fail_on = Some(2);
        held.release_all(&mut sink);

        assert_eq!(
            sink.calls,
            vec![
                InputCall::Down(key("ControlLeft")),
                InputCall::Down(key("A")),
                InputCall::Up(key("ControlLeft")),
            ]
        );
        assert!(held.is_empty());
    }

    #[test]
    fn failed_explicit_up_is_retried_during_session_cleanup() {
        let mut sink = FakeSink::default();
        let mut held = HeldInputs::default();
        held.press(&mut sink, key("A")).unwrap();

        // The next Up fails. The token must remain owned so cleanup retries it.
        sink.fail_on = Some(1);
        assert_eq!(
            held.release(&mut sink, &key("A")),
            Err(InputError::new("forced failure"))
        );
        assert!(!held.is_empty());

        held.release_all(&mut sink);
        assert_eq!(
            sink.calls,
            vec![InputCall::Down(key("A")), InputCall::Up(key("A"))]
        );
        assert!(held.is_empty());
    }

    struct PanicOnSecondDownSink {
        calls: Arc<Mutex<Vec<InputCall>>>,
    }

    impl InputSink for PanicOnSecondDownSink {
        fn key_down(&mut self, key: &str) -> Result<(), InputError> {
            if self.calls.lock().unwrap().len() == 1 {
                panic!("simulated input failure");
            }
            self.calls
                .lock()
                .unwrap()
                .push(InputCall::Down(InputToken::Key(key.to_string())));
            Ok(())
        }

        fn key_up(&mut self, key: &str) -> Result<(), InputError> {
            self.calls
                .lock()
                .unwrap()
                .push(InputCall::Up(InputToken::Key(key.to_string())));
            Ok(())
        }

        fn mouse_down(&mut self, _button: InputMouseButton) -> Result<(), InputError> {
            Ok(())
        }

        fn mouse_up(&mut self, _button: InputMouseButton) -> Result<(), InputError> {
            Ok(())
        }

        fn move_to(&mut self, _x: i32, _y: i32) -> Result<(), InputError> {
            Ok(())
        }

        fn scroll(&mut self, _clicks: i32) -> Result<(), InputError> {
            Ok(())
        }
    }

    #[test]
    fn drop_releases_held_inputs_even_after_a_panic() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let session_calls = calls.clone();

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut session = InputSession::new(PanicOnSecondDownSink {
                calls: session_calls,
            });
            session.press(key("A")).unwrap();
            // This press panics inside the sink before the session can
            // register it as held.
            let _ = session.press(key("B"));
        }));

        assert!(result.is_err());
        assert_eq!(
            *calls.lock().unwrap(),
            vec![InputCall::Down(key("A")), InputCall::Up(key("A"))]
        );
    }
}
