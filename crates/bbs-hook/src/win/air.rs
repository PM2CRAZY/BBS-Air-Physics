//! KH1's and KH2's jump arcs inside BBS's own player air state.
//!
//! BBS's air update `0x140263030 (player)` (Checked (decomp)) runs states 4 (jump) and 5 (fall)
//! through phases at player `+0x4E8`: 1 the take-off wait, 2 rising and 3 / 6 falling with its
//! own vertical speed (`+0x50C` / `+0x510`, gravity `+0x508` from `+0x32C`), and 4 / 5 with the
//! vertical speed left as it is (BBS's air-phase classifier `0x140263DB0`: 5 rising, 4
//! falling). In every phase but 1 and 7 it then runs BBS's air actions (`0x1402176B0`), writes
//! `+0x510` into the movement's y velocity (`+0x80` → `+0xB4`), steers, and in the falling
//! phases checks the landing and ledges.
//!
//! With the rule at KH1 or KH2, a jump or fall is put in phase 5 (rising) or 4 (falling) and
//! handed the speed of that game's arc each frame (`arc::Arc`): BBS's own update does the rest.
//! Jump and High Jump (the ground jumps) are the game's own; BBS's other air moves (Doubleflight,
//! air commands) rise as BBS's and fall as the game's (the owner: question 2 (b)). The take-off
//! wait is BBS's (its jump motion). Steering stays BBS's.

use super::hook;
use crate::arc::{Arc, Params, Rule};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// BBS's player air update (states 4 and 5). Steam RVA.
const AIR_UPDATE: usize = 0x263030;
/// `mov rax, rsp; mov [rax+10h], rsi` (whole instructions, position-independent).
const AIR_UPDATE_START: [u8; 7] = [0x48, 0x8B, 0xC4, 0x48, 0x89, 0x70, 0x10];
/// Whether the jump button is held: `0x140272670 (pad + 0x58)` with the player's pad at
/// `+0x360` (BBS's own jump uses it in its hold, `0x140263030`). Checked (decomp).
const JUMP_HELD: usize = 0x272670;
const JUMP_HELD_START: [u8; 6] = [0x40, 0x53, 0x48, 0x83, 0xEC, 0x20];

/// High Jump (player `+0x34C`, level `+0x34E`): its command row (`0x140811080 + id × 0x1E`)
/// gives 100 / 114 / 123 / 132 / 143 % of the jump speed by level and +2.25 max rise; Jump
/// `0xF9` is 100 %. Checked (exe) (the rows), (decomp) (`0x1402645F0`), (trace) (the owner's
/// log: every jump was `0xFA`).
const CMD_HIGH_JUMP: u16 = 0xFA;
/// The jump motions `0x1402645F0` starts a ground jump with (9 Jump, 0xC the others; an air
/// jump is 0x3E), at player `+0x422`. Checked (decomp).
const MOTION: usize = 0x422;
const LEVEL: usize = 0x34E;
const FLAGS: usize = 0x318;
/// Player flag set by `0x1402645F0`'s special jump (`0x14025DBD0` = 4): BBS's own.
const SPECIAL_JUMP: u32 = 0x1000;

// Player fields (Checked (decomp)).
const STATE: usize = 0x304;
const STEP: usize = 0x328;
const GRAVITY: usize = 0x32C;
const COMMAND: usize = 0x34C;
const PAD: usize = 0x360;
const PPM: usize = 0x370;
const PHASE: usize = 0x4E8;
const VY: usize = 0x50C;
const VY_OUT: usize = 0x510;
const MOVEMENT: usize = 0x80;
// Movement fields: flags (`0x8040000`: a ceiling, as BBS's own rise reads it), y velocity.
const MV_FLAGS: usize = 0x170;
const MV_VY: usize = 0xB4;
/// `PPM +8` times this is BBS's gravity before its modifiers (`0x140220550`).
const BBS_G_PER_PPM: f32 = 0.008_166_667;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static BASE: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug)]
enum Mode {
    /// BBS's own rise (a jump other than the plain one); ours from its apex.
    BbsRise,
    /// The chosen game's arc, in phase 5 / 4.
    Arc(Arc),
}

#[derive(Clone, Copy, Debug)]
struct Track {
    player: usize,
    rule: Rule,
    mode: Mode,
    /// The vertical speed we last handed BBS (a different one is BBS's own launch).
    #[allow(dead_code)]
    handed: f32,
}

static TRACK: Mutex<Option<Track>> = Mutex::new(None);

unsafe fn rd<T: Copy>(a: usize) -> T {
    std::ptr::read_unaligned(a as *const T)
}
unsafe fn wr<T: Copy>(a: usize, v: T) {
    std::ptr::write_unaligned(a as *mut T, v)
}

pub unsafe fn install(base: usize) -> bool {
    BASE.store(base, Ordering::Relaxed);
    if !hook::bytes_are(base + JUMP_HELD, &JUMP_HELD_START) {
        super::log(&format!("jump-button check at {:#x}: unexpected bytes", base + JUMP_HELD));
        return false;
    }
    hook::install(base + AIR_UPDATE, &AIR_UPDATE_START, air_update as *const () as usize, &ORIGINAL, "BBS's player air update")
}

unsafe fn original(p: usize) {
    let f: unsafe extern "C" fn(usize) = std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
    f(p)
}

unsafe fn jump_held(p: usize) -> bool {
    let pad: usize = rd(p + PAD);
    if pad == 0 {
        return true;
    }
    let f: unsafe extern "C" fn(usize) -> i32 = std::mem::transmute(BASE.load(Ordering::Relaxed) + JUMP_HELD);
    f(pad + 0x58) != 0
}

/// The game's numbers with BBS's gravity modifiers on this player (its script coefficient, the
/// halving status `0x140220550` applies): KH's gravity times BBS's gravity now over BBS's
/// unmodified one (1 normally).
unsafe fn params(p: usize, rule: Rule) -> Option<Params> {
    let mut par = rule.params()?;
    let ppm: usize = rd(p + PPM);
    if ppm != 0 {
        let k = rd::<f32>(ppm + 8) * BBS_G_PER_PPM;
        let m = -rd::<f32>(p + GRAVITY) / k;
        if k > 0.0 && m.is_finite() && m > 0.0 && m <= 8.0 {
            par.gravity *= m;
        }
    }
    Some(par)
}

/// Writes this frame's vertical speed where BBS's air update reads it, and the phase that
/// leaves it alone (5 rising, 4 falling).
unsafe fn hand_over(p: usize, vy: f32, movement_too: bool) {
    wr(p + VY, vy);
    wr(p + VY_OUT, vy);
    wr::<i32>(p + PHASE, if vy > 0.0 { 5 } else { 4 });
    let mv: usize = rd(p + MOVEMENT);
    if movement_too && mv != 0 {
        wr(mv + MV_VY, vy);
    }
}

unsafe fn step(p: usize, arc: &mut Arc, par: &Params) -> f32 {
    arc.gravity = par.gravity;
    if arc.rising() {
        let mv: usize = rd(p + MOVEMENT);
        if mv != 0 && rd::<u32>(mv + MV_FLAGS) & 0x804_0000 != 0 && arc.t > 0.0 {
            arc.bonk(par);
        } else {
            arc.release(jump_held(p), par);
        }
    }
    let dt: f32 = rd(p + STEP);
    arc.step(if dt.is_finite() && dt > 0.0 { dt } else { 1.0 })
}

/// BBS's take-off just ended (phase 1 to 2): Jump and High Jump get the game's own; a fall goes
/// on from the arc he had (an air attack ending) or starts from rest; an air jump rises as BBS's.
unsafe fn lift_off(p: usize, rule: Rule, par: &Params, before: Option<Track>) -> Track {
    let state: i32 = rd(p + STATE);
    let cmd: u16 = rd(p + COMMAND);
    let ground = state == 4 && matches!(rd::<u16>(p + MOTION), 9 | 0xC) && rd::<u32>(p + FLAGS) & SPECIAL_JUMP == 0;
    let mode = if ground || state == 5 {
        let mut arc = if ground && cmd == CMD_HIGH_JUMP {
            Arc::high_jump(rd(p + LEVEL), par)
        } else if ground {
            Arc::jump(par)
        } else {
            match before.map(|t| t.mode) {
                Some(Mode::Arc(a)) => a,
                _ => Arc::falling(0.0, par),
            }
        };
        let vy = step(p, &mut arc, par);
        hand_over(p, vy, true);
        if ground {
            super::log(&format!("jump: command {cmd:#x} level {}, {} arc {:.0} high, apex in {:.1} frames", rd::<u8>(p + LEVEL), rule.name(), arc.height * crate::arc::KH_PER_BBS, arc.uptime));
        }
        Mode::Arc(arc)
    } else {
        super::log(&format!("jump: command {cmd:#x} (air jump), BBS's rise, then {}'s fall", rule.name()));
        Mode::BbsRise
    };
    Track { player: p, rule, mode, handed: rd(p + VY) }
}

unsafe extern "C" fn air_update(p: usize) {
    let rule = super::rule();
    let mut g = TRACK.lock().unwrap_or_else(|e| e.into_inner());
    if g.is_some_and(|t| t.player != p || t.rule != rule) {
        // Another player, or the ini changed mid-air: BBS's own phase back if it was ours.
        if let Some(t) = g.take() {
            if t.player == p && matches!(t.mode, Mode::Arc(_)) && matches!(rd::<i32>(p + PHASE), 4 | 5) {
                wr::<i32>(p + PHASE, if rd::<f32>(p + VY) > 0.0 { 2 } else { 3 });
            }
        }
    }
    let state: i32 = rd(p + STATE);
    let Some(par) = params(p, rule) else {
        *g = None;
        drop(g);
        return original(p);
    };
    if state != 4 && state != 5 {
        *g = None;
        drop(g);
        return original(p);
    }
    let phase: i32 = rd(p + PHASE);
    match (phase, g.map(|t| t.mode)) {
        (1, _) => {
            let before = g.take();
            original(p);
            if matches!(rd::<i32>(p + STATE), 4 | 5) && rd::<i32>(p + PHASE) == 2 {
                *g = Some(lift_off(p, rule, &par, before));
            }
        }
        (2 | 3 | 6, Some(Mode::BbsRise)) | (2, None) if rd::<f32>(p + VY) > 0.0 => {
            *g = Some(Track { player: p, rule, mode: Mode::BbsRise, handed: 0.0 });
            original(p);
            // BBS's rise is over at its apex: the game's fall from there.
            if matches!(rd::<i32>(p + STATE), 4 | 5) && matches!(rd::<i32>(p + PHASE), 2 | 3 | 6) && rd::<f32>(p + VY) <= 0.0 {
                let down = -rd::<f32>(p + VY);
                *g = Some(Track { player: p, rule, mode: Mode::Arc(Arc::falling(down, &par)), handed: -down });
                wr::<i32>(p + PHASE, 4);
            } else if !matches!(rd::<i32>(p + PHASE), 2 | 3 | 6) {
                *g = None;
            }
        }
        (2 | 3 | 6, _) => {
            // A fall BBS runs (an air move ended, or one we didn't see begin): the game's.
            let down = (-rd::<f32>(p + VY)).max(0.0);
            let mut arc = Arc::falling(down, &par);
            let vy = step(p, &mut arc, &par);
            hand_over(p, vy, false);
            *g = Some(Track { player: p, rule, mode: Mode::Arc(arc), handed: vy });
            original(p);
            after_arc(p, &mut g);
        }
        (4 | 5, Some(Mode::Arc(mut arc))) => {
            let vy = step(p, &mut arc, &par);
            hand_over(p, vy, false);
            *g = Some(Track { player: p, rule, mode: Mode::Arc(arc), handed: vy });
            original(p);
            after_arc(p, &mut g);
        }
        _ => {
            // BBS's own phases (the landing 7, its glide-like 4 out of `0x140264200`, ...).
            *g = None;
            drop(g);
            original(p);
        }
    }
}

/// After BBS's update ran on our phase: anything that moved him on (a landing, a ledge, an air
/// move, a new jump) ends our arc.
unsafe fn after_arc(p: usize, g: &mut Option<Track>) {
    if !matches!(rd::<i32>(p + STATE), 4 | 5) || !matches!(rd::<i32>(p + PHASE), 4 | 5) {
        *g = None;
    }
}

