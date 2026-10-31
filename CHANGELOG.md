# Changelog

## 0.0.2-dev — operator-confirmed console result, 2026-10-27

- Reuse SaltyNX's existing readable 4 KiB shared page in the game before requesting
  another session/handle/mapping. Reject ambiguous or incomplete scans.
- Record `shared_source` and `last_shared_error_stage` in diagnostics.
- This addresses the observed 0.0.1 initialization failure: the hook ran but all
  60 metadata-acquisition attempts ended with kernel `OutOfResource` (`0x10801`),
  leaving normalization inactive. The old log does not identify the failing SVC.
- Fifteen host tests and the required host/Switch build and Clippy checks pass.
- Operator confirmed the test works at 120 Hz / 60 FPS with Deluxe in Vanilla
  mode after the updated NRO was installed and hash-readback verified. Normal-speed
  gameplay is confirmed for this setup; the report includes no stopwatch values.
- Post-fix diagnostic readback, 60 Hz/transition regression checks, clean-I/O
  frametime measurements and input-latency measurements remain outstanding.
- This is a local development build; the published 0.0.1 release is unchanged.

## 0.0.1 — 2026-10-27

Initial research prerelease for source review. Its later console trial confirmed
hook loading but failed metadata acquisition, so normalization did not activate.

- Signature-guarded replacement of Smash 13.0.5's VSync-count worker.
- Convert two physical event wakes into one logical tick only for the requested
  NVN / 120 Hz / interval 2 / docked-target-60 configuration.
- Preserve every event wait, atomic operation, stop check, and real catch-up count.
- Read the existing SaltyNX 2.1.2 shared block through a short-lived IPC session.
- Treat the plugin-active byte as an initialization latch so an overlay's
  liveness probe does not reset the half-event phase.
- Observe-only mode, startup diagnostics, and opt-in continuous telemetry.
- Fourteen host tests; clean host/Switch Clippy and successful Skyline release build.

No successful normal-speed result was established for 0.0.1. HOME timing is
outside this plugin's scope.
