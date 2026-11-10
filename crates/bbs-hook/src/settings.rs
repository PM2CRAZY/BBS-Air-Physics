//! `bbs-air-physics.ini` in the game folder (made on first launch):
//!
//! ```ini
//! air_physics = kh2   ; bbs | kh1 | kh2
//! combo_master = on   ; on | off: every character's combo goes on after a miss
//! ```
//!
//! Read at start and again whenever the file changes (the game needn't restart).

use crate::arc::Rule;

pub const FILE: &str = "bbs-air-physics.ini";

/// The default when the file or the key is missing, or the value isn't one of the three.
pub const DEFAULT: Rule = Rule::Kh2;

pub const TEMPLATE: &str = "; BBS Air Physics settings (read at start and whenever this file is saved).\n\
; air_physics: whose air physics the player has: bbs (Birth by Sleep's own), kh1 or kh2.\n\
air_physics = kh2\n\
; combo_master: on gives Terra, Ventus and Aqua Combo Master's effect (a combo goes on when a swing misses).\n\
combo_master = on\n";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub rule: Rule,
    /// Combo Master's effect for everyone (the owner: "give everyone combo master for right
    /// now"); on unless the ini says off.
    pub combo_master: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { rule: DEFAULT, combo_master: true }
    }
}

fn on_off(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "on" | "true" | "1" | "yes" => Some(true),
        "off" | "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

/// Reads the ini's text (`;` and `#` start comments; keys are case-blind).
pub fn parse(text: &str) -> Settings {
    let mut s = Settings::default();
    for line in text.lines() {
        let line = line.split([';', '#']).next().unwrap_or("");
        let Some((k, v)) = line.split_once('=') else { continue };
        let k = k.trim();
        if k.eq_ignore_ascii_case("air_physics") {
            s.rule = Rule::parse(v).unwrap_or(DEFAULT);
        } else if k.eq_ignore_ascii_case("combo_master") {
            s.combo_master = on_off(v).unwrap_or(true);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_rule_and_defaults_to_kh2() {
        assert_eq!(parse("air_physics = kh1 ; KH1").rule, Rule::Kh1);
        assert_eq!(parse("AIR_PHYSICS=BBS").rule, Rule::Bbs);
        assert_eq!(parse("air_physics = what").rule, Rule::Kh2);
        assert_eq!(parse("").rule, Rule::Kh2);
        assert_eq!(parse(TEMPLATE).rule, Rule::Kh2);
    }

    #[test]
    fn combo_master_is_on_unless_off() {
        assert!(parse("").combo_master);
        assert!(parse(TEMPLATE).combo_master);
        assert!(!parse("combo_master = off").combo_master);
        assert!(parse("combo_master = huh").combo_master);
    }
}
