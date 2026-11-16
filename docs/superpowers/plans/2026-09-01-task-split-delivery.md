# Task Split Delivery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver macro playback safety, executor lifecycle unification, Linux compatibility isolation, and a verified Windows installer as separate pull requests.

**Architecture:** `RuntimeCoordinator` is the logical-run authority. Each executor owns its `InputSession` until release cleanup completes, then reports exactly one terminal result. Platform adapters stay behind `InputSink`.

**Tech Stack:** Rust 2021, Tokio, Tauri 2, Enigo/Rdev, NSIS, TypeScript/Vite.

**Spec:** `docs/superpowers/specs/2026-09-01-task-split-delivery-design.md`

## Global Constraints

- Each task has a dedicated branch, one final review, and one pull request.
- No automated test sends real keyboard or mouse input.
- Preserve legacy `RawMove` deserialization/playback but reject new creation.
- Do not package a branch that has a blocked review.

---

### Task 1: Rebuild Macro Playback Safety PR

**Files:**

- Modify: `src-tauri/src/engine/macro_engine/playback.rs`
- Modify: `src-tauri/src/engine/macro_engine/state.rs`
- Modify: `src-tauri/src/engine/macro_engine/types.rs`
- Modify: `src-tauri/src/engine/input.rs`
- Modify: `src-tauri/src/engine/runtime.rs`
- Modify: `src-tauri/src/commands.rs`
- Test: playback, input, and runtime modules.

- [ ] **Step 1: Branch from `196904e`**

Create `task/05-macro-safety` from `196904e9881860be47529b56d2ac5b714f7b2aba`. Do not copy `clicker.rs`, `keyboard_clicker.rs`, `recording.rs`, or hotkey lifecycle changes.

- [ ] **Step 2: Add failing safety tests**

Cover keyboard and mouse Hold cancellation; deadline checks before every instant action, raw point, and recorded-text character; failed explicit `Up` retry; and a macro send error returning the coordinator to `Idle` before another macro run.

- [ ] **Step 3: Implement macro-only behavior**

Use `RuntimeCoordinator::start_macro`, coordinator-owned cancellation/deadline control, `InputSession`, and `interruptible_wait`. Terminal paths call `finish_macro` or cleaned `fail_macro` only after cleanup. Reject `RawMove` in add/update/duplicate command APIs.

- [ ] **Step 4: Verify and publish**

Run `cargo test --manifest-path src-tauri/Cargo.toml macro_engine::playback::tests`, `npm.cmd run test:rust`, `npm.cmd run lint`, and `git diff --check`. Commit `fix: make macro playback safely interruptible`, final-review once, push, and create the Task 5 PR.

---

### Task 2: Create Executor Lifecycle PR

**Files:**

- Modify: `src-tauri/src/engine/clicker.rs`
- Modify: `src-tauri/src/engine/keyboard_clicker.rs`
- Modify: `src-tauri/src/engine/macro_engine/recording.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/engine/runtime.rs`
- Test: executor and runtime modules.

- [ ] **Step 1: Add failing lifecycle tests**

Cover Busy while another mode runs; stop during a Hold remaining `Stopping` until cleanup; finite mouse/keyboard completion returning `Idle`; and Macro selection permitting recording.

- [ ] **Step 2: Implement real-entry-point coordination**

Route mouse, keyboard, macro, recording, and hotkeys through the coordinator. Stop cancels the executor token; the worker finishes the coordinator only after cleanup. Synchronize the selected UI mode before recording begins.

- [ ] **Step 3: Verify and publish**

Run focused executor/runtime tests, full Rust tests, lint, and diff check; commit, final-review once, push, and create the lifecycle PR based on Task 1.

---

### Task 3: Create Linux Compatibility Isolation PR

**Files:** platform adapter files and platform-mapping unit tests only.

- [ ] **Step 1: Add adapter-only failing tests**

Cover retained legacy Linux mappings and stable unsupported-input errors. Do not change Windows executor lifecycle behavior.

- [ ] **Step 2: Isolate platform behavior**

Keep Linux mapping/text behavior inside the sink boundary. Do not modify coordinator, hotkeys, recording lifecycle, clicker, or keyboard executor.

- [ ] **Step 3: Verify and publish**

Run focused tests, full Rust tests, lint, and diff check; commit, final-review once, push, and create the Linux PR based on Task 2.

---

### Task 4: Build and Verify Windows EXE

**Files:** Create `docs/superpowers/verification/2026-09-01-windows-package.md`.

- [ ] **Step 1: Confirm review and branch cleanliness**

Verify all three PR branches are clean and approved before packaging.

- [ ] **Step 2: Run the discovered Windows build script**

Use the `package.json` Windows/NSIS build script and preserve its exact exit code.

- [ ] **Step 3: Record the artifact**

Verify the generated NSIS `.exe` exists without launching it. Record its absolute path, size, checksum, timestamp, command, and exit code.
