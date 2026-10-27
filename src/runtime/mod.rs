//! Version-guarded experimental clock correction, isolated from the renderer.
mod ipc;

use crate::{metadata, policy::Clock};
use skyline::hooks::{getRegionAddress, Region};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

const SITE: usize = 0x3810ff0;
const WAIT: usize = 0x39c1440;
const DIRECTORY: &str = "sd:/ultimate/ssbu-refresh-clock";
const WORDS: [u32; 19] = [
    0xf81d0ff5, 0xa9014ff4, 0xa9027bfd, 0x910083fd, 0xf9400414, 0x91042293, 0x9104e295, 0xaa1303e0,
    0x9406c10c, 0xc85ffea8, 0x91000508, 0xc809fea8, 0x35ffffa9, 0x394ec288, 0x34ffff28, 0xa9427bfd,
    0xa9414ff4, 0xf84307f5, 0xd65f03c0,
];

static MAIN: AtomicUsize = AtomicUsize::new(0);
static PAGE: AtomicUsize = AtomicUsize::new(0);
static PREFIX: AtomicUsize = AtomicUsize::new(0);
static OBSERVE_ONLY: AtomicBool = AtomicBool::new(false);
static NORMALIZING: AtomicBool = AtomicBool::new(false);
static THREAD_ENTRIES: AtomicU64 = AtomicU64::new(0);
static PHYSICAL: AtomicU64 = AtomicU64::new(0);
static LOGICAL: AtomicU64 = AtomicU64::new(0);
static NORMALIZED_EVENTS: AtomicU64 = AtomicU64::new(0);
static TRANSITIONS: AtomicU64 = AtomicU64::new(0);

unsafe fn sample() -> metadata::Sample {
    let page = PAGE.load(Ordering::Acquire);
    let prefix = PREFIX.load(Ordering::Acquire);
    if page == 0 || prefix == 0 {
        return metadata::Sample::default();
    }
    let p = prefix as *const u8;
    if (p as *const u32).read_volatile() != metadata::MAGIC {
        return metadata::Sample::default();
    }
    let read = |offset| p.add(offset).read_volatile();
    let result = metadata::Sample {
        valid: true,
        global_refresh: (page as *const u8).add(1).read_volatile(),
        active: read(metadata::ACTIVE),
        interval: read(metadata::INTERVAL),
        api: read(metadata::API),
        refresh: read(metadata::REFRESH),
        docked_target: read(metadata::DOCKED_TARGET),
    };
    if (p as *const u32).read_volatile() == metadata::MAGIC {
        result
    } else {
        metadata::Sample::default()
    }
}

#[skyline::hook(offset = SITE)]
unsafe extern "C" fn ssbu_refresh_clock_worker(callable: *mut u8) {
    if callable.is_null() || (callable as usize) & 7 != 0 {
        return original!()(callable);
    }
    let device = callable.add(8).cast::<*mut u8>().read();
    if device.is_null() || (device as usize) & 7 != 0 {
        return original!()(callable);
    }
    THREAD_ENTRIES.fetch_add(1, Ordering::Relaxed);
    let wait: unsafe extern "C" fn(*mut u8) =
        std::mem::transmute(MAIN.load(Ordering::Acquire) + WAIT);
    let counter = &*device.add(0x138).cast::<AtomicU64>();
    let mut clock = Clock::default();
    let mut gate = metadata::Gate::default();
    let mut previous_mode = false;
    loop {
        wait(device.add(0x108));
        let normalize = !OBSERVE_ONLY.load(Ordering::Relaxed) && gate.normalize_120(sample());
        let increment = clock.on_event(normalize);
        // Preserve the original atomic ordering even on a suppressed half tick.
        counter.fetch_add(increment, Ordering::AcqRel);
        PHYSICAL.fetch_add(1, Ordering::Relaxed);
        LOGICAL.fetch_add(increment, Ordering::Relaxed);
        if normalize {
            NORMALIZED_EVENTS.fetch_add(1, Ordering::Relaxed);
        }
        if normalize != previous_mode {
            TRANSITIONS.fetch_add(1, Ordering::Relaxed);
            previous_mode = normalize;
        }
        NORMALIZING.store(normalize, Ordering::Relaxed);
        // Every physical event retains the original stop check, including half ticks.
        if device.add(0x3b0).read_volatile() != 0 {
            break;
        }
    }
}

unsafe fn signature_matches(base: usize) -> bool {
    let end = getRegionAddress(Region::Rodata) as usize;
    if base == 0 || base.checked_add(WAIT + 16).is_none_or(|x| x > end) {
        return false;
    }
    let worker_ok = WORDS
        .iter()
        .enumerate()
        .all(|(i, word)| ((base + SITE + 4 * i) as *const u32).read_volatile() == *word);
    // Verify the exact import thunk used by the original worker as well.
    let wait_words = [0xd000c6f0, 0xf9448211, 0x91240210, 0xd61f0220];
    worker_ok
        && wait_words
            .iter()
            .enumerate()
            .all(|(i, word)| ((base + WAIT + 4 * i) as *const u32).read_volatile() == *word)
}

fn monitor(guard_ok: bool, telemetry: bool) {
    let _ = std::fs::create_dir_all(DIRECTORY);
    let mut attempts = 0;
    let mut last_error = 0;
    let mut previously_ready = false;
    let mut previous_counters = None;
    let start = unsafe { skyline::nn::os::GetSystemTick() } as u64;
    for round in 0_u64.. {
        if guard_ok && PAGE.load(Ordering::Acquire) == 0 && attempts < 60 {
            attempts += 1;
            match unsafe { ipc::map_existing_shared_memory() } {
                Ok(page) => PAGE.store(page, Ordering::Release),
                Err(error) => last_error = error,
            }
        }
        let page = PAGE.load(Ordering::Acquire);
        if page != 0 && PREFIX.load(Ordering::Acquire) == 0 {
            let mut bytes = [0_u8; metadata::PAGE_SIZE];
            for (i, byte) in bytes.iter_mut().enumerate() {
                *byte = unsafe { ((page + i) as *const u8).read_volatile() };
            }
            if let Some(offset) = metadata::find_prefix(&bytes) {
                PREFIX.store(page + offset, Ordering::Release);
            }
        }
        let ready = !guard_ok
            || (PREFIX.load(Ordering::Acquire) != 0 && PHYSICAL.load(Ordering::Relaxed) != 0);
        let complete = (ready && previously_ready) || round >= 59;
        if telemetry || complete {
            let current = unsafe { sample() };
            let now = unsafe { skyline::nn::os::GetSystemTick() } as u64;
            let physical = PHYSICAL.load(Ordering::Relaxed);
            let logical = LOGICAL.load(Ordering::Relaxed);
            let interval_valid = previous_counters.is_some();
            let (old_time, old_physical, old_logical) =
                previous_counters.unwrap_or((now, physical, logical));
            let elapsed_interval = now.wrapping_sub(old_time);
            let physical_interval = physical.wrapping_sub(old_physical);
            let logical_interval = logical.wrapping_sub(old_logical);
            let final_snapshot = !telemetry && complete;
            let status = format!(
                "candidate=0.0.1\nhardware_accepted=false\nguard_ok={guard_ok}\ncontinuous_telemetry={telemetry}\nsnapshot_final={final_snapshot}\nobserve_only={}\nthread_entries={}\nnormalizing={}\nphysical_events={physical}\nlogical_ticks={logical}\nnormalized_events={}\nmode_transitions={}\nelapsed_system_ticks={}\ntick_hz=19200000\ninterval_valid={interval_valid}\ninterval_system_ticks={elapsed_interval}\ninterval_physical_events={physical_interval}\ninterval_logical_ticks={logical_interval}\nmetadata={current:?}\nshared_map_attempts={attempts}\nlast_ipc_error=0x{last_error:08x}\nnot_measured=gameplay_updates,displayed_frames,frametimes,input_latency\n",
                OBSERVE_ONLY.load(Ordering::Relaxed), THREAD_ENTRIES.load(Ordering::Relaxed),
                NORMALIZING.load(Ordering::Relaxed), NORMALIZED_EVENTS.load(Ordering::Relaxed),
                TRANSITIONS.load(Ordering::Relaxed), now.wrapping_sub(start),
            );
            let _ = std::fs::write(format!("{DIRECTORY}/status.txt"), status);
            previous_counters = Some((now, physical, logical));
        }
        if !telemetry && complete {
            // One startup snapshot by default; no ongoing SD I/O during matches.
            // The mapping and event worker deliberately outlive this monitor.
            break;
        }
        previously_ready = ready;
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[skyline::main(name = "ssbu_refresh_clock")]
pub fn main() {
    let observe = std::path::Path::new(DIRECTORY)
        .join("observe-only.flag")
        .exists();
    OBSERVE_ONLY.store(observe, Ordering::Relaxed);
    let telemetry = std::path::Path::new(DIRECTORY)
        .join("telemetry.flag")
        .exists();
    let base = unsafe { getRegionAddress(Region::Text) as usize };
    MAIN.store(base, Ordering::Release);
    let guard_ok = unsafe { signature_matches(base) };
    if guard_ok {
        skyline::install_hook!(ssbu_refresh_clock_worker);
    }
    skyline::println!(
        "[ssbu_refresh_clock] experimental0.0.1 guard={guard_ok} observe_only={observe}"
    );
    let _ = std::thread::Builder::new()
        .name("ssbu-refresh-clock-status".into())
        .spawn(move || monitor(guard_ok, telemetry));
}
