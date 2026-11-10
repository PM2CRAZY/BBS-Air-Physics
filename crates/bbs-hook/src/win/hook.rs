//! Inline hooks on 64-bit code (as the KH2-Combat-in-KH1 mod's): the first bytes of a function
//! become a `jmp rel32` to a stub near the executable (an absolute jump to ours); a trampoline
//! holds the moved bytes and jumps back. The moved bytes must be whole, position-independent
//! instructions, checked against the Steam exe's before anything is written.

use super::sys::*;
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

/// `jmp [rip+0]; dq to` (14 bytes).
fn abs_jmp(to: usize) -> [u8; 14] {
    let mut b = [0u8; 14];
    b[0] = 0xFF;
    b[1] = 0x25;
    b[6..14].copy_from_slice(&(to as u64).to_le_bytes());
    b
}

/// Executable memory within 2 GB below `near` (for `rel32` jumps from the game's code).
unsafe fn alloc_near(near: usize, size: usize) -> Option<usize> {
    let mut at = (near & !0xFFFF).saturating_sub(0x1000_0000);
    let low = near.saturating_sub(0x7000_0000);
    while at > low {
        let p = VirtualAlloc(at as *mut c_void, size, 0x3000, 0x40);
        if !p.is_null() {
            return Some(p as usize);
        }
        at -= 0x10000;
    }
    None
}

/// Whether the bytes at `at` are `expect`.
pub unsafe fn bytes_are(at: usize, expect: &[u8]) -> bool {
    std::slice::from_raw_parts(at as *const u8, expect.len()) == expect
}

/// Hooks the function at `target` (whose first `expect.len()` bytes, at least 5 and whole
/// instructions, must be `expect`) to `ours`. `original` gets the trampoline (call it to run
/// BBS's own function). Returns false, changing nothing, if the bytes differ.
pub unsafe fn install(target: usize, expect: &[u8], ours: usize, original: &AtomicUsize, what: &str) -> bool {
    if expect.len() < 5 || !bytes_are(target, expect) {
        let have = std::slice::from_raw_parts(target as *const u8, expect.len());
        super::log(&format!("hook {what} at {target:#x}: unexpected bytes {have:02x?}, not hooked"));
        return false;
    }
    let Some(mem) = alloc_near(target, 64) else {
        super::log(&format!("hook {what}: no memory near {target:#x}"));
        return false;
    };
    let tramp = mem;
    std::ptr::copy_nonoverlapping(expect.as_ptr(), tramp as *mut u8, expect.len());
    let back = abs_jmp(target + expect.len());
    std::ptr::copy_nonoverlapping(back.as_ptr(), (tramp + expect.len()) as *mut u8, back.len());
    let stub = mem + 32;
    let go = abs_jmp(ours);
    std::ptr::copy_nonoverlapping(go.as_ptr(), stub as *mut u8, go.len());
    original.store(tramp, Ordering::Release);
    let mut old = 0u32;
    if VirtualProtect(target as *mut c_void, expect.len(), 0x40, &mut old) == 0 {
        super::log(&format!("hook {what}: VirtualProtect failed"));
        return false;
    }
    let rel = (stub as i64 - (target as i64 + 5)) as i32;
    let p = target as *mut u8;
    *p = 0xE9;
    std::ptr::copy_nonoverlapping(rel.to_le_bytes().as_ptr(), p.add(1), 4);
    for i in 5..expect.len() {
        *p.add(i) = 0x90;
    }
    let mut tmp = 0u32;
    VirtualProtect(target as *mut c_void, expect.len(), old, &mut tmp);
    FlushInstructionCache(GetCurrentProcess(), target as *const c_void, expect.len());
    super::log(&format!("hooked {what} at {target:#x}"));
    true
}

/// Writes `new` over `at` (code or data) if the bytes there are `expect`.
pub unsafe fn patch(at: usize, expect: &[u8], new: &[u8]) -> bool {
    if !bytes_are(at, expect) || expect.len() != new.len() {
        return false;
    }
    let mut old = 0u32;
    if VirtualProtect(at as *mut c_void, new.len(), 0x40, &mut old) == 0 {
        return false;
    }
    std::ptr::copy_nonoverlapping(new.as_ptr(), at as *mut u8, new.len());
    let mut tmp = 0u32;
    VirtualProtect(at as *mut c_void, new.len(), old, &mut tmp);
    FlushInstructionCache(GetCurrentProcess(), at as *const c_void, new.len());
    true
}
