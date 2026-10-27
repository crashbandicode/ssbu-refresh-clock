# Design and evidence boundaries

## Timing path

Static analysis of the retained Smash Ultimate 13.0.5 main executable identified:

1. Registration of a display VSync event through `nn::vi::GetDisplayVsyncEvent`.
2. A `VSync Count Thread` callable at main-relative `0x3810ff0`, which waits on
   device + `0x108`, increments the atomic counter at + `0x138`, and checks stop
   byte + `0x3b0` after each event.
3. Present/acquire at `0x38701e0` compares that count with its previous sample
   (+ `0x140`) and reports extra events.
4. The main loop deposits one plus that excess into pending simulation credits,
   which its normal/catch-up scheduler consumes.

This establishes a mechanism by which 120 physical events and 60 presentations
can produce extra gameplay updates. It does not prove every frame always updates
twice: the scheduler has state-dependent suppression and a minimum-one-credit rule.

The candidate replaces only the dedicated worker, after matching all 76 original
bytes and the 16-byte wait-import thunk. It does not hook present/acquire or the
main-loop caller. The matching binary itself is not distributed.

## Clock conversion

`src/policy.rs` retains one half-event of phase per worker. In the supported
configuration, increments alternate `0, 1`; elsewhere they remain `1` per event.
Returning to native mode clears the pending fraction, never the game counter.

Every iteration still waits on the original event, performs an AcqRel atomic
operation (including `fetch_add(0)` on half ticks), and checks termination.
The worker continues counting while rendering stalls, so four elapsed logical
ticks still allow the existing helper to report three extra ticks.

There is no allocation, formatting, file I/O or IPC in the event loop.

## Mode detection and resource lifetime

The gate reads SaltyNX's existing shared memory. It requires valid magic, NVN API,
120 in both refresh fields, interval 2 and a docked target of 60. The active byte
must have been observed as 1 for this valid NVN block. It is then latched because
Status Monitor can clear it as a liveness probe. Invalid magic or a non-NVN API
resets the latch. Actual mode/interval/target changes take effect immediately.

The inspected prefix is 174 bytes of SaltyNX 2.1.2's 310-byte core block:

| Field | Byte offset in block |
| --- | ---: |
| Magic `0x00465053` | 0 |
| Active | 9 |
| Interval | 11 |
| API | 14 |
| Current refresh | 159 |
| Docked FPS target | 164 |

Global refresh is byte 1 of the 4096-byte shared page. Block discovery accepts
exactly one complete, four-byte-aligned matching prefix.

SaltyNX accepts only one service session at a time. A background thread obtains
a copied shared-memory handle with command 7, ends the session with command 0,
closes it, and maps the existing object. No display setters or allocation commands
are sent. The map must use the object's RW permission, but the candidate only
reads it. It excludes heap/alias ranges and existing mappings, closes the copied
handle, and retains the successful mapping for the game-process lifetime.

Unavailable/rejected metadata retains native counter increments. A rejected CMIF
reply also closes an identifiable copied handle rather than leaking it.

## Diagnostics

Default behavior writes one startup snapshot and stops the monitor after readiness
or about 60 initialization iterations. `telemetry.flag`, latched at launch, enables
one status write per second with interval counters and elapsed system ticks.
Use it for activation/rate checks; disable it for frametime comparisons.

`observe-only.flag` keeps all original increments while collecting the same
counters. Neither flag changes an existing display configuration.

## Upstream references

These are source/protocol references, not claims of endorsement or binary matching:

- [Skyline Rust bindings](https://github.com/skyline-rs/skyline-rs), dependency 0.6.0.
- [SaltyNX 2.1.2](https://github.com/masagrator/SaltyNX/tree/af8941410ff13b63586b615bf5a0eba340d782fd):
  `saltysd_core/source/NX-FPS-Common.hpp`, `api/NVN.cpp`, `ipc.h`, and
  `saltysd_proc/source/service.c`, `main.c`, `display_refresh_rate.cpp`.
- [FPSLocker 3.5.0](https://github.com/masagrator/FPSLocker/tree/0931909324a917b8cd0eea58587c170075327d94):
  shared-field consumers and docked target controls. It does not clear the active byte.
- [Status Monitor probe](https://github.com/masagrator/Status-Monitor-Overlay/blob/0434c0bac70f4fd2a82db85179f283fb9c504d77/source/Utils.hpp):
  `CheckIfGameRunning` can clear the active byte until the next presentation.
  This upstream version was not matched to an installed overlay.

## Remaining runtime questions

The worker may already be running before Skyline installs the entry hook; zero
`thread_entries` would reveal that. Half-event parity can interact with present
sampling and the caller's minimum-one-credit rule. Shared mode fields are not one
atomic snapshot and can change during transitions. These require console evidence;
host tests cannot establish a universal timing correction.
