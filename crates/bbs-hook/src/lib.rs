//! BBS Air Physics: an in-game choice of air physics for Kingdom Hearts Birth by Sleep Final Mix
//! (PC, Steam): BBS's own, KH1's or KH2's.
//!
//! OpenKH's Panacea loads this DLL from the mod's `bbs\dll\` folder inside BBS only. It extends
//! BBS's own player air state (`win::air`): KH1's and KH2's jump arcs run in BBS's air phases,
//! and BBS still steers, lands, grabs ledges and starts air moves.

#![allow(clippy::missing_safety_doc)]

pub mod arc;
pub mod settings;

/// The mod's version (the log, the Config menu): `t` + the stage of the latest `AGENTS.md`
/// entry, bumped with every change to the source.
pub const VERSION: &str = "t3";

#[cfg(all(windows, target_arch = "x86_64"))]
mod win;

#[cfg(all(windows, target_arch = "x86_64"))]
pub use win::*;

#[cfg(test)]
mod tests {
    #[test]
    fn version_matches_the_latest_agents_entry() {
        let agents = include_str!("../../../AGENTS.md");
        let top = agents.lines().find(|l| l.starts_with("### ")).unwrap_or("");
        let stage = top.split("Stage ").nth(1).and_then(|s| s.split(' ').next()).unwrap_or("");
        assert_eq!(format!("t{stage}"), super::VERSION, "AGENTS.md's top entry is {top:?}");
    }
}
