//! SaltyNX 2.1.2's shared-memory prefix. No writes or allocation requests.
pub const PAGE_SIZE: usize = 0x1000;
pub const PREFIX_SIZE: usize = 174;
pub const MAGIC: u32 = 0x0046_5053;
pub const ACTIVE: usize = 9;
pub const INTERVAL: usize = 11;
pub const API: usize = 14;
pub const REFRESH: usize = 159;
pub const DOCKED_TARGET: usize = 164;

#[derive(Clone, Copy, Debug, Default)]
pub struct Sample {
    pub valid: bool,
    pub global_refresh: u8,
    pub active: u8,
    pub interval: u8,
    pub api: u8,
    pub refresh: u8,
    pub docked_target: u8,
}

impl Sample {
    fn matches_120_60(self) -> bool {
        self.valid
            && self.global_refresh == 120
            && self.interval == 2
            && self.api == 1
            && self.refresh == 120
            && self.docked_target == 60
    }
}

/// NX-FPS sets pluginActive on presentation; Status Monitor can clear it to probe
/// liveness. It is an initialization latch, not a per-event mode setting.
#[derive(Default)]
pub struct Gate {
    active_seen: bool,
}

impl Gate {
    pub fn normalize_120(&mut self, sample: Sample) -> bool {
        if !sample.valid || sample.api != 1 {
            self.active_seen = false;
        } else if sample.active == 1 {
            self.active_seen = true;
        }
        // Actual refresh/interval/target changes take effect immediately. Never
        // keep halving a genuine 60 Hz stream behind a general debounce timeout.
        self.active_seen && sample.matches_120_60()
    }
}

/// Only accept a unique aligned block, never a truncated prefix.
pub fn find_prefix(page: &[u8]) -> Option<usize> {
    if page.len() != PAGE_SIZE {
        return None;
    }
    let mut found = None;
    for offset in (4..=PAGE_SIZE - PREFIX_SIZE).step_by(4) {
        if page[offset..offset + 4] == MAGIC.to_le_bytes() {
            if found.is_some() {
                return None;
            }
            found = Some(offset);
        }
    }
    found
}

/// Exact no-buffer/no-PID command format used by the inspected SaltySD service.
pub fn request(command: u32) -> [u32; 16] {
    let mut words = [0; 16];
    words[0] = 4; // HIPC Request.
    words[1] = 8; // 16-byte CMIF data plus 16 bytes padding budget.
    words[4] = 0x4943_4653; // SFCI.
    words[6] = command;
    words
}

/// Command7 response: one copied handle, no other descriptors, successful SFCO.
pub fn shared_handle_reply(words: &[u32; 16]) -> Option<u32> {
    if words[0] != 4
        || words[1] != 0x8000_0008
        || words[2] != 2
        || words[3] == 0
        || words[4] != 0x4f43_4653
        || words[5..8] != [0, 0, 0]
    {
        return None;
    }
    single_copied_handle(words)
}

/// Resource ownership is independent of CMIF success. A rejected reply may still
/// contain a kernel-delivered handle that must be closed. No PID or extra handles.
pub fn single_copied_handle(words: &[u32; 16]) -> Option<u32> {
    (words[1] & 0x8000_0000 != 0 && words[2] == 2 && words[3] != 0).then_some(words[3])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready() -> Sample {
        Sample {
            valid: true,
            global_refresh: 120,
            active: 1,
            interval: 2,
            api: 1,
            refresh: 120,
            docked_target: 60,
        }
    }

    #[test]
    fn only_the_requested_mode_activates_normalization() {
        assert!(Gate::default().normalize_120(ready()));
        for sample in [
            Sample {
                valid: false,
                ..ready()
            },
            Sample {
                global_refresh: 60,
                ..ready()
            },
            Sample {
                active: 0,
                ..ready()
            },
            Sample {
                interval: 1,
                ..ready()
            },
            Sample { api: 2, ..ready() },
            Sample {
                refresh: 60,
                ..ready()
            },
            Sample {
                docked_target: 120,
                ..ready()
            },
            Sample::default(),
        ] {
            assert!(!Gate::default().normalize_120(sample));
        }
    }

    #[test]
    fn heartbeat_probe_preserves_phase_and_sixty_ticks() {
        let mut gate = Gate::default();
        let mut clock = crate::policy::Clock::default();
        let mut ticks = 0;
        for wake in 0..120 {
            let sample = Sample {
                active: u8::from(!(3..5).contains(&wake)),
                ..ready()
            };
            ticks += clock.on_event(gate.normalize_120(sample));
        }
        assert_eq!(ticks, 60);
    }

    #[test]
    fn real_mode_changes_exit_immediately_and_invalid_blocks_reset_activation() {
        let mut gate = Gate::default();
        assert!(gate.normalize_120(ready()));
        assert!(!gate.normalize_120(Sample {
            global_refresh: 60,
            refresh: 60,
            interval: 1,
            ..ready()
        }));
        assert!(!gate.normalize_120(Sample {
            docked_target: 120,
            ..ready()
        }));
        assert!(!gate.normalize_120(Sample::default()));
        assert!(!gate.normalize_120(Sample {
            active: 0,
            ..ready()
        }));
        assert!(gate.normalize_120(ready()));
        assert!(!gate.normalize_120(Sample { api: 2, ..ready() }));
        assert!(!gate.normalize_120(Sample {
            active: 0,
            ..ready()
        }));
    }

    #[test]
    fn locate_one_complete_aligned_prefix_and_reject_ambiguity() {
        let mut page = [0_u8; PAGE_SIZE];
        assert_eq!(find_prefix(&page), None);
        page[4..8].copy_from_slice(&MAGIC.to_le_bytes());
        assert_eq!(find_prefix(&page), Some(4));
        page[316..320].copy_from_slice(&MAGIC.to_le_bytes());
        assert_eq!(find_prefix(&page), None);
        page[4..8].fill(0);
        assert_eq!(find_prefix(&page), Some(316));
        assert_eq!(find_prefix(&page[..400]), None);
        page.fill(0);
        page[PAGE_SIZE - 4..].copy_from_slice(&MAGIC.to_le_bytes());
        assert_eq!(find_prefix(&page), None);
    }

    #[test]
    fn ipc_request_matches_libnx_layout_without_writing_configuration() {
        let words = request(7);
        assert_eq!(&words[..8], &[4, 8, 0, 0, 0x4943_4653, 0, 7, 0]);
        assert!(words[8..].iter().all(|w| *w == 0));
        assert_eq!(request(0)[6], 0); // EndSession, not a display setter.
    }

    #[test]
    fn accept_expected_handle_and_reject_changed_response_layout_or_error() {
        let mut words = [0_u32; 16];
        words[..8].copy_from_slice(&[4, 0x8000_0008, 2, 42, 0x4f43_4653, 0, 0, 0]);
        assert_eq!(shared_handle_reply(&words), Some(42));
        for index in [0, 1, 2, 4, 5, 6, 7] {
            let mut changed = words;
            changed[index] ^= 1;
            assert_eq!(shared_handle_reply(&changed), None);
        }
        words[3] = 0;
        assert_eq!(shared_handle_reply(&words), None);
    }

    #[test]
    fn rejected_cmif_reply_still_exposes_its_copied_handle_for_cleanup() {
        let mut words = [0_u32; 16];
        words[..8].copy_from_slice(&[4, 0x8000_0008, 2, 42, 0x4f43_4653, 0, 0xee01, 0]);
        assert_eq!(shared_handle_reply(&words), None);
        assert_eq!(single_copied_handle(&words), Some(42));
        words[2] = 3; // SendPID changes the handle offset: do not close a PID word.
        assert_eq!(single_copied_handle(&words), None);
        words[2] = 2;
        words[1] &= !0x8000_0000; // No special header: this is not a handle.
        assert_eq!(single_copied_handle(&words), None);
    }
}
