# Changelog

## 0.0.1 — 2026-10-27

Initial research prerelease for source review; hardware validation is pending.

- Signature-guarded replacement of Smash 13.0.5's VSync-count worker.
- Convert two physical event wakes into one logical tick only for the requested
  NVN / 120 Hz / interval 2 / docked-target-60 configuration.
- Preserve every event wait, atomic operation, stop check, and real catch-up count.
- Read the existing SaltyNX 2.1.2 shared block through a short-lived IPC session.
- Treat the plugin-active byte as an initialization latch so an overlay's
  liveness probe does not reset the half-event phase.
- Observe-only mode, startup diagnostics, and opt-in continuous telemetry.
- Fourteen host tests; clean host/Switch Clippy and successful Skyline release build.

Normal timer speed, stable frametimes, HOME behavior, input latency, and installed
mod compatibility have not been verified on hardware. HOME timing is outside this
plugin's scope.
