# Validation

## 0.1.0 release build — 2026-10-31

0.1.0 is the code of the console-tested 0.0.2-dev build (below), built as a
cdylib only so that the release profile's LTO applies. That build
(`80bab7c8…`) was still on the Switch on 2026-10-31. Its working tree,
committed as `07829f2`, rebuilds to the same SHA-256 with the earlier crate
type.

| Evidence | Result |
| --- | --- |
| NRO | 61,440 bytes; resident 479,232 → 65,536 bytes |
| NRO SHA256 | `1246f448fcd42092423ff9df51b7eaade806f95ec3da460ba51cf8f2576ff3c0` |
| Exported symbols | Unchanged |
| Host checks | 15 tests; host and Switch Clippy; no-default-features and Skyline release builds |
| Console | Not yet run; LTO changes the generated code |

## Console result — 2026-10-27

**Normal-speed gameplay is operator-confirmed for the local 0.0.2-dev build at
120 Hz / 60 FPS.** After installation and the requested full-relaunch/timed-match
test, the operator reported that it worked and asked for the documentation to be
updated. This accepts resolution of the reported fast-gameplay symptom on this setup.
The reply supplied no numerical stopwatch readings; no post-fix status file has
been retrieved, so exact event rates and the selected acquisition path are not
yet confirmed by diagnostics.

Test context: Erista Switch 1, Smash Ultimate 13.0.5, firmware 23.0.1,
Atmosphère 1.12.1, SaltyNX 2.1.2, FPSLocker 3.5.0, sys-dock 1.0.2 and Deluxe
in Vanilla mode. The requested 120 Hz / 60 FPS configuration and existing profiles
were retained. Continuous telemetry was enabled for this trial.

| Evidence | Result |
| --- | --- |
| Tested version | Local `0.0.2-dev`; not the published `v0.0.1` artifact |
| NRO size | 475,136 bytes |
| NRO SHA256 | `80bab7c8cc4a4dfc1200dda9852c84df1ac3cef71c9af4617774396844853a20` |
| Installation verification | 67 fresh file readbacks; 65 protected existing files unchanged |
| Gameplay-speed result | Operator confirmed the updated test works |
| Post-fix metadata/event counters | Awaiting diagnostic readback |
| 60 Hz and transition regressions | Outstanding |
| Frametime/input-latency measurements | Outstanding; clean-I/O frametime test requires telemetry disabled |

The installed build has source-input/artifact hashes retained in the external
test manifest. It was built from local changes based on
`59ecd4d98f1b704a7fa516e0c878a69b047b9049`; it has no new published tag or release.

## 0.0.2-dev local correction

The installed 0.0.1 NRO was hash-verified against the public release. Its console
snapshot reported `guard_ok=true`, `thread_entries=1`, `observe_only=false`, but
`normalizing=false`, zero normalized events, invalid metadata and 60 unsuccessful
acquisition attempts ending in `0x10801` (kernel `OutOfResource`). This confirms
failed activation; it does not test the effect of an active half-rate counter.
The old diagnostic did not identify which acquisition stage exhausted resources.

SaltyNX 2.1.2's injected core already maps a 4 KiB shared-memory object into the
game before initializing NX-FPS (`saltysd_core/source/main.c`, lines 512–525 at
`af8941410ff13b63586b615bf5a0eba340d782fd`). The development change borrows that
existing mapping before attempting any redundant session/handle/map allocation.
Only complete readable SharedMem pages are inspected; unique-prefix validation
and the existing exact-mode gate remain required. Ambiguous/incomplete searches
retain the stock clock. New logs identify acquisition source and failure stage.

Passed: `cargo fmt`, 15 host tests, zero-warning host and Switch Clippy,
`cargo build --no-default-features`, and `cargo skyline build --release`.
The event worker, full signatures, clock policy and metadata mode gate are unchanged.
The operator-confirmed hardware result is recorded above. The acquisition change
has not had a separate independent source audit.

## 0.0.1 release validation (historical)

Local validation passed at release time. The subsequent console trial showed
failed metadata acquisition, as described above; 0.0.1 did not activate the fix.

### Completed release checks

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

### Release artifact provenance

The release's `manifest.json` identifies the source commit/tag, source-input
hashes and NRO SHA256. `SHA256SUMS.txt` covers the downloadable artifacts. The
unstripped ELF and raw analysis evidence are retained externally, not in Git.

The candidate was developed against an Erista Switch 1 setup using Smash 13.0.5,
SaltyNX 2.1.2 and FPSLocker 3.5.0. A doubled match timer was reported at 120 Hz /
60 FPS before the candidate. That report is baseline symptom evidence. Release
preparation involved no console writes; later console trials are recorded above.

## Remaining hardware validation

1. Retrieve the post-fix diagnostics to verify version guard, worker entry,
   acquisition source and mode activation. The old 0.0.1 log cannot verify 0.0.2-dev.
2. Check approximately 120 physical wakes and 60 logical increments per second
   using a representative match interval. Record whether the latest interval was
   captured after HOME/exit. Those are event-thread counters, not game updates.
3. Verify native 60 Hz behavior and restore the requested 120 Hz / 60 FPS setup.
4. Exercise relaunch, HOME/resume, dock/undock and menu/match transitions.
5. Compare frametimes with continuous telemetry disabled and other settings held
   constant. Test compatibility with the actual installed mod stack.

For reproducibility on additional setups, compare 20 game-timer seconds against
20 real seconds without pauses or slow-motion events. The current operator report
is sufficient to record the first normal-speed success; it is not a numerical
timing dataset or a full compatibility matrix.

The full wait/termination loop is preserved. The successful trial covers this
reported setup; hook load order elsewhere, half-event parity, minimum-one-credit
behavior and interactions with other mod stacks remain runtime questions.
SaltyNX error paths can temporarily report 60 Hz. The candidate deliberately exits
normalization immediately rather than debounce all fields and delay a genuine
120-to-60 Hz transition; unexpected `mode_transitions` can reveal such changes.
No HOME correction, latency reduction, stable frametimes or full mod compatibility
is claimed.
