# Validation — 0.0.1

**Local validation passed; console acceptance is pending.** This release is a
reviewable research candidate, not a verified game-speed or frame-pacing fix.

## Completed checks

| Check | Result |
| --- | --- |
| `cargo fmt` | Successful |
| `cargo test` | 14 passed, 0 failed |
| `cargo clippy --all-targets -- -D warnings` | Zero warnings |
| `cargo build --no-default-features` | Successful |
| `cargo skyline clippy` | Zero warnings; Switch-only runtime checked |
| `cargo skyline build --release` | Successful optimized NRO and unstripped ELF |
| Full worker and import-thunk signatures | Match the retained 13.0.5 executable |
| Independent initial source audit | No worker-loop, offset or session blocker found |
| Independent metadata-writer audit | Confirmed active-byte probe semantics; incorporated |

Tests cover 60/120 Hz clock units, fractional phase across render boundaries,
genuine delayed-frame credits, native-mode transitions, missing metadata,
active-byte probe clears, invalid-block/API activation reset, strict IPC replies,
copied-handle cleanup identification and address-range selection.

The parent reviewed final integration and ran the checks after integrating the
active-byte latch, rejected-reply cleanup and opt-in continuous logging. The initial
independent audit is not represented as having reviewed those later edits.

## Artifact provenance

The release's `manifest.json` identifies the source commit/tag, source-input
hashes and NRO SHA256. `SHA256SUMS.txt` covers the downloadable artifacts. The
unstripped ELF and raw analysis evidence are retained externally, not in Git.

The candidate was developed against an Erista Switch 1 setup using Smash 13.0.5,
SaltyNX 2.1.2 and FPSLocker 3.5.0. A doubled match timer was reported at 120 Hz /
60 FPS before the candidate. That report is symptom evidence, not a candidate
test result. No console files were changed during development.

## Required hardware acceptance

1. Verify the version guard, worker entry, shared mapping and mode gate activate.
2. Check approximately 120 physical wakes and 60 logical increments per second
   using the interval counters. Those are event-thread counters, not game updates.
3. Confirm 20 game-timer seconds take approximately 20 real seconds at 120 Hz /
   60 FPS, without pauses or slow-motion events.
4. Verify native 60 Hz behavior and restore the requested 120 Hz / 60 FPS setup.
5. Exercise relaunch, HOME/resume, dock/undock and menu/match transitions.
6. Compare frametimes with continuous telemetry disabled and other settings held
   constant. Test compatibility with the actual installed mod stack.

The full wait/termination loop is preserved, but hook load order, half-event parity,
minimum-one-credit behavior and unmapped third-party hooks remain runtime questions.
SaltyNX error paths can temporarily report 60 Hz. The candidate deliberately exits
normalization immediately rather than debounce all fields and delay a genuine
120-to-60 Hz transition; unexpected `mode_transitions` can reveal such changes.
No HOME correction, latency reduction, stable frametimes or full mod compatibility
is claimed.
