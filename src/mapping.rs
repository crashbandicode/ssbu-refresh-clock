//! Select an unoccupied page without entering reserved heap/alias regions.
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
