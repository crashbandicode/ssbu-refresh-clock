# On the console: files, flags and open checks

2026-11-13. Moved from the README and checked against `src/runtime/mod.rs`
and `src/runtime/ipc.rs`. Results are in [VALIDATION.md](VALIDATION.md).

## Files

- NRO:
  `atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libssbu_refresh_clock.nro`
- Status: `ultimate/ssbu-refresh-clock/status.txt`
- The release's SD-layout ZIP holds only the NRO at that path and a short
  `README.txt`. It turns on no flag.

## Flags (read once, at launch)

- `ultimate/ssbu-refresh-clock/observe-only.flag`: the worker still counts
  everything, but every increment stays at its original value.
- `ultimate/ssbu-refresh-clock/telemetry.flag`: the status file is rewritten
  once a second. Without it, the status thread writes one final snapshot
  once initialization is done (or after 60 one-second rounds) and exits; the
  event worker and the shared mapping stay. A startup snapshot is not a rate
  measurement. Remove the flag and relaunch for frame-time tests, so the SD
  writes do not interfere.
- Fully exit and relaunch Smash after changing the NRO or a flag. Opening
  HOME does not start a new game process.

## Rollback

Exit Smash, remove this plugin's NRO (or put back the previous one), remove
any flags you added, and relaunch. Back up the plugin folder before a trial
and keep other plugins and settings as they are.

## Reading the status file

- Active at 120 Hz / 60 FPS: `guard_ok=true`, `thread_entries=1`,
  `metadata=Sample { valid: true, … }`, `normalizing=true`.
- With `telemetry.flag`, once `interval_valid=true`, the file holds the change
  since the previous sample: `interval_physical_events * 19200000 /
  interval_system_ticks` is the refresh rate the worker saw and
  `interval_logical_ticks` the ticks it passed on. These are event-thread
  counters, not measured gameplay updates. Leave out start-up and mode
  changes; one lifetime average is misleading.
- `shared_source` says where SaltyNX's settings came from
  (`borrowed_saltynx_page` or `new_shared_mapping`); `last_shared_error_stage`
  names the failed step and `last_ipc_error` its result code.

## Checks still open (2026-11-13)

1. Done once, 2026-11-02 (0.1.0, card readback): at 120 Hz / 60 FPS the
   status above, about 120 refreshes and 60 ticks a second.
2. Done once, 2026-11-02: at 60 Hz one tick per refresh and
   `normalizing=false`. The match timer at 60 Hz was not checked.
3. Open: dock/undock, HOME/resume, relaunch, menu/match transitions and
   frame times, with `telemetry.flag` removed for the frame-time test. The
   original catch-up loop must still run for genuinely slow frames.
4. Open: game speed against a stopwatch with 0.1.0. On another setup,
   compare 20 game-timer seconds with 20 real seconds in a normal match. The
   game's minimum-one-credit rule means one good result does not cover every
   rendering condition.
