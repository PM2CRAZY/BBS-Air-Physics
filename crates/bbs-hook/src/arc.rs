//! KH1's and KH2's jump and fall arcs, in BBS's units, as plain numbers (no game memory here).
//!
//! Both games keep a jump as a curve of height against air time (60 Hz frames), not as a
//! velocity plus gravity: rising, `y = h · sin(π/2 · sin(π/2 · t/T))` with `T = √(2h/g)`; after
//! the apex, `y = h − g/2 · (t − T)²`, each frame's drop capped. BBS's air state is handed the
//! vertical speed this curve gives each frame (`win::air`).
//!
//! Sources (docs/bbs-air-physics.md):
//! - KH2: `movement-model.md` of the KH2-Combat-in-KH1 repo, Checked (exe): g = 20/49, fall cap
//!   16 a frame, Sora's jump 185, velocities recomputed on KH2's 30 Hz tick (every second
//!   frame); early release: after 6 frames of rise and more than 5 before the apex, a hop of
//!   g × 12.5 from where he is.
//! - KH1: Steam KH1 FM decomp, Checked (decomp): g = 0.48469388 (`0x1402AF960`), a frame's drop
//!   capped at 20 (`0x1402AF6A0`), jump height 190 (the stats block's default, `0x1402A4730`),
//!   stepped every frame; early release (`0x1402A7B30`): the same rule, a launch at speed 5.
//! - Units: KH's are centimetres, BBS's metres. KH2 Sora's root bone stands 102.623 high in his
//!   rest pose, Ventus's `center` joint 1.026229: 100.0001. Checked (files).

/// KH units per BBS unit. Checked (files): KH2 Sora's rest-pose root height over Ventus's.
pub const KH_PER_BBS: f32 = 100.0;

const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;

/// Which game's air physics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    Bbs,
    Kh1,
    Kh2,
}

impl Rule {
    pub fn name(self) -> &'static str {
        match self {
            Rule::Bbs => "BBS",
            Rule::Kh1 => "KH1",
            Rule::Kh2 => "KH2",
        }
    }

    /// The ini's word for it (`bbs`, `kh1`, `kh2`; anything else: none).
    pub fn parse(s: &str) -> Option<Rule> {
        match s.trim().to_ascii_lowercase().as_str() {
            "bbs" => Some(Rule::Bbs),
            "kh1" => Some(Rule::Kh1),
            "kh2" => Some(Rule::Kh2),
            _ => None,
        }
    }

    /// The game's numbers in BBS units (per 60 Hz frame); `None` for BBS (its own code).
    pub fn params(self) -> Option<Params> {
        match self {
            Rule::Bbs => None,
            Rule::Kh1 => Some(Params::KH1),
            Rule::Kh2 => Some(Params::KH2),
        }
    }
}

/// One game's air numbers, already in BBS units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    /// Gravity, units per 60 Hz frame squared (positive).
    pub gravity: f32,
    /// The fastest fall, units per 60 Hz frame.
    pub fall_max: f32,
    /// The plain jump's height.
    pub jump: f32,
    /// Frames between recomputing the vertical speed (KH2's 30 Hz tick: 2; KH1: every frame).
    pub tick: f32,
}

impl Params {
    pub const KH2: Params = Params { gravity: 20.0 / 49.0 / KH_PER_BBS, fall_max: 16.0 / KH_PER_BBS, jump: 185.0 / KH_PER_BBS, tick: 2.0 };
    pub const KH1: Params = Params { gravity: 0.484_693_88 / KH_PER_BBS, fall_max: 20.0 / KH_PER_BBS, jump: 190.0 / KH_PER_BBS, tick: 1.0 };
}

/// Early release: only after this many frames of rise...
pub const HOLD_MIN: f32 = 6.0;
/// ...and while the apex is more than this many frames away.
pub const HOLD_APEX_MARGIN: f32 = 5.0;
/// The hop that replaces the rest of the rise: a launch whose apex is this many frames away
/// (KH1 launches at speed 5, `T = 5`; KH2's hop is `g × 0.5 × 5 × 5` high: the same).
pub const RELEASE_HOP_T: f32 = 5.0;

/// The rise's shape, 0..1 over 0..1 of the rise.
fn sinsin(s: f32) -> f32 {
    (HALF_PI * (HALF_PI * s).sin()).sin()
}

/// An arc: a rise of `height` over `uptime` frames (0: none, a fall), then a fall at `gravity`.
/// `t` is the air time on it (60 Hz frames). Heights are relative to where it started.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arc {
    pub height: f32,
    pub uptime: f32,
    pub t: f32,
    pub gravity: f32,
    pub fall_max: f32,
    /// Frames between speed recomputes, and the speed and frames left of the current one.
    pub tick: f32,
    tick_left: f32,
    tick_vy: f32,
    /// A button jump: releasing jump early cuts it (`HOLD_MIN`, `HOLD_APEX_MARGIN`).
    pub holdable: bool,
    /// The jump button was let go since the jump began.
    pub released: bool,
}

impl Arc {
    /// A rise of `height` (KH1's launch `0x1402AF960`, KH2's jump): `T = √(2h/g)`.
    pub fn rise(height: f32, p: &Params) -> Arc {
        let h = height.max(0.0);
        let uptime = if h > 0.0 { (2.0 * h / p.gravity).sqrt() } else { 0.0 };
        Arc { height: h, uptime, t: 0.0, gravity: p.gravity, fall_max: p.fall_max, tick: p.tick, tick_left: 0.0, tick_vy: 0.0, holdable: false, released: false }
    }

    /// The plain jump (holdable).
    pub fn jump(p: &Params) -> Arc {
        Arc { holdable: true, ..Arc::rise(p.jump, p) }
    }

    /// Falling, already going down at `down` units a frame (0: from rest): the fall's apex is
    /// put `down / g` frames back (KH1's `Arc::falling`).
    pub fn falling(down: f32, p: &Params) -> Arc {
        let tau = down.clamp(0.0, p.fall_max) / p.gravity;
        Arc { t: tau, ..Arc::rise(0.0, p) }
    }

    pub fn rising(&self) -> bool {
        self.t < self.uptime
    }

    /// Height at air time `t`.
    pub fn y(&self, t: f32) -> f32 {
        if t < self.uptime {
            self.height * sinsin(t / self.uptime)
        } else {
            let d = t - self.uptime;
            self.height - 0.5 * self.gravity * d * d
        }
    }

    /// The vertical speed for the next `dt` frames (up positive, per 60 Hz frame), and the air
    /// time on by `dt`. Recomputed every `tick` frames over a whole tick and held between
    /// (KH2's 30 Hz tick); a fall's drop is capped at `fall_max` a frame.
    pub fn step(&mut self, dt: f32) -> f32 {
        let dt = dt.max(0.0);
        if self.tick_left <= 1e-4 {
            let span = self.tick.max(dt).max(1e-3);
            let dy = self.y(self.t + span) - self.y(self.t);
            self.tick_vy = (dy / span).max(-self.fall_max);
            self.tick_left = span;
        }
        self.tick_left -= dt;
        self.t += dt;
        self.tick_vy
    }

    /// Early release (KH1 `0x1402A7B30`, KH2's same rule on its tick): with the button up (latched
    /// once let go), `HOLD_MIN` frames into the rise and the apex more than `HOLD_APEX_MARGIN`
    /// away, the rest of the rise becomes a hop of `RELEASE_HOP_T` frames from here. Past that
    /// margin the window is shut for good. Returns whether it cut.
    pub fn release(&mut self, held: bool, p: &Params) -> bool {
        if !self.holdable {
            return false;
        }
        if !held {
            self.released = true;
        }
        if self.uptime <= self.t + HOLD_APEX_MARGIN {
            self.holdable = false;
            return false;
        }
        // KH1 checks every frame from `HOLD_MIN` on (`6.0 <= t`); KH2 only as a tick starts and
        // once past it (`t > 6`: its first chance is frame 8, the "about 115 tapped").
        let ticked = self.tick <= 1.0 || self.tick_left <= 1e-4;
        let due = if self.tick <= 1.0 { self.t >= HOLD_MIN } else { self.t > HOLD_MIN };
        if self.released && ticked && due {
            let hop = 0.5 * p.gravity * RELEASE_HOP_T * RELEASE_HOP_T;
            *self = Arc { gravity: self.gravity, ..Arc::rise(hop, p) };
            return true;
        }
        false
    }

    /// A ceiling stopped the rise: falling from rest from here. (Assumption: neither KH game's
    /// own ceiling rule was read.)
    pub fn bonk(&mut self, p: &Params) {
        *self = Arc::falling(0.0, p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flies an arc at 60 fps from 0 until it comes back to 0; returns (apex, frame of apex,
    /// frames in the air).
    fn fly(mut a: Arc, held_for: f32, p: &Params) -> (f32, u32, u32) {
        let (mut y, mut apex, mut at, mut f) = (0.0f32, 0.0f32, 0u32, 0u32);
        loop {
            a.release(a.t < held_for, p);
            y += a.step(1.0);
            f += 1;
            if y > apex {
                apex = y;
                at = f;
            }
            if y <= 0.0 || f > 400 {
                return (apex, at, f);
            }
        }
    }

    #[test]
    fn kh2_jump_is_185_held_with_its_apex_at_30_frames_and_lands_at_61() {
        let p = Params::KH2;
        let (apex, at, land) = fly(Arc::jump(&p), 1e9, &p);
        assert!((apex * KH_PER_BBS - 185.0).abs() < 0.5, "{apex}");
        // T = sqrt(2*185/(20/49)) = 30.1; KH2's own trace: takeoff frame 4, apex 33, landing
        // 65 (the takeoff frames are BBS's own motion here).
        assert!((29..=32).contains(&at), "{at}");
        assert!((59..=63).contains(&land), "{land}");
    }

    #[test]
    fn kh2_tapped_jump_cuts_to_a_small_hop_near_115() {
        let p = Params::KH2;
        let (apex, ..) = fly(Arc::jump(&p), 1.0, &p);
        // KH2 on the user's install: "about 115 tapped".
        assert!((100.0..130.0).contains(&(apex * KH_PER_BBS)), "{}", apex * KH_PER_BBS);
    }

    #[test]
    fn kh1_jump_is_190_and_falls_capped_at_20() {
        let p = Params::KH1;
        let (apex, ..) = fly(Arc::jump(&p), 1e9, &p);
        assert!((apex * KH_PER_BBS - 190.0).abs() < 0.5, "{apex}");
        let mut a = Arc::falling(0.0, &p);
        for _ in 0..200 {
            a.step(1.0);
        }
        assert!((a.step(1.0) * KH_PER_BBS + 20.0).abs() < 1e-3);
    }

    #[test]
    fn kh2_fall_cap_is_16_and_held_over_a_tick() {
        let p = Params::KH2;
        let mut a = Arc::falling(0.0, &p);
        for _ in 0..200 {
            a.step(1.0);
        }
        let (v1, v2) = (a.step(1.0), a.step(1.0));
        assert!((v1 * KH_PER_BBS + 16.0).abs() < 1e-3 && v1 == v2);
        // Two frames of one tick hold one speed; KH1 recomputes every frame.
        let mut b = Arc::falling(0.0, &p);
        let (s1, s2, s3) = (b.step(1.0), b.step(1.0), b.step(1.0));
        assert!(s1 == s2 && s2 != s3);
    }

    #[test]
    fn thirty_fps_covers_the_same_arc() {
        let p = Params::KH2;
        let (mut a, mut b) = (Arc::jump(&p), Arc::jump(&p));
        let (mut ya, mut yb) = (0.0, 0.0);
        for _ in 0..20 {
            ya += a.step(2.0) * 2.0;
            yb += b.step(1.0);
            yb += b.step(1.0);
        }
        assert!((ya - yb).abs() < 1e-4);
    }

    #[test]
    fn falling_from_speed_keeps_the_speed() {
        let p = Params::KH1;
        let mut a = Arc::falling(0.05, &p);
        let v = a.step(1.0);
        assert!((v + 0.05).abs() < p.gravity * 1.01, "{v}");
    }

    #[test]
    fn rules_parse() {
        assert_eq!(Rule::parse(" KH1 "), Some(Rule::Kh1));
        assert_eq!(Rule::parse("bbs"), Some(Rule::Bbs));
        assert_eq!(Rule::parse("x"), None);
    }
}
