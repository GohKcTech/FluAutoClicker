# My Window Auto Click Windows Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the pinned FluAutoClicker `next` baseline into a Windows-only personal auto-clicker with mutually exclusive mouse, keyboard, and recorded-macro modes; configurable F8 start/stop; fixed F12 emergency stop; count, duration, and manual stopping; and safe, versioned local presets.

**Architecture:** Add one Rust runtime coordinator that owns the top-level state, cancellation, deadline, and held-input cleanup. Mouse, keyboard, and macro playback become executors behind an injectable `InputSink`, while recording and preset persistence remain separate pure-data modules. The TypeScript UI becomes a thin projection of backend state and commands; it never decides whether two executors may run concurrently.

**Tech Stack:** Rust 2021, Tauri 2, Tokio, Enigo/Rdev, Serde/serde_json, TypeScript 5.6, Vite 6, Node smoke tests, Windows MSVC toolchain.

**Spec:** `docs/superpowers/specs/2026-08-30-windows-auto-click-design.md`

## Global Constraints

- Keep upstream MIT copyright and license notices.
- Target Windows 10/11 only. Do not retain Linux `/dev/uinput`, AppImage, DEB, RPM, macOS, mobile, or CLI execution paths.
- Do not add anti-cheat bypasses, elevation, driver installation, network services, telemetry, cloud sync, or parallel macro execution.
- F12 is fixed, non-configurable, registered independently, and remains active while ordinary hotkeys are suspended for capture.
- Only release inputs that this process successfully pressed through `InputSink`; never release arbitrary physical user input.
- Every behavior change follows red-green-refactor: add one failing focused test, observe the expected failure, implement the smallest change, rerun the focused test, then run the broader gate.
- Do not claim Rust or release-build success until the MSVC toolchain and Microsoft C++ Build Tools are present and the exact command exits 0.
- Do not run real input injection smoke tests without first telling the user what keys/clicks will be generated and obtaining confirmation.
- Preserve upstream-readable `app_config.json`, profile JSON, and `macro.json` until migration tests prove compatibility.
- Commit only task-local files. Run `git diff --check` before every commit.

## Planned File Structure and Responsibilities

```text
src-tauri/src/
├── engine/
│   ├── runtime.rs              # AppMode, RuntimePhase, StopPolicy, transitions, cancellation
│   ├── input.rs                # InputSink, InputToken, HeldInputs, release-all guard
│   ├── executor.rs             # shared deadline/count loop and interruptible waits
│   ├── clicker.rs              # mouse cycle construction and Windows execution adapter
│   ├── keyboard_clicker.rs     # keyboard cycle construction and Windows execution adapter
│   ├── macro_engine/
│   │   ├── playback.rs         # macro action interpreter using executor/input abstractions
│   │   ├── recording.rs        # monotonic event collector and hook adapter
│   │   ├── storage.rs          # legacy macro reader only during migration
│   │   ├── state.rs            # in-memory macro state
│   │   └── types.rs            # serializable macro actions/options
│   ├── preset.rs               # v3 preset schema, validation, migration
│   ├── atomic_store.rs         # temp write, flush, Windows replace, backup recovery
│   ├── config_store.rs         # app/preset paths and repository operations
│   ├── state.rs                # AppState composition; no duplicate running flags
│   └── mod.rs
├── commands.rs                 # validated Tauri command boundary
├── lib.rs                      # app setup, hotkeys, tray, clean shutdown
└── main.rs
src/
├── index.html                  # three-mode UI and settings drawers
├── scripts/
│   ├── app/bootstrap.ts        # initialize retained modules only
│   ├── app/runtime.ts          # runtime-status projection and controls lock
│   ├── app/start-stop.ts       # start/stop command calls
│   ├── hotkeys.ts              # configurable hotkeys plus fixed F12 display
│   ├── settings-persistence.ts # v3 config DTO mapping
│   ├── profiles.ts             # preset CRUD/import/export UI
│   └── macro/*                 # recording/action editor/repeat controls
├── lang/en.json                # complete fallback keys
└── lang/zh.json                # default language keys
scripts/
├── smoke-product.mjs           # removed-feature and branding assertions
├── smoke-i18n.mjs              # exactly zh/en and identical keys
└── smoke-static.mjs            # required DOM/command wiring
```

---

### Task 1: Establish a verifiable Windows MSVC baseline

**Files:**

- Create: `rust-toolchain.toml`
- Modify: `pnpm-workspace.yaml`
- Modify: `README.md`
- Test: command-line baseline only

- [ ] **Step 1: Record the current expected toolchain failure**

Run:

```powershell
rustup show
npm.cmd run test:rust
```

Expected before setup: the active host is `stable-x86_64-pc-windows-gnu`, and Rust test linking fails because `dlltool.exe` is missing. Save the exact terminal output in the task notes; do not mark the test green.

- [ ] **Step 2: Add the repository toolchain pin**

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable-x86_64-pc-windows-msvc"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

Add a README prerequisites section naming Rust MSVC, Microsoft C++ Build Tools with “Desktop development with C++”, and WebView2. Do not install Build Tools until the user separately approves that system change.

Keep the dependency-build approval explicit and reviewable:

```yaml
allowBuilds:
  esbuild: true
```

- [ ] **Step 3: Verify or stop at the explicit environment gate**

Obtain explicit user approval before installing the repository-pinned Rust toolchain. If Microsoft C++ Build Tools are also missing, request separate approval for that installer and its “Desktop development with C++” workload.

Run:

```powershell
rustup toolchain install stable-x86_64-pc-windows-msvc --profile minimal --component rustfmt --component clippy
rustup show active-toolchain
npm.cmd run test:rust
npm.cmd run lint
npm.cmd run build
```

Expected: active toolchain ends in `windows-msvc`; all three project commands exit 0. If MSVC compilation reports missing `link.exe`, stop this task and request approval to install Microsoft C++ Build Tools. Do not continue implementation on an unverified Rust baseline.

- [ ] **Step 4: Commit the baseline pin**

```powershell
git diff --check
git add rust-toolchain.toml pnpm-workspace.yaml README.md
git commit -m "build: pin Windows MSVC Rust toolchain"
```

---

### Task 2: Introduce the single runtime state machine

**Files:**

- Create: `src-tauri/src/engine/runtime.rs`
- Modify: `src-tauri/src/engine/mod.rs`
- Modify: `src-tauri/src/engine/state.rs`
- Test: `src-tauri/src/engine/runtime.rs`

- [ ] **Step 1: Write failing transition and policy tests**

Add tests covering: startup is always idle even when the selected mode is restored; idle-to-mode start; rejection of a second start; recording/playback mutual exclusion; running-to-stopping-to-idle; error cleanup returning to idle; count zero rejection; duration zero rejection.

```rust
#[test]
fn rejects_reentrant_start() {
    let mut runtime = RuntimeModel::default();
    runtime.start(AppMode::Mouse, StopPolicy::UntilStopped).unwrap();
    assert_eq!(
        runtime.start(AppMode::Keyboard, StopPolicy::RepeatCount(2)),
        Err(RuntimeError::Busy(RuntimePhase::RunningMouse))
    );
}

#[test]
fn validates_stop_policy() {
    assert_eq!(StopPolicy::RepeatCount(0).validate(), Err(RuntimeError::InvalidCount));
    assert_eq!(StopPolicy::DurationMs(0).validate(), Err(RuntimeError::InvalidDuration));
}
```

- [ ] **Step 2: Run the focused test and observe red**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::runtime::tests
```

Expected: compilation fails because `engine::runtime` and its types do not exist.

- [ ] **Step 3: Implement the pure model**

Use these exact public types:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppMode { Mouse, Keyboard, Macro }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePhase {
    Idle, Recording, RunningMouse, RunningKeyboard, PlayingMacro, Stopping, Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum StopPolicy { UntilStopped, RepeatCount(u32), DurationMs(u64) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    Busy(RuntimePhase), InvalidTransition, InvalidCount, InvalidDuration,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RuntimeSnapshot {
    pub phase: RuntimePhase,
    pub selected_mode: AppMode,
    pub completed_cycles: u64,
    pub last_error: Option<String>,
}
```

`RuntimeModel` owns `phase`, `selected_mode`, `completed_cycles`, and `last_error`. `RuntimeCoordinator` wraps it in `tokio::sync::Mutex` and owns the current cancellation generation. An execution error emits a cleaned `Error` snapshot and then an `Idle` snapshot that retains `last_error` until the next successful start. Remove `is_running`, `kb_is_running`, and macro player state as independent authorities; compatibility getters may derive booleans from `RuntimePhase` during migration.

- [ ] **Step 4: Verify focused and full Rust tests**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::runtime::tests
npm.cmd run test:rust
```

Expected: both commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src-tauri/src/engine/runtime.rs src-tauri/src/engine/mod.rs src-tauri/src/engine/state.rs
git commit -m "feat: add unified automation runtime state"
```

---

### Task 3: Add cancellable waits and owned-input cleanup

**Files:**

- Create: `src-tauri/src/engine/input.rs`
- Create: `src-tauri/src/engine/executor.rs`
- Modify: `src-tauri/src/engine/mod.rs`
- Modify: `src-tauri/Cargo.toml`
- Test: `src-tauri/src/engine/input.rs`
- Test: `src-tauri/src/engine/executor.rs`

- [ ] **Step 1: Write failing cleanup-order tests with a fake sink**

```rust
#[derive(Default)]
struct FakeSink { calls: Vec<InputCall>, fail_on: Option<usize> }

#[test]
fn releases_only_successfully_held_inputs_in_reverse_order() {
    let mut sink = FakeSink::default();
    let mut held = HeldInputs::default();
    held.press(&mut sink, InputToken::Key("ControlLeft".into())).unwrap();
    held.press(&mut sink, InputToken::Key("A".into())).unwrap();
    held.release_all(&mut sink);
    assert_eq!(sink.calls, vec![
        InputCall::Down(InputToken::Key("ControlLeft".into())),
        InputCall::Down(InputToken::Key("A".into())),
        InputCall::Up(InputToken::Key("A".into())),
        InputCall::Up(InputToken::Key("ControlLeft".into())),
    ]);
}
```

Also test duplicate `Down`, unmatched `Up`, a failed `Down` not entering the held list, and release continuing after one `Up` error.

- [ ] **Step 2: Write a paused-time cancellation test**

```rust
#[tokio::test(start_paused = true)]
async fn long_wait_returns_on_cancel() {
    let cancel = Cancellation::new();
    let waiter = tokio::spawn(interruptible_wait(
        Duration::from_secs(60), cancel.clone(), None,
    ));
    cancel.cancel();
    assert_eq!(waiter.await.unwrap(), WaitOutcome::Cancelled);
}
```

The test must complete after `cancel.cancel()` without advancing simulated time. This proves cancellation is event-driven rather than a polling loop and supports the requirement to enter release cleanup within 100 milliseconds under normal load.

- [ ] **Step 3: Observe red**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::input::tests
cargo test --manifest-path src-tauri/Cargo.toml engine::executor::tests
```

Expected: missing module/type errors.

- [ ] **Step 4: Implement the abstractions**

Add `tokio-util = { version = "0.7", features = ["rt"] }` and add `"test-util"` to the existing Tokio feature list so paused-time tests compile. Implement:

```rust
pub trait InputSink: Send {
    fn key_down(&mut self, key: &str) -> Result<(), InputError>;
    fn key_up(&mut self, key: &str) -> Result<(), InputError>;
    fn mouse_down(&mut self, button: InputMouseButton) -> Result<(), InputError>;
    fn mouse_up(&mut self, button: InputMouseButton) -> Result<(), InputError>;
    fn move_to(&mut self, x: i32, y: i32) -> Result<(), InputError>;
    fn scroll(&mut self, clicks: i32) -> Result<(), InputError>;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InputMouseButton { Left, Middle, Right, Front, Back }

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InputToken { Key(String), Mouse(InputMouseButton) }
```

Use `tokio_util::sync::CancellationToken` in `Cancellation`. Implement `interruptible_wait` with `tokio::select!` over token cancellation, optional deadline, and `tokio::time::sleep`. `HeldInputs` registers only after a successful down and removes only after a successful up. Every executor owns a cleanup guard and calls `release_all` on all return paths.

Wrap each sink in an `InputSession<S>` whose synchronous `Drop` implementation calls `release_all`. Add a `catch_unwind` test that panics after `Down` and proves the session still emits the matching `Up`; this is the last-resort panic path, while ordinary cancellation/errors return normally.

- [ ] **Step 5: Verify**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::input::tests
cargo test --manifest-path src-tauri/Cargo.toml engine::executor::tests
npm.cmd run test:rust
```

Expected: cleanup, cancellation, and full Rust suites exit 0.

- [ ] **Step 6: Commit**

```powershell
git diff --check
git add src-tauri/Cargo.toml src-tauri/src/engine/input.rs src-tauri/src/engine/executor.rs src-tauri/src/engine/mod.rs
git commit -m "feat: add cancellable execution and input cleanup"
```

---

### Task 4: Move mouse and keyboard repetition onto the shared executor

**Files:**

- Modify: `src-tauri/src/engine/clicker.rs`
- Modify: `src-tauri/src/engine/keyboard_clicker.rs`
- Modify: `src-tauri/src/engine/state.rs`
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/src/engine/clicker.rs`
- Test: `src-tauri/src/engine/keyboard_clicker.rs`

- [ ] **Step 1: Add failing behavior tests**

For each executor test exact count, hard deadline during a hold, cancellation during the interval, send failure cleanup, and modifier release order.

```rust
#[tokio::test(start_paused = true)]
async fn keyboard_count_means_complete_down_up_cycles() {
    let sink = SharedFakeSink::default();
    run_keyboard(test_keyboard("A"), StopPolicy::RepeatCount(3), sink.clone()).await.unwrap();
    assert_eq!(sink.complete_key_cycles("A"), 3);
}

#[tokio::test(start_paused = true)]
async fn mouse_duration_interrupts_a_long_hold() {
    let sink = SharedFakeSink::default();
    let run = tokio::spawn(run_mouse(
        test_mouse_hold(Duration::from_secs(30)),
        StopPolicy::DurationMs(1_000),
        sink.clone(),
    ));
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(run.await.unwrap().unwrap(), RunEnd::Deadline);
    assert!(sink.no_inputs_held());
}
```

- [ ] **Step 2: Observe red against current loops**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::clicker::tests
cargo test --manifest-path src-tauri/Cargo.toml engine::keyboard_clicker::tests
```

Expected: new deadline/cleanup assertions fail because existing loops use independent atomics and ordinary sleeps.

- [ ] **Step 3: Implement common cycle semantics**

Convert UI settings once at command entry into `MouseRunConfig` or `KeyboardRunConfig`, including a validated `StopPolicy`. Implement one complete mouse cycle as down/up; double click remains one outer cycle containing two down/up pairs. Implement one keyboard cycle as modifiers down in canonical order, key down/up, modifiers up in reverse order.

Map Windows injection failures into stable categories (`PermissionMismatch`, `UnsupportedKey`, `SendFailed`) so the UI can explain that a higher-integrity target may require launching the tool manually at the same integrity level; never trigger elevation automatically.

Route `toggle_clicker` and `toggle_keyboard_clicker` through:

```rust
pub async fn toggle_selected_mode(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<RuntimeSnapshot, String>
```

Return an error when phase is not `Idle` or when the keyboard target exactly equals the configured start/stop shortcut.

- [ ] **Step 4: Verify**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::clicker::tests
cargo test --manifest-path src-tauri/Cargo.toml engine::keyboard_clicker::tests
npm.cmd run test:rust
npm.cmd run lint
```

Expected: all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src-tauri/src/engine/clicker.rs src-tauri/src/engine/keyboard_clicker.rs src-tauri/src/engine/state.rs src-tauri/src/commands.rs
git commit -m "refactor: unify mouse and keyboard execution"
```

---

### Task 5: Make macro playback interruptible and cleanup-safe

**Files:**

- Modify: `src-tauri/src/engine/macro_engine/playback.rs`
- Modify: `src-tauri/src/engine/macro_engine/state.rs`
- Modify: `src-tauri/src/engine/macro_engine/types.rs`
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/src/engine/macro_engine/playback.rs`

- [ ] **Step 1: Add failing playback tests**

Cover finite iterations, mid-iteration duration cutoff, cancel during `Sleep`, cancel during keyboard/mouse `Hold`, injected send error, legacy `RawMove` playback, and release order.

```rust
#[tokio::test(start_paused = true)]
async fn deadline_cuts_off_macro_mid_iteration_and_releases_key() {
    let actions = vec![key_down("ShiftLeft"), sleep_ms(60_000), key_up("ShiftLeft")];
    let sink = SharedFakeSink::default();
    let run = spawn_macro(actions, StopPolicy::DurationMs(500), sink.clone());
    tokio::time::advance(Duration::from_millis(500)).await;
    assert_eq!(run.await.unwrap(), RunEnd::Deadline);
    assert!(sink.no_inputs_held());
}
```

- [ ] **Step 2: Observe red**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml macro_engine::playback::tests
```

Expected: at least the long sleep/hold cancellation tests time out or fail cleanup assertions.

- [ ] **Step 3: Refactor playback through `InputSink` and `interruptible_wait`**

Keep deserialization and execution for `RawMove`, but remove it immediately from the new-action creation APIs in `commands.rs`. Convert `MacroRepeatMode` to the common `StopPolicy` at the boundary. Increment a repeat count only after the final action completes. Return `RunEnd::{Completed, Cancelled, Deadline}` and let the runtime coordinator perform the single phase transition back to `Idle`.

- [ ] **Step 4: Verify**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml macro_engine::playback::tests
npm.cmd run test:rust
npm.cmd run lint
```

Expected: all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src-tauri/src/engine/macro_engine/playback.rs src-tauri/src/engine/macro_engine/state.rs src-tauri/src/engine/macro_engine/types.rs src-tauri/src/commands.rs
git commit -m "fix: make macro playback safely interruptible"
```

---

### Task 6: Correct macro recording timing and event filtering

**Files:**

- Modify: `src-tauri/src/engine/macro_engine/recording.rs`
- Modify: `src-tauri/src/engine/macro_engine/types.rs`
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/src/engine/macro_engine/recording.rs`

- [ ] **Step 1: Extract a pure recorder and write failing tests**

Define `RecordingEvent`, `RecordingClock`, and `MacroRecorder<C: RecordingClock>`. Use a fake monotonic clock in tests. Cover left/right modifier identity, key auto-repeat deduplication, mouse down/up, wheel, delay generation, filtered control hotkeys, and ignoring injected playback while phase is not `Recording`.

```rust
#[test]
fn repeated_key_down_is_deduplicated_but_up_is_kept() {
    let mut recorder = recorder_at(&[0, 10, 250]);
    recorder.push(key_down("KeyA"));
    recorder.push(key_down("KeyA"));
    recorder.push(key_up("KeyA"));
    assert_eq!(recorder.actions(), &[keyboard_down("KeyA"), sleep_ms(250), keyboard_up("KeyA")]);
}

#[test]
fn excludes_recording_hotkey_and_emergency_stop_edges() {
    let mut recorder = test_recorder();
    for event in shortcut_edges("Ctrl+Shift+R").into_iter().chain(shortcut_edges("F12")) {
        recorder.push(event);
    }
    assert!(recorder.actions().is_empty());
}
```

- [ ] **Step 2: Observe red**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml macro_engine::recording::tests
```

Expected: new monotonic recorder APIs are missing and current `SystemTime` behavior cannot satisfy tests.

- [ ] **Step 3: Implement monotonic recording**

Production `RecordingClock` wraps `std::time::Instant`. Emit `Sleep` from monotonic deltas when `record_delays` is true. Preserve all down/up ordering. Default `record_mouse_moves` to `Off`; expose only `Off` and `Smooth` for new settings. Keep `Raw` deserialization for migration but reject selecting it through `set_macro_recording_options`.

- [ ] **Step 4: Verify**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml macro_engine::recording::tests
npm.cmd run test:rust
```

Expected: all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src-tauri/src/engine/macro_engine/recording.rs src-tauri/src/engine/macro_engine/types.rs src-tauri/src/commands.rs
git commit -m "fix: record macros with monotonic event timing"
```

---

### Task 7: Enforce F8, recording-hotkey, and fixed F12 rules

**Files:**

- Modify: `src-tauri/src/engine/state.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src/scripts/hotkeys.ts`
- Test: `src-tauri/src/lib.rs`
- Test: `scripts/smoke-static.mjs`

- [ ] **Step 1: Add failing hotkey-policy tests**

```rust
#[test]
fn default_hotkeys_match_product_contract() {
    let keys = RuntimeHotkeys::default();
    assert_eq!(keys.toggle_start_stop, "F8");
    assert_eq!(keys.toggle_macro_recording, "Ctrl+Shift+R");
    assert_eq!(EMERGENCY_STOP_HOTKEY, "F12");
}

#[test]
fn rejects_collisions_and_preserves_old_bindings() {
    let old = RuntimeHotkeys::default();
    assert!(validate_hotkey_update(&old, HotkeyAction::ToggleRecording, "F8").is_err());
    assert!(validate_hotkey_update(&old, HotkeyAction::ToggleStartStop, "F12").is_err());
    assert_eq!(old.toggle_start_stop, "F8");
}
```

Extend static smoke to require a non-editable F12 row and configurable F8/recording rows.

- [ ] **Step 2: Observe red**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml hotkey
npm.cmd run smoke:static
```

Expected: Rust expects F8 but current default is F6; DOM lacks a fixed emergency-stop row.

- [ ] **Step 3: Implement registration and rollback behavior**

Register F12 separately from ordinary hotkeys. The global handler must process F12 before checking `hotkeys_suspended`. Its only action is `RuntimeCoordinator::stop(StopReason::Emergency)`. For an ordinary binding change: validate syntax and collision, register the candidate, unregister the old binding only after success, persist only after runtime registration succeeds, and restore the old binding on persistence failure.

Remove the press-duration dual behavior for F8. One `Pressed` event toggles start/stop; ignore `Released` and OS key-repeat events.

- [ ] **Step 4: Verify**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml hotkey
npm.cmd run smoke:static
npm.cmd run lint
npm.cmd run test:rust
```

Expected: all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src-tauri/src/engine/state.rs src-tauri/src/lib.rs src-tauri/src/commands.rs src/scripts/hotkeys.ts scripts/smoke-static.mjs
git commit -m "feat: add F8 control and fixed F12 emergency stop"
```

---

### Task 8: Define and validate the versioned preset schema

**Files:**

- Create: `src-tauri/src/engine/preset.rs`
- Modify: `src-tauri/src/engine/mod.rs`
- Modify: `src-tauri/src/engine/config_store.rs`
- Modify: `src-tauri/src/engine/macro_engine/state.rs`
- Test: `src-tauri/src/engine/preset.rs`

- [ ] **Step 1: Write failing schema, range, and migration tests**

Test a valid v3 round trip; stable ID/name/update timestamp; each mode’s stop policy; unknown action rejection; invalid coordinate/count/duration rejection; legacy app config/profile/macro migration; and fixed F12 not appearing in serialized data.

```rust
#[test]
fn preset_round_trip_preserves_all_three_modes() {
    let preset = Preset::sample();
    let json = serde_json::to_string(&preset).unwrap();
    let loaded = Preset::parse_and_validate(json.as_bytes()).unwrap();
    assert_eq!(loaded, preset);
    assert!(!json.contains("F12"));
}

#[test]
fn rejects_unknown_macro_action_without_partial_import() {
    let mut input = serde_json::to_value(Preset::sample()).unwrap();
    input["macro_preset"]["actions"] = serde_json::json!([{"type":"launch_process"}]);
    let bytes = serde_json::to_vec(&input).unwrap();
    assert!(matches!(Preset::parse_and_validate(&bytes), Err(PresetError::InvalidAction(_))));
}
```

- [ ] **Step 2: Observe red**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::preset::tests
```

Expected: missing `preset` module and types.

- [ ] **Step 3: Implement v3 data types**

Use these top-level fields:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub updated_at_unix_ms: u64,
    pub selected_mode: AppMode,
    pub mouse: MousePreset,
    pub keyboard: KeyboardPreset,
    pub macro_preset: MacroPreset,
    pub hotkeys: ConfigurableHotkeys,
}
```

Set `CURRENT_PRESET_VERSION = 3`. Add `uuid = { version = "1", features = ["v4", "serde"] }` for stable IDs. Validate names after trim, UUID syntax, timestamps, key names, coordinates within signed 32-bit range, CPS 1..=10,000, hold/delay 1..=3,600,000 ms, repeat count 1..=1,000,000, duration 1..=86,400,000 ms, macro speed 0.1..=100.0, and action count no greater than 1,000,000.

Parse legacy formats into separate `LegacyAppConfigV2`, `LegacyProfileV1`, and `LegacyMacroV2` DTOs, then convert to `Preset`; do not loosen v3 with broad `serde(default)` on required fields.

- [ ] **Step 4: Verify**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::preset::tests
npm.cmd run test:rust
```

Expected: all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src-tauri/Cargo.toml src-tauri/src/engine/preset.rs src-tauri/src/engine/mod.rs src-tauri/src/engine/config_store.rs src-tauri/src/engine/macro_engine/state.rs
git commit -m "feat: add validated versioned preset schema"
```

---

### Task 9: Add atomic preset writes, backup recovery, and bounded import

**Files:**

- Create: `src-tauri/src/engine/atomic_store.rs`
- Modify: `src-tauri/src/engine/config_store.rs`
- Modify: `src-tauri/src/engine/macro_engine/storage.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/Cargo.toml`
- Test: `src-tauri/src/engine/atomic_store.rs`
- Test: `src-tauri/src/engine/config_store.rs`

- [ ] **Step 1: Add failing filesystem tests using temporary directories**

Add `tempfile = "3"` under `[dev-dependencies]`. Test successful save, temp-file cleanup, previous version copied to `.bak`, simulated replace failure preserving the original, corrupt primary recovering from `.bak` without overwrite, and 50 MiB import acceptance versus `50 MiB + 1 byte` rejection before JSON parsing.

```rust
#[test]
fn failed_replace_keeps_last_valid_primary() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("preset.json");
    fs::write(&target, b"old-valid").unwrap();
    let ops = FailingReplaceOps::new();
    assert!(atomic_write_with(&ops, &target, b"new-valid").is_err());
    assert_eq!(fs::read(&target).unwrap(), b"old-valid");
}

#[test]
fn rejects_oversize_import_before_deserialization() {
    assert_eq!(validate_import_size(MAX_IMPORT_BYTES + 1), Err(StoreError::TooLarge));
}
```

- [ ] **Step 2: Observe red**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::atomic_store::tests
cargo test --manifest-path src-tauri/Cargo.toml engine::config_store::tests
```

Expected: new storage APIs are missing.

- [ ] **Step 3: Implement durable replacement**

Add `windows-sys = { version = "0.59", features = ["Win32_Foundation", "Win32_Storage_FileSystem"] }` to the Windows target dependencies. Write to a unique file in the target directory, `write_all`, and `sync_all`. When a primary exists, call `ReplaceFileW(target, replacement, backup)` so Windows atomically installs the flushed replacement and writes the old primary to `.bak`; when no primary exists, rename the flushed temp file into place. Remove only the owned temp file on failure.

Add:

```rust
pub const MAX_IMPORT_BYTES: u64 = 50 * 1024 * 1024;

pub trait FileOps {
    fn sync_write(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;
    fn replace(&self, replacement: &Path, target: &Path, backup: &Path) -> io::Result<()>;
}
```

Move all app, profile, macro, and backup writes through `atomic_write`. Commands import bytes or a selected path and check metadata size before reading. Do not accept a frontend-provided arbitrary export path for imports.

- [ ] **Step 4: Verify**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml engine::atomic_store::tests
cargo test --manifest-path src-tauri/Cargo.toml engine::config_store::tests
npm.cmd run test:rust
```

Expected: all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src-tauri/Cargo.toml src-tauri/src/engine/atomic_store.rs src-tauri/src/engine/config_store.rs src-tauri/src/engine/macro_engine/storage.rs src-tauri/src/commands.rs
git commit -m "feat: persist presets atomically with recovery"
```

---

### Task 10: Cut non-Windows and out-of-scope product features

**Files:**

- Delete: `src-tauri/src/engine/uinput.rs`
- Delete: `src-tauri/src/engine/keyboard_uinput.rs`
- Delete: `src-tauri/src/engine/jiggler.rs`
- Delete: `src/scripts/uinput-permissions.ts`
- Delete: `src/scripts/update-check.ts`
- Delete: `src/lang/de.json`
- Delete: `src/lang/es.json`
- Delete: `src/lang/fr.json`
- Delete: `src/lang/ja.json`
- Delete: `src/lang/ko.json`
- Delete: `src/lang/pt.json`
- Delete: `src/lang/ru.json`
- Modify: `src-tauri/src/engine/mod.rs`
- Modify: `src-tauri/src/engine/state.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src/scripts/app/bootstrap.ts`
- Modify: `src/scripts/drawer.ts`
- Modify: `src/scripts/settings-persistence.ts`
- Modify: `src/lang/index.ts`
- Modify: `src/index.html`
- Create: `scripts/smoke-product.mjs`
- Modify: `package.json`
- Test: `scripts/smoke-product.mjs`

- [ ] **Step 1: Write the failing product-scope smoke test**

The script recursively reads source/config files and fails when it finds retained product paths or registrations. Use explicit checks:

```js
const forbiddenFiles = [
  "src-tauri/src/engine/uinput.rs",
  "src-tauri/src/engine/keyboard_uinput.rs",
  "src-tauri/src/engine/jiggler.rs",
  "src/scripts/uinput-permissions.ts",
  "src/scripts/update-check.ts",
];
assert.deepEqual(forbiddenFiles.filter(existsSync), []);
assert.equal(cargoToml.includes("input-linux"), false);
assert.equal(cargoToml.includes("tauri-plugin-cli"), false);
assert.equal(tauriConfig.includes('"appimage"'), false);
assert.equal(indexHtml.includes('id="jiggler-btn"'), false);
assert.equal(indexHtml.includes('id="github-btn"'), false);
assert.equal(indexHtml.includes('id="autostart-trigger"'), false);
assert.equal(sourceText.includes("api.github.com"), false);
```

Add `"smoke:product": "node scripts/smoke-product.mjs"` and include it in `lint`.

- [ ] **Step 2: Observe red**

```powershell
npm.cmd run smoke:product
```

Expected: failures list Linux input, jiggler, CLI, updater, autostart, community/network, extra language, or non-Windows bundle remnants.

- [ ] **Step 3: Remove the full feature slices**

Delete commands, imports, state fields, tasks, event listeners, HTML, CSS selectors, translations, plugins, dependencies, and Tauri capabilities associated with the removed features. Keep `tauri-plugin-single-instance` only for single-instance enforcement, with no CLI action parsing. Set bundle targets to `["nsis"]` and retain only Windows icons. Remove shell/opener plugins if no retained call site remains.

Set the Cargo package name to `my-window-auto-click`, its Rust library name to `my_window_auto_click_lib`, and Tauri product metadata to `My Window Auto Click`; use the identifier `com.gohka.mywindowautoclick`. Keep upstream attribution in LICENSE/NOTICE, not in promotional UI.

- [ ] **Step 4: Verify removed symbols and builds**

```powershell
npm.cmd run smoke:product
rg -n "uinput|jiggler|OZone|Hyprland|prerelease|api\.github\.com|set_start_on_system_startup|tauri_plugin_cli" src src-tauri package.json scripts
npm.cmd run lint
npm.cmd run build
npm.cmd run test:rust
```

Expected: smoke, lint, build, and Rust tests exit 0. The `rg` command exits 1 with no matches except explicit forbidden-term assertions inside `scripts/smoke-product.mjs`; inspect those lines rather than treating exit 1 alone as proof.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add -A src src-tauri scripts package.json
git commit -m "refactor: trim project to Windows automation core"
```

---

### Task 11: Rebuild the three-mode UI as a runtime projection

**Files:**

- Modify: `src/index.html`
- Create: `src/scripts/app/runtime.ts`
- Modify: `src/scripts/app/bootstrap.ts`
- Modify: `src/scripts/app/start-stop.ts`
- Modify: `src/scripts/app/toggle-groups.ts`
- Modify: `src/scripts/mouse.ts`
- Modify: `src/scripts/keyboard.ts`
- Modify: `src/scripts/macro/config-ui.ts`
- Modify: `src/scripts/macro/recording.ts`
- Modify: `src/styles/styles.css`
- Modify: `src/styles/components/features.css`
- Modify: `src/styles/components/macro.css`
- Modify: `scripts/smoke-static.mjs`
- Test: `scripts/smoke-static.mjs`

- [ ] **Step 1: Extend static smoke with the required UI contract**

Assert exactly three mode tabs (`mouse`, `keyboard`, `macro`); each mode has `until_stopped`, `repeat_count`, and `duration` controls; a single main action button; a visible runtime status; record/stop controls; a fixed F12 hint; and no `Raw` movement option.

```js
for (const mode of ["mouse", "keyboard", "macro"]) {
  assert.match(html, new RegExp(`data-mode="${mode}"`));
  for (const policy of ["until_stopped", "repeat_count", "duration"]) {
    assert.match(html, new RegExp(`data-stop-policy="${policy}"`));
  }
}
assert.equal((html.match(/id="start-btn"/g) ?? []).length, 1);
assert.match(html, /data-fixed-hotkey="F12"/);
assert.doesNotMatch(html, /data-move-mode="raw"/i);
```

- [ ] **Step 2: Observe red**

```powershell
npm.cmd run smoke:static
```

Expected: missing unified stop-policy and fixed-hotkey DOM assertions.

- [ ] **Step 3: Implement the runtime projection**

Use one frontend type matching Rust serialization:

```ts
export type RuntimePhase =
  | "idle" | "recording" | "running_mouse" | "running_keyboard"
  | "playing_macro" | "stopping" | "error";

export type RuntimeSnapshot = {
  phase: RuntimePhase;
  selected_mode: "mouse" | "keyboard" | "macro";
  completed_cycles: number;
  last_error?: string;
};
```

`runtime.ts` listens to one `runtime-status-changed` event and applies the snapshot. Disable all mode switches and critical setting inputs whenever phase is not `idle`; leave the stop button enabled in running/recording states. `start-stop.ts` invokes `start_selected_mode` or `stop_runtime`, never mutates local running state optimistically. Render `stopping` until the backend emits `idle`.

- [ ] **Step 4: Verify**

```powershell
npm.cmd run smoke:static
npm.cmd run typecheck
npm.cmd run build
```

Expected: all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src/index.html src/scripts/app/runtime.ts src/scripts/app/bootstrap.ts src/scripts/app/start-stop.ts src/scripts/app/toggle-groups.ts src/scripts/mouse.ts src/scripts/keyboard.ts src/scripts/macro/config-ui.ts src/scripts/macro/recording.ts src/styles scripts/smoke-static.mjs
git commit -m "feat: simplify UI around three automation modes"
```

---

### Task 12: Make Chinese the default with English fallback

**Files:**

- Modify: `src/scripts/i18n.ts`
- Modify: `src/lang/index.ts`
- Modify: `src/lang/zh.json`
- Modify: `src/lang/en.json`
- Modify: `src/index.html`
- Modify: `scripts/smoke-i18n.mjs`
- Test: `scripts/smoke-i18n.mjs`

- [ ] **Step 1: Write failing locale-contract checks**

Require the language registry to contain exactly `zh` and `en`, every key to exist in both files, `zh` to be the default, `en` to be the missing-key fallback, and all visible new runtime/preset/error labels to use `data-i18n`.

- [ ] **Step 2: Observe red**

```powershell
npm.cmd run smoke:i18n
```

Expected: registry/default/removed-language or missing-key failures.

- [ ] **Step 3: Implement locale fallback**

```ts
export const DEFAULT_LANGUAGE = "zh";
export const FALLBACK_LANGUAGE = "en";
export const languages = { zh, en } as const;
```

If saved language is absent or unsupported, choose `zh`. For each missing Chinese key, render the English value. Add translations for all phases, stop policies, fixed F12 help, hotkey conflicts, permission mismatch, corrupted preset recovery, import validation, recording filters, and cleanup errors.

- [ ] **Step 4: Verify**

```powershell
npm.cmd run smoke:i18n
npm.cmd run typecheck
npm.cmd run build
```

Expected: exactly two locales with identical key sets; all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src/scripts/i18n.ts src/lang/index.ts src/lang/zh.json src/lang/en.json src/index.html scripts/smoke-i18n.mjs
git commit -m "feat: default UI to Chinese with English fallback"
```

---

### Task 13: Wire reliable preset management into the UI

**Files:**

- Modify: `src/scripts/settings-persistence.ts`
- Modify: `src/scripts/profiles.ts`
- Modify: `src/scripts/macro/backend.ts`
- Modify: `src/scripts/macro/state.ts`
- Modify: `src/index.html`
- Modify: `src-tauri/src/commands.rs`
- Modify: `scripts/smoke-static.mjs`
- Test: `src-tauri/src/commands.rs`
- Test: `scripts/smoke-static.mjs`

- [ ] **Step 1: Add failing command-boundary and UI smoke tests**

Rust tests cover create, duplicate, rename, delete, switch, reset-default, import, and export command inputs. They must reject deleting the default preset, mutation while runtime is non-idle, duplicate normalized names, over-size imports, and malformed v3 JSON. Static smoke requires visible controls for each operation.

```rust
#[tokio::test]
async fn default_preset_can_reset_but_not_delete() {
    let repo = TestPresetRepository::new();
    assert_eq!(repo.delete(DEFAULT_PRESET_ID).await, Err(PresetError::DefaultProtected));
    repo.reset_default().await.unwrap();
    assert_eq!(repo.load(DEFAULT_PRESET_ID).await.unwrap(), Preset::default_named());
}
```

- [ ] **Step 2: Observe red**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml preset_command
npm.cmd run smoke:static
```

Expected: missing v3 repository command APIs or controls.

- [ ] **Step 3: Implement one typed command surface**

Expose:

```rust
list_presets() -> Result<Vec<PresetSummary>, String>
load_preset(id: String) -> Result<Preset, String>
create_preset(name: String, copy_from: Option<String>) -> Result<Preset, String>
rename_preset(id: String, name: String) -> Result<PresetSummary, String>
delete_preset(id: String) -> Result<(), String>
reset_default_preset() -> Result<Preset, String>
import_preset(path: String) -> Result<Preset, String>
export_preset(id: String, path: String) -> Result<(), String>
```

After a successful preset switch, update backend state first, emit the new snapshot/config, and only then update the frontend selection. On save failure, show the backend error and retain the last persisted preset selection. Keep export as a user-selected destination; constrain import through the dialog selection and 50 MiB backend check.

- [ ] **Step 4: Verify**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml preset_command
npm.cmd run smoke:static
npm.cmd run lint
npm.cmd run build
npm.cmd run test:rust
```

Expected: all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src/scripts/settings-persistence.ts src/scripts/profiles.ts src/scripts/macro/backend.ts src/scripts/macro/state.ts src/index.html src-tauri/src/commands.rs scripts/smoke-static.mjs
git commit -m "feat: connect reliable preset management"
```

---

### Task 14: Guarantee cleanup on stop, error, tray exit, and window exit

**Files:**

- Modify: `src-tauri/src/engine/runtime.rs`
- Modify: `src-tauri/src/engine/input.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/src/engine/runtime.rs`
- Test: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add failing lifecycle tests**

Test UI stop, F8 stop, F12 stop, executor error, deadline, tray start/stop, tray quit, and window/application exit. Each test asserts: cancellation issued; phase reaches `Stopping`; owned inputs release; recording hook stops writing; phase reaches `Idle` or cleaned `Error`; exit continuation runs only after cleanup.

```rust
#[tokio::test]
async fn quit_waits_for_cleanup_before_exit_callback() {
    let harness = RuntimeHarness::with_held_key("ShiftLeft");
    harness.request_shutdown().await.unwrap();
    assert!(harness.sink().no_inputs_held());
    assert_eq!(harness.events(), [LifecycleEvent::Released("ShiftLeft"), LifecycleEvent::Exit]);
}
```

- [ ] **Step 2: Observe red**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml lifecycle
```

Expected: current direct `app.exit(0)` path precedes unified cleanup.

- [ ] **Step 3: Implement `shutdown_runtime` once**

All quit paths call one async function:

```rust
pub async fn shutdown_runtime(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    state.runtime.stop(StopReason::ApplicationExit).await?;
    state.runtime.wait_until_idle(Duration::from_secs(2)).await?;
    unregister_ordinary_hotkeys(app)?;
    unregister_emergency_hotkey(app)?;
    Ok(())
}
```

The tray menu contains `Show`, a runtime-derived `Start` or `Stop` item for the currently selected/running mode, and `Quit`. Tray start/stop calls the same coordinator commands as F8 and the main button. Tray quit and `quit_app` spawn cleanup, then call `app.exit(0)`. A real OS termination cannot be made async-safe; document that limitation, while all normal application-controlled exits use this path. Monitor executor `JoinHandle` results; on a panic, rely on `InputSession` drop cleanup, transition through cleaned `Error`, and emit the actionable error before returning to `Idle`.

- [ ] **Step 4: Verify**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml lifecycle
npm.cmd run test:rust
npm.cmd run lint
```

Expected: all commands exit 0.

- [ ] **Step 5: Commit**

```powershell
git diff --check
git add src-tauri/src/engine/runtime.rs src-tauri/src/engine/input.rs src-tauri/src/lib.rs src-tauri/src/commands.rs
git commit -m "fix: clean up owned inputs on every normal exit"
```

---

### Task 15: Final branding, documentation, and automated verification

**Files:**

- Modify: `README.md`
- Create: `NOTICE.md`
- Verify unchanged: `LICENSE`
- Modify: `package.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src/index.html`
- Modify: `scripts/smoke-product.mjs`
- Test: all automated checks

- [ ] **Step 1: Make product/attribution assertions fail before final edits**

Extend `smoke-product.mjs` to require the product name in package, Cargo, Tauri config, title, and tray strings; NSIS as the only bundle target; no external runtime `https://` URL or network `fetch` call in `src/` (the loopback Vite `devUrl` is allowed); and upstream attribution in `NOTICE.md`.

- [ ] **Step 2: Observe red**

```powershell
npm.cmd run smoke:product
```

Expected: any remaining name, bundle, URL, or notice mismatch is reported.

- [ ] **Step 3: Finish product metadata and usage documentation**

README must document the three modes, F8/F12 behavior, recording exclusions, preset location/backup, Windows permission limitation, offline behavior, build prerequisites, development commands, and controlled smoke procedure. `NOTICE.md` names the FluAutoClicker upstream repository, pinned source commit, and MIT basis. Keep the original MIT license text and copyright.

- [ ] **Step 4: Run the complete automated gate from a clean terminal**

```powershell
git status --short
pnpm.cmd install --frozen-lockfile
npm.cmd run lint
npm.cmd run build
npm.cmd run test:rust
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm.cmd run tauri build -- --bundles nsis
git diff --check
```

Expected:

- only intentional tracked changes are present before the final commit;
- frozen install exits 0 without modifying the lockfile;
- lint, frontend build, Rust tests, Clippy, and NSIS release build all exit 0;
- the installer appears under `src-tauri/target/release/bundle/nsis/`;
- `git diff --check` prints nothing and exits 0.

- [ ] **Step 5: Commit the final automated state**

```powershell
git add README.md NOTICE.md package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json src/index.html scripts/smoke-product.mjs pnpm-lock.yaml
git commit -m "docs: finalize Windows auto-clicker release"
```

---

### Task 16: Perform user-approved controlled input smoke and package handoff

**Files:**

- Create: `docs/verification/2026-08-30-controlled-input-smoke.md`
- Test: manual Windows behavior only after explicit approval

- [ ] **Step 1: Present the exact smoke sequence and obtain approval**

Tell the user the application will target a disposable Notepad window and will generate: three left clicks at a safe coordinate; three `A` key cycles; recorded `Ctrl+Shift+X`, one right click, and one wheel step; an F8 stop during a five-second wait; and an F12 stop during a held `Shift`. Do not begin until the user confirms.

- [ ] **Step 2: Prepare a safe target**

Open a new empty Notepad document, position it away from destructive controls, ensure no password field or privileged window is focused, and start the built release binary without elevation.

- [ ] **Step 3: Execute and record each acceptance result**

Record pass/fail and observable evidence for:

1. Mouse `RepeatCount(3)` completes exactly three cycles.
2. Keyboard `RepeatCount(3)` enters exactly `aaa`.
3. Macro recording omits the recording hotkey and preserves key down/up, right click, wheel, and delays.
4. Macro playback reproduces the recorded sequence once.
5. F8 interrupts the five-second wait and returns to idle.
6. F12 interrupts the held `Shift` and subsequent typing proves Shift is not stuck.
7. Closing from tray while idle exits cleanly.
8. A named preset survives restart and restores all three mode settings.
9. A deliberately malformed imported copy is rejected without changing the active preset.

- [ ] **Step 4: Save verification evidence**

Write `docs/verification/2026-08-30-controlled-input-smoke.md` with environment, binary SHA-256, exact actions, results, and any deviations. A failed row remains failed until rerun; do not infer success from UI state alone.

- [ ] **Step 5: Re-run non-input checks after any smoke fix**

```powershell
npm.cmd run lint
npm.cmd run build
npm.cmd run test:rust
npm.cmd run tauri build -- --bundles nsis
git diff --check
```

Expected: all commands exit 0 and a new installer is produced if code changed.

- [ ] **Step 6: Commit verification evidence**

```powershell
git add docs/verification/2026-08-30-controlled-input-smoke.md
git commit -m "test: record controlled Windows input smoke"
```

## Final Acceptance Checklist

- [ ] `git status --short` shows no unintended or untracked files.
- [ ] The repository remains based on upstream commit `e37448f536aea18350dfbcbd4fef90bca218341d`, with local history preserved.
- [ ] Mouse, keyboard, and macro are mutually exclusive and start stopped.
- [ ] Each mode passes manual, count, and hard-duration stopping semantics.
- [ ] F8 is configurable, recording hotkey is configurable, and fixed F12 works during capture and execution.
- [ ] Cancellation during wait/hold releases only application-owned inputs.
- [ ] Recorder preserves down/up, modifiers, clicks, wheel, and monotonic delays while excluding control hotkeys.
- [ ] Presets have stable IDs, validated v3 JSON, 50 MiB import limit, atomic replacement, backup recovery, and legacy migration.
- [ ] Linux, jiggler/O-Zone, CLI, autostart, updater/prerelease, community/network entry points, and extra languages are absent.
- [ ] Chinese is default and English is the complete fallback.
- [ ] Lint, frontend build, Rust tests, Clippy, NSIS build, and controlled smoke each have real exit/result evidence.
- [ ] LICENSE and NOTICE preserve upstream MIT attribution.
