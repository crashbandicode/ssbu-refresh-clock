# Project instructions

This is a standalone Rust Skyline research plugin, not part of Deluxe or SGPO.

- Keep the plugin Rust-only and in-game. Ship one `libssbu_refresh_clock.nro`.
- Preserve the original event wait, atomic ordering, termination check and genuine
  catch-up behavior. Do not replace global SDK waits or force extra-event counts
  to zero. Never insert a callback inside an exclusive-load/store retry loop.
- Preserve the full version/site guard and stock fallback for unsupported modes.
- Do not patch the SGPO-sensitive `0x374812c` neighborhood or introduce renderer,
  GPU wait, buffer-index, input, HOME, clock-frequency or gameplay-state changes.
- Do not claim console success from a build, host test or FPS overlay alone.
- Native Ubuntu commands use `/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command`.
- Before completing Cargo work: `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `cargo build --no-default-features`, `cargo skyline clippy` with zero warnings,
  and `cargo skyline build --release`.
- Forgejo is the publication target. Commit/push/publish only when requested.
- Keep handoffs, operational receipts, symbols, hashes and release artifacts in
  the external evidence directory. Never commit Nintendo binaries/assets, build
  outputs, credentials, private transcripts or generic model watchers.
