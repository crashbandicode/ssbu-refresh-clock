# ssbu-refresh-clock

An experimental Rust Skyline plugin for **Super Smash Bros. Ultimate 13.0.5**,
targeting **120 Hz docked output, 60 rendered FPS, and normal gameplay speed**.

**Version 0.0.1 is a research prerelease for review. Console validation is pending.**
The source, host tests, and Switch build have been checked; normal game speed,
frame pacing, and compatibility on an actual Switch have not been established.

- [v0.0.1 release and downloads](https://git.syntaxbyexample.com/intpa/ssbu-refresh-clock/releases/tag/v0.0.1)
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
No shared state or display setting is written. Its sole IPC session is released
immediately after obtaining a copied shared-memory handle; no IPC in the event
loop. A missing service, rejected layout or unsupported display configuration
keeps the stock counter behavior.

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
- Rollback: exit Smash, remove only this added NRO, then relaunch. Preserve all
  existing mods/settings. No existing NRO is replaced by this candidate.

Back up the current plugin inventory before installation and verify the download
against the release's `SHA256SUMS.txt`. Fully close Smash before copying files.

## Acceptance still required

1. `guard_ok=true`, `thread_entries=1`, valid metadata and `normalizing=true` in
   status at 120 Hz / 60 FPS. Missing prerequisites are not successful activation.
2. With `telemetry.flag`, once normalization is active and `interval_valid=true`,
   the file includes the delta since its previous sample:
   `interval_physical_events * 19200000 / interval_system_ticks` should be roughly
   120/s, and the equivalent `interval_logical_ticks` rate roughly 60/s. These are
   event-thread counters, not measured gameplay updates. The latest file retains
   this interval so two live SD reads are unnecessary. Exclude startup and mode
   transitions; a single lifetime average is misleading.
3. A normal match's timer advances 20 seconds in approximately 20 real seconds.
4. At 60 Hz, the timer remains normal and counters return to one increment/event.
5. Check dock/undock, HOME/resume, full game relaunch, menu/match transitions and
   frametime behavior. Preserve the original catch-up loop for genuine slow frames.

The game's minimum-one-credit rule means a counter unit correction alone cannot
prove normal speed under arbitrary irregular rendering. Timer/frametime acceptance
on the actual console decides whether this candidate is sufficient.

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
