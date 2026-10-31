# ssbu-refresh-clock

An experimental Rust Skyline plugin for **Super Smash Bros. Ultimate 13.0.5**,
targeting **120 Hz docked output, 60 rendered FPS, and normal gameplay speed**.

**Normal-speed gameplay at 120 Hz / 60 FPS is operator-confirmed with 0.0.2-dev**
on the tested Erista setup (2026-10-27), using Deluxe's Vanilla mode. This is the
first successful console result; see [validation details](docs/VALIDATION.md) for
the tested build and remaining checks.

**0.1.0** (2026-10-31) releases that correction, built with LTO (about 400 KiB
less memory). The **0.0.1** prerelease does not contain it: that version's hook
loaded, but metadata acquisition failed with kernel `OutOfResource` (`0x10801`),
leaving normalization inactive. Since 0.0.2-dev the plugin reuses the shared page
SaltyNX already mapped into the game.

- [v0.1.0 release](https://git.syntaxbyexample.com/intpa/ssbu-refresh-clock/releases/tag/v0.1.0)
- [Historical v0.0.1 research release](https://git.syntaxbyexample.com/intpa/ssbu-refresh-clock/releases/tag/v0.0.1)
- [Design and source references](docs/DESIGN.md)
- [Validation results and remaining questions](docs/VALIDATION.md)
- [Changelog](CHANGELOG.md)

## Why this exists

A 60 FPS presentation cap does not necessarily mean 60 gameplay updates per
second. Smash counts display VSync events and can turn extra events into extra
simulation credits. The reported setup displayed 60 FPS at 120 Hz while its match
timer progressed approximately twice as fast. This candidate normalizes that
game-specific clock at its source while retaining real late-frame catch-up.

The implementation targets an **Erista Switch 1**, Smash **13.0.5**, SaltyNX
**2.1.2**, and FPSLocker **3.5.0**. Those are development baselines, not a tested
compatibility matrix. It requires a working Skyline installation and an already
configured 120 Hz / 60 FPS setup.

## Behavior

This standalone Rust Skyline NRO changes the unit of Smash's VSync-derived clock
from physical refresh events to 60 Hz ticks, only when SaltyNX reports the exact
requested configuration: NVN initialized, reported refresh 120 Hz in both SaltyNX
fields, interval 2, docked FPS target 60. Other states retain the original increment
of one per event. The shared `pluginActive` byte is latched after first activation
because Status Monitor may clear it for a liveness probe. Invalid block magic or
a non-NVN API resets that latch; refresh/interval/target changes apply immediately.

The original event wait, atomic counter ordering and stop check remain on every
physical event. At 120 Hz, two events advance the game counter once. The counter
continues advancing while rendering is slow, preserving real catch-up information.
No GPU/present/buffer-index hook, catch-up bypass, input hook or gameplay multiplier.
This does not modify HOME animation timing or promise to fix irregular frametimes.

The entire 76-byte 13.0.5 worker and its wait-import thunk must match before the
hook is installed. Metadata is read through SaltyNX 2.1.2's existing shared page.
Initialization first searches complete, readable 4 KiB shared-memory regions for
one unique complete NX-FPS prefix and borrows the existing game-lifetime mapping.
It does not read arbitrary heaps or code for this search. An incomplete scan or
ambiguous page selection fails closed. If no matching page exists, a short IPC
session obtains a copied shared-memory handle as a fallback, then closes promptly.
No shared state or display setting is written; no discovery/IPC runs in the event
loop. Missing metadata or an unsupported display configuration retains stock behavior.

## Files and rollback

The release contains the standalone NRO and an SD-layout ZIP. The ZIP contains
only the NRO at the path below plus a short installation note; it does not enable
diagnostic flags automatically. For a backed-up console trial:

- NRO destination:
  `atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libssbu_refresh_clock.nro`
- Diagnostics:
  `ultimate/ssbu-refresh-clock/status.txt`
- Optional baseline: create `ultimate/ssbu-refresh-clock/observe-only.flag` before
  launching Smash. It keeps the same observer but leaves all counter increments
  at their original value. The flag is latched at launch.
- Optional continuous counters: create `ultimate/ssbu-refresh-clock/telemetry.flag`
  before launch to update status once per second. Without it, the monitor writes
  one final startup snapshot after initialization (or 60 initialization iterations),
  then exits. The event worker and shared mapping remain active. A startup snapshot is
  not a live rate measurement. Remove this flag and relaunch for frametime tests
  so recurring SD writes cannot interfere with the comparison.
- Fully exit and relaunch Smash after changing the NRO or flag. Merely opening
  HOME does not start a new game process.
- Rollback: exit Smash, remove this plugin's NRO or restore its backed-up previous
  version when testing an update. Remove any test flags you added, then relaunch.
  Preserve other plugins and settings.

Back up the current plugin inventory before installation and verify the download
against the release's `SHA256SUMS.txt`. Fully close Smash before copying files.

## Console validation

The operator confirmed that the 0.0.2-dev test resolved the fast-gameplay symptom
after the requested full-relaunch and timed-match check. The report was qualitative;
no stopwatch values or post-fix diagnostic readback have been collected yet.
Remaining checks:

1. `guard_ok=true`, `thread_entries=1`, valid metadata and `normalizing=true` in
   status at 120 Hz / 60 FPS. Missing prerequisites are not successful activation.
2. With `telemetry.flag`, once normalization is active and `interval_valid=true`,
   the file includes the delta since its previous sample:
   `interval_physical_events * 19200000 / interval_system_ticks` should be roughly
   120/s, and the equivalent `interval_logical_ticks` rate roughly 60/s. These are
   event-thread counters, not measured gameplay updates. The latest file retains
   this interval so two live SD reads are unnecessary. Exclude startup and mode
   transitions; a single lifetime average is misleading.
   Development builds also report `shared_source` (normally `borrowed_saltynx_page`)
   and `last_shared_error_stage`; the legacy `last_ipc_error` holds the result code.
3. At 60 Hz, the timer remains normal and counters return to one increment/event.
4. Check dock/undock, HOME/resume, full game relaunch, menu/match transitions and
    frametime behavior. Preserve the original catch-up loop for genuine slow frames.

For reproduction on another setup, compare 20 game-timer seconds with 20 real
seconds in a normal match. The game's minimum-one-credit rule means the successful
result on this setup does not establish behavior under every rendering condition.
Stable frametimes and input-latency improvements have not been measured.

## Development

Host checks use Rust/Cargo. Building the NRO also requires an installed
[`cargo-skyline`](https://github.com/skyline-rs/cargo-skyline) toolchain; plain
host builds do not compile the Switch-only runtime module.

```text
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --no-default-features
cargo skyline clippy
cargo skyline build --release
```

Release output:
`target/aarch64-skyline-switch/release/libssbu_refresh_clock.nro`.

The committed `Cargo.lock` pins dependencies. Runtime code is Rust-only and the
Skyline dependency is pinned to 0.6.0. Nintendo binaries/assets, build caches,
release binaries, and private operational logs are not part of the source tree.
