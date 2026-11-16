# Task Split Delivery Design

## Goal

Replace the unpushed mixed Task 5 change with small, independently reviewable
Windows auto-clicker repairs, then produce a verified Windows NSIS installer.

## Decision

The existing mixed implementation is not a delivery unit. It is split into
three stacked pull requests, each with one final review only:

1. **Macro playback safety.** Own the macro run lifecycle through the runtime
   coordinator, interrupt every macro action safely, preserve legacy
   `RawMove` playback only, release only process-owned inputs, and return to
   `Idle` after macro cleanup. This PR includes tests for keyboard and mouse
   holds, sleep, deadlines, send failures, repeat counts, and release order.
2. **Executor lifecycle unification.** Route real mouse, keyboard, recording,
   and hotkey entry points through the coordinator. A mode remains busy until
   the executor has observed cancellation and released held inputs. Selecting
   Macro must update the coordinator before recording starts.
3. **Linux compatibility and removal boundary.** Keep any required legacy
   Linux behavior isolated from the Windows delivery path. Do not mix Linux
   adapter changes into either of the first two pull requests.

## Ownership Contract

`RuntimeCoordinator` owns the logical run: active mode, cancellation token,
deadline, and the sole transition that permits another mode to start.
`InputSession` owns physical input tokens and must finish its cleanup before
the executor reports completion to the coordinator. Failed explicit `Up`
operations remain owned for cleanup retry.

## Packaging Gate

No installer is treated as deliverable until all three pull requests have a
clean final review and their focused tests, full Rust suite, lint, and diff
checks pass. Packaging then runs the repository's Windows NSIS build command
and verifies the generated `.exe` artifact exists; no real input injection is
used for this gate.
