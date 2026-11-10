//! Windows side: loading, the log, the ini, and the hooks.
//!
//! Panacea (OpenKH's mod loader, `DBGHELP.dll` in the game folder) loads every DLL in
//! `<mods>\bbs\dll\` inside Birth by Sleep only, then calls `OnInit`. The work starts in
//! `DllMain`, and only in BBS's exe (the folder is shared with KH1, KH2 and Re:CoM).

mod air;
mod combo;
mod hook;
mod sys;

use crate::arc::Rule;
use crate::settings;
use std::ffi::c_void;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use sys::*;

pub const BBS_EXE: &str = "KINGDOM HEARTS Birth by Sleep FINAL MIX.exe";

/// The Steam exe's image base (Ghidra's addresses are this plus the RVA).
pub const IMAGE_BASE: usize = 0x1_4000_0000;

static LOG: Mutex<Option<std::fs::File>> = Mutex::new(None);

/// The rule now (`Rule as u8`: 0 BBS, 1 KH1, 2 KH2).
static RULE: AtomicU8 = AtomicU8::new(2);

pub fn rule() -> Rule {
    match RULE.load(Ordering::Relaxed) {
        0 => Rule::Bbs,
        1 => Rule::Kh1,
        _ => Rule::Kh2,
    }
}

fn set_rule(r: Rule) {
    RULE.store(
        match r {
            Rule::Bbs => 0,
            Rule::Kh1 => 1,
            Rule::Kh2 => 2,
        },
        Ordering::Relaxed,
    );
}

pub fn log(line: &str) {
    if let Ok(mut g) = LOG.lock() {
        if let Some(f) = g.as_mut() {
            let _ = writeln!(f, "{line}");
            let _ = f.flush();
        }
    }
}

fn exe_path() -> String {
    let mut buf = [0u16; 1024];
    let n = unsafe { GetModuleFileNameW(std::ptr::null_mut(), buf.as_mut_ptr(), buf.len() as u32) } as usize;
    String::from_utf16_lossy(&buf[..n.min(buf.len())])
}

fn exe_dir() -> String {
    let p = exe_path();
    p.rsplit_once('\\').map(|(d, _)| d.to_string()).unwrap_or(p)
}

fn ini_path() -> String {
    format!("{}\\{}", exe_dir(), settings::FILE)
}

/// Reads the ini (making it on first launch) and applies it; logs a change.
fn load_settings(first: bool) {
    let path = ini_path();
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => {
            let _ = std::fs::write(&path, settings::TEMPLATE);
            settings::TEMPLATE.to_string()
        }
    };
    let s = settings::parse(&text);
    let was = rule();
    set_rule(s.rule);
    unsafe { combo::apply(s.combo_master) };
    if first {
        log(&format!("air physics: {}", s.rule.name()));
    } else if was != s.rule {
        log(&format!("air physics: {} (was {}; the ini changed)", s.rule.name(), was.name()));
    }
}

fn ini_stamp() -> Option<std::time::SystemTime> {
    std::fs::metadata(ini_path()).and_then(|m| m.modified()).ok()
}

/// Rereads the ini when it's saved (twice a second, only its timestamp).
unsafe extern "system" fn watch(_: *mut c_void) -> u32 {
    let mut stamp = ini_stamp();
    loop {
        Sleep(500);
        let now = ini_stamp();
        if now != stamp {
            stamp = now;
            load_settings(false);
        }
    }
}

/// Panacea's entry point (the work started in `DllMain`; this only notes the route).
#[no_mangle]
pub unsafe extern "C" fn OnInit(mod_path: *const u16) {
    let mut s = String::new();
    if !mod_path.is_null() {
        let n = (0..1024).take_while(|&i| *mod_path.add(i) != 0).count();
        s = String::from_utf16_lossy(std::slice::from_raw_parts(mod_path, n));
    }
    log(&format!("loaded by Panacea (mod folder {s})"));
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(_inst: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason != 1 {
        return 1;
    }
    let exe = exe_path();
    if !exe.rsplit('\\').next().is_some_and(|n| n.eq_ignore_ascii_case(BBS_EXE)) {
        return 1;
    }
    let path = format!("{}\\bbs-air-physics.log", exe_dir());
    let _ = std::fs::rename(&path, format!("{}\\bbs-air-physics.prev.log", exe_dir()));
    if let Ok(f) = OpenOptions::new().create(true).write(true).truncate(true).open(&path) {
        *LOG.lock().unwrap_or_else(|e| e.into_inner()) = Some(f);
    }
    log(&format!("BBS Air Physics {} loaded in {exe}", crate::VERSION));
    std::panic::set_hook(Box::new(|info| log(&format!("PANIC: {info}"))));
    let base = GetModuleHandleA(std::ptr::null()) as usize;
    combo::install(base);
    load_settings(true);
    if air::install(base) {
        CreateThread(std::ptr::null_mut(), 0, watch, std::ptr::null_mut(), 0, std::ptr::null_mut());
    } else {
        log("not the Steam BBS Final Mix exe this mod knows: nothing was changed");
    }
    1
}
