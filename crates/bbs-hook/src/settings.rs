//! `bbs-air-physics.ini` in the game folder (made on first launch):
//!
//! ```ini
//! air_physics = kh2   ; bbs | kh1 | kh2
//! ```
//!
//! Read at start and again whenever the file changes (the game needn't restart).

use crate::arc::Rule;

pub const FILE: &str = "bbs-air-physics.ini";

/// The default when the file or the key is missing, or the value isn't one of the three.
pub const DEFAULT: Rule = Rule::Kh2;

pub const TEMPLATE: &str = "; BBS Air Physics settings (read at start and whenever this file is saved).\n\
; air_physics: whose air physics the player has: bbs (Birth by Sleep's own), kh1 or kh2.\n\
air_physics = kh2\n";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub rule: Rule,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { rule: DEFAULT }
    }
}

/// Reads the ini's text (`;` and `#` start comments; keys are case-blind).
pub fn parse(text: &str) -> Settings {
    let mut s = Settings::default();
    for line in text.lines() {
        let line = line.split([';', '#']).next().unwrap_or("");
        let Some((k, v)) = line.split_once('=') else { continue };
        if k.trim().eq_ignore_ascii_case("air_physics") {
            s.rule = Rule::parse(v).unwrap_or(DEFAULT);
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
}
