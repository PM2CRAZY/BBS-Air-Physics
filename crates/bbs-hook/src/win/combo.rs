//! Combo Master's effect for every character (the owner: "give everyone combo master for right
//! now"; the ini's `combo_master`).
//!
//! BBS's combo-continue check `0x14021C140` (Checked (decomp)), for an attack (`+0x5E0` 1),
//! goes on to the next swing only when the deck's queued flag (`+0x390` → `+0x60` bit `0x1000`)
//! is set or the last swing hit (player `+0x320` bit `0x20`, set by the hit handler
//! `0x140293D40`); otherwise it ends the combo: `test r10b, 20h; je` at `0x14021C376`. Combo
//! Master lets a combo go on after a miss, so the `je` (`74 33` at `0x14021C37A`, Checked (exe))
//! is made two NOPs while the setting is on, and put back when it's off. Assumption: the
//! ability's own check wasn't found (it may be in `0x14021E490`, which Ghidra couldn't
//! decompile); this gives its effect, not the ability's menu entry.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const MISS_ENDS_COMBO: usize = 0x21C37A;
const ORIGINAL: [u8; 2] = [0x74, 0x33];
const PATCHED: [u8; 2] = [0x90, 0x90];

static BASE: AtomicUsize = AtomicUsize::new(0);
static ON: AtomicBool = AtomicBool::new(false);

/// Checks the bytes are BBS's (or ours); false: not this exe, nothing done.
pub unsafe fn install(base: usize) -> bool {
    let at = base + MISS_ENDS_COMBO;
    if super::hook::bytes_are(at, &ORIGINAL) || super::hook::bytes_are(at, &PATCHED) {
        BASE.store(base, Ordering::Relaxed);
        true
    } else {
        super::log(&format!("Combo Master: unexpected bytes at {at:#x}, left alone"));
        false
    }
}

/// Puts the setting in (or takes it out); logs a change.
pub unsafe fn apply(on: bool) {
    let base = BASE.load(Ordering::Relaxed);
    if base == 0 || ON.load(Ordering::Relaxed) == on {
        return;
    }
    let at = base + MISS_ENDS_COMBO;
    let (from, to) = if on { (ORIGINAL, PATCHED) } else { (PATCHED, ORIGINAL) };
    if super::hook::patch(at, &from, &to) {
        ON.store(on, Ordering::Relaxed);
        super::log(&format!("Combo Master for everyone: {}", if on { "on" } else { "off" }));
    }
}
