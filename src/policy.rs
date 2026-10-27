//! Convert physical event wakes to the game's original 60 Hz clock units.
//! The caller still performs its original atomic operation and stop check.

#[derive(Default)]
pub struct Clock {
    half_event: bool,
}

impl Clock {
    /// At the confirmed 120 Hz / 60 FPS configuration, two events make one tick.
    /// Every other configuration retains the stock increment of one per wake.
    pub fn on_event(&mut self, normalize_120: bool) -> u64 {
        if !normalize_120 {
            self.half_event = false;
            return 1;
        }
        self.half_event = !self.half_event;
        u64::from(!self.half_event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_supported_display_rates_supply_sixty_ticks_per_second() {
        for (hz, normalize) in [(60, false), (120, true)] {
            let mut clock = Clock::default();
            for _ in 0..120 {
                let ticks: u64 = (0..hz).map(|_| clock.on_event(normalize)).sum();
                assert_eq!(ticks, 60);
            }
        }
    }

    #[test]
    fn a_delayed_render_still_observes_all_elapsed_logical_ticks() {
        // The worker keeps running independently of the blocked renderer.
        let mut clock = Clock::default();
        let mut counter = 900_u64;
        let previous = counter;
        for _ in 0..8 {
            counter += clock.on_event(true);
        }
        assert_eq!(counter - previous, 4); // 8/120s = 4/60s.
        assert_eq!((counter - previous).saturating_sub(1), 3); // Real catch-up remains.
    }

    #[test]
    fn conversion_keeps_subtick_phase_across_render_boundaries() {
        let mut clock = Clock::default();
        let first: u64 = (0..3).map(|_| clock.on_event(true)).sum();
        let second: u64 = (0..3).map(|_| clock.on_event(true)).sum();
        assert_eq!((first, second), (1, 2));
        assert_eq!(first + second, 3);
    }

    #[test]
    fn returning_to_native_never_halves_the_native_clock() {
        let mut clock = Clock::default();
        assert_eq!(clock.on_event(true), 0); // Pending half event.
        for _ in 0..600 {
            assert_eq!(clock.on_event(false), 1);
        }
        assert_eq!(clock.on_event(true), 0); // New epoch, no stale phase.
        assert_eq!(clock.on_event(true), 1);
    }

    #[test]
    fn missing_metadata_or_observe_only_preserves_every_original_increment() {
        let mut clock = Clock::default();
        for _ in 0..1200 {
            assert_eq!(clock.on_event(false), 1);
        }
    }
}
