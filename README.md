# ssbu-refresh-clock

> **Disclaimer:** this project was developed with the assistance of AI
> (Claude, Codex and other coding agents) under the operator's direction and
> review. Nothing is claimed beyond the dated tests cited below.

An experimental Skyline plugin for Super Smash Bros. Ultimate 13.0.5 on a
Switch set to 120 Hz docked output with a 60 FPS cap (through SaltyNX and
FPSLocker). In that setup the game's clock counts 120 display refreshes a
second, and the reported match timer ran about twice as fast. The plugin
makes the clock count two refreshes as one tick in exactly that setup; in
every other setup it changes nothing.

There is no GIF: the effect is game speed on a console with SaltyNX, and Eden
has no SaltyNX (run e18, 2026-10-31: the plugin loaded and stayed inactive).

## Status

- **2026-10-27:** with 0.0.2-dev on the operator's Erista Switch at
  120 Hz / 60 FPS (Deluxe in Vanilla mode), the operator reported normal game
  speed after a full relaunch and a timed match. No stopwatch numbers were
  taken.
- **2026-10-31:** 0.1.0 released: the same code built with LTO, about
  400 KiB less game memory. It was installed on the Switch card that day, and
  the latest card install records (2026-11-12) still list it.
- **2026-11-02:** 0.1.0's status file, read back from the card, showed the
  correction on at 120 Hz / 60 FPS (about 120 refreshes and 60 ticks a
  second) and off at 60 Hz (one tick per refresh).
- Not measured yet: game speed against a stopwatch with 0.1.0, frame times,
  input latency, dock/undock and HOME transitions.

The 0.0.1 prerelease does not contain the fix: on the console it loaded but
could not read SaltyNX's settings, so it never switched on.

## Install

Tested on: Super Smash Bros. Ultimate 13.0.5 · Switch firmware 23.5.0 · Atmosphère 1.12.2

Copy `libssbu_refresh_clock.nro` from the
[v0.1.0 release](https://git.syntaxbyexample.com/intpa/ssbu-refresh-clock/releases/tag/v0.1.0)
to `sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/`, after
checking it against the release's `SHA256SUMS.txt`. It needs Skyline and an
already configured 120 Hz / 60 FPS setup; it was developed against SaltyNX
2.1.2 and FPSLocker 3.5.0. Fully exit and relaunch Smash after copying. To
undo it, delete the file. Flags, the status file and the open checks:
[docs/CONSOLE.md](docs/CONSOLE.md).

## Build

```text
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --no-default-features
cargo skyline clippy
cargo skyline build --release
```

The NRO needs [`cargo-skyline`](https://github.com/jam1garner/cargo-skyline);
plain host builds skip the Switch-only runtime. Output:
`target/aarch64-skyline-switch/release/libssbu_refresh_clock.nro`.

## More

- [docs/DESIGN.md](docs/DESIGN.md): how it works and where it stops.
- [docs/CONSOLE.md](docs/CONSOLE.md): files, flags, rollback and open checks.
- [docs/VALIDATION.md](docs/VALIDATION.md): dated test results.
- [CHANGELOG.md](CHANGELOG.md)
- Releases: [v0.1.0](https://git.syntaxbyexample.com/intpa/ssbu-refresh-clock/releases/tag/v0.1.0),
  [v0.0.1](https://git.syntaxbyexample.com/intpa/ssbu-refresh-clock/releases/tag/v0.0.1)
  (historical research release).

## License

MIT; see [LICENSE](LICENSE).
