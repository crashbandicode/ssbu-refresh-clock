//! Recognize an existing shared page or select space for a fallback mapping.

/// SaltyNX 2.1.2 maps one 4 KiB shared-memory object into the game before NX-FPS.
/// Never scan heaps, code, device mappings, or unreadable/partial regions for it.
pub fn readable_shared_page(kind: u32, permission: u32, size: u64) -> bool {
    kind == 0x06 && matches!(permission, 1 | 3) && size == 0x1000
}

pub fn candidate(start: u64, end: u64, excluded: &[(u64, u64)]) -> Option<u64> {
    const PAGE: u64 = 0x1000;
    let mut address = start.checked_add(PAGE)?.checked_add(PAGE - 1)? & !(PAGE - 1);
    loop {
        let limit = address.checked_add(PAGE)?;
        if limit.checked_add(PAGE)? > end {
            return None;
        }
        let collision = excluded.iter().find(|&&(base, size)| {
            base.checked_add(size)
                .is_none_or(|reserved_end| address < reserved_end && base < limit)
        });
        match collision {
            Some(&(base, size)) => {
                address = base
                    .checked_add(size)?
                    .checked_add(PAGE)?
                    .checked_add(PAGE - 1)?
                    & !(PAGE - 1);
            }
            None => return Some(address),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_page_scan_only_reads_the_expected_shared_memory_region() {
        assert!(readable_shared_page(0x06, 1, 0x1000));
        assert!(readable_shared_page(0x06, 3, 0x1000));
        for kind in [0, 3, 4, 5, 0x0b] {
            assert!(!readable_shared_page(kind, 3, 0x1000));
        }
        for permission in [0, 2, 4, 5, 7] {
            assert!(!readable_shared_page(0x06, permission, 0x1000));
        }
        for size in [0, 0x800, 0x2000, u64::MAX] {
            assert!(!readable_shared_page(0x06, 3, size));
        }
    }

    #[test]
    fn leaves_guards_and_skips_reserved_ranges_in_any_order() {
        assert_eq!(candidate(0x1000, 0xa000, &[]), Some(0x2000));
        assert_eq!(
            candidate(0x1000, 0xa000, &[(0x3000, 0x3000), (0x1000, 0x2000)]),
            Some(0x7000)
        );
        assert_eq!(candidate(0x1000, 0xa000, &[(0, 0x9000)]), None);
    }

    #[test]
    fn refuses_short_or_overflowing_regions() {
        assert_eq!(candidate(0x1000, 0x3000, &[]), None);
        assert_eq!(candidate(u64::MAX - 8, u64::MAX, &[]), None);
        assert_eq!(candidate(0x1000, 0xa000, &[(0x2000, u64::MAX)]), None);
    }
}
