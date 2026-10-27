//! One short SaltySD session to obtain its existing shared memory; no setters.
use crate::{mapping, metadata};
use core::arch::asm;

const PAGE: usize = metadata::PAGE_SIZE;

#[repr(C)]
#[derive(Default)]
struct MemoryInfo {
    address: u64,
    size: u64,
    kind: u32,
    attributes: u32,
    permission: u32,
    ipc_references: u32,
    device_references: u32,
    padding: u32,
}

unsafe fn close(handle: u32) {
    asm!("svc 0x16", inout("x0") u64::from(handle) => _, clobber_abi("C"), options(nostack));
}

unsafe fn connect() -> Result<u32, u32> {
    let result: u64;
    let handle: u64;
    asm!("svc 0x1f", lateout("x0") result,
        inout("x1") c"SaltySD".as_ptr() as u64 => handle,
        clobber_abi("C"), options(nostack));
    if result as u32 == 0 {
        Ok(handle as u32)
    } else {
        Err(result as u32)
    }
}

unsafe fn transact(handle: u32, command: u32) -> Result<[u32; 16], u32> {
    let tls: *mut u32;
    asm!("mrs {}, tpidrro_el0", out(reg) tls, options(nostack, nomem, preserves_flags));
    // Preserve this thread's IPC scratch area across the direct kernel calls.
    let mut saved = [0_u32; 64];
    core::ptr::copy_nonoverlapping(tls, saved.as_mut_ptr(), saved.len());
    let request = metadata::request(command);
    core::ptr::copy_nonoverlapping(request.as_ptr(), tls, request.len());
    let result: u64;
    asm!("svc 0x21", inout("x0") u64::from(handle) => result,
        clobber_abi("C"), options(nostack));
    let mut response = [0_u32; 16];
    core::ptr::copy_nonoverlapping(tls, response.as_mut_ptr(), response.len());
    core::ptr::copy_nonoverlapping(saved.as_ptr(), tls, saved.len());
    if result as u32 == 0 {
        Ok(response)
    } else {
        Err(result as u32)
    }
}

unsafe fn get_info(kind: u64) -> Result<u64, u32> {
    let result: u64;
    let value: u64;
    asm!("svc 0x29", lateout("x0") result, inout("x1") kind => value,
        in("x2") 0xffff_8001_u64, in("x3") 0_u64,
        clobber_abi("C"), options(nostack));
    if result as u32 == 0 {
        Ok(value)
    } else {
        Err(result as u32)
    }
}

unsafe fn query(address: u64) -> Result<MemoryInfo, u32> {
    let mut info = MemoryInfo::default();
    let result: u64;
    asm!("svc 0x6", inout("x0") &mut info as *mut MemoryInfo as u64 => result,
        lateout("x1") _, in("x2") address, clobber_abi("C"), options(nostack));
    if result as u32 == 0 {
        Ok(info)
    } else {
        Err(result as u32)
    }
}

unsafe fn map(handle: u32) -> Result<usize, u32> {
    let base = get_info(12)?;
    let end = base.checked_add(get_info(13)?).ok_or(0xffff_0001_u32)?;
    let excluded = [(get_info(2)?, get_info(3)?), (get_info(4)?, get_info(5)?)];
    let mut cursor = base;
    let mut last_error = 0xffff_0002;
    for _ in 0..1024 {
        if cursor >= end {
            break;
        }
        let info = query(cursor)?;
        let region_end = info
            .address
            .checked_add(info.size)
            .ok_or(0xffff_0003_u32)?
            .min(end);
        if region_end <= cursor {
            break;
        }
        if info.kind == 0 && info.permission == 0 {
            if let Some(address) =
                mapping::candidate(cursor.max(info.address), region_end, &excluded)
            {
                let result: u64;
                // SaltyNX creates remote permission RW; Horizon requires an exact match.
                // This candidate never writes the mapped object.
                asm!("svc 0x13", inout("x0") u64::from(handle) => result,
                    in("x1") address, in("x2") PAGE as u64, in("x3") 3_u64,
                    clobber_abi("C"), options(nostack));
                if result as u32 == 0 {
                    return Ok(address as usize);
                }
                last_error = result as u32;
                // An allocation race cannot overwrite anything: the kernel rejects it.
            }
        }
        cursor = region_end;
    }
    Err(last_error)
}

/// Retain the successful mapping for the entire game process lifetime. The
/// background thread owns initialization; the event worker never performs IPC.
pub(super) unsafe fn map_existing_shared_memory() -> Result<usize, u32> {
    let session = connect()?;
    let reply = transact(session, 7);
    // EndSession intentionally terminates the server's synchronous request.
    let _ = transact(session, 0);
    close(session); // Never hold SaltyNX's sole session while sleeping.
    let words = reply?;
    let Some(handle) = metadata::shared_handle_reply(&words) else {
        if let Some(handle) = metadata::single_copied_handle(&words) {
            close(handle);
        }
        return Err(0xffff_0004);
    };
    let mapped = map(handle);
    close(handle);
    mapped
}
