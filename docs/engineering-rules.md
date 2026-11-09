# Engineering rules

Detail behind the hard rules in `design-philosophy-instructions.md`.

## Steam addresses only
Use addresses from the Steam exes only (BBS Final Mix, KH1 Final Mix, KH2 Final Mix in
`KINGDOM HEARTS -HD 1.5+2.5 ReMIX-`). Never reuse addresses from the Epic build, the PSP
original or a PS4 build. The OG Rust rewrite's addresses are Epic: use its logic and values, and
find the Steam address by matching the logic in the Steam decomp. New addresses are plain
constants, used only when the build is Steam. Every new hook checks the function's exact first
bytes (at least 5, read from the Steam exe) and refuses to install if they don't match.

Steam exes the decomps were made from (SHA-256):
- BBS FM  `KINGDOM HEARTS Birth by Sleep FINAL MIX.exe` 9,346,320 bytes
  `375a811243f1f95f786f00318e2e1210dd5881b25bbf0912a0ad8f81650c976c`
- KH1 FM  `KINGDOM HEARTS FINAL MIX.exe` 7,490,832 bytes
  `d790746245d26159f3ee0e1060e33b2fa2de06941850a4ac724f598722884bac`
- KH2 FM  `KINGDOM HEARTS II FINAL MIX.exe` 8,659,216 bytes
  `9002b2de6a1f91a790bd0673de125d1cf833f7942bfec827cdcf6ba64d5849ed`

## No decompiled code or game files in the repo
Decomp findings go in as plain-language notes (address, what it does, how sure we are), never as
pasted decompiler output. Ghidra databases, decomp exports and game files stay outside the repo
(`local/` is ignored, and the .gitignore is a whitelist).

## Don't break BBS's story and events
Before changing a movement function, check what reads its results (events, scripts, minigames,
the Command Board, Mirage Arena, per-character content) and say what you found.

## Performance
- Say whether a design adds hooks, per-frame work or duplicate state on top of BBS, and its cost.
- Avoid per-frame allocations, redundant hooks and heavy tracing in normal play.
- Measure, don't guess.

## Keep the docs current
When something is verified, record it in the matching doc in this folder: the address, what it
does, and a tag: Checked (exe / decomp / files / trace) or Assumption.

## Versioning
Any change under crates/ bumps VERSION in `crates/bbs-hook/src/lib.rs` (t + next number) and adds
a matching `### <date>: Stage <n> (testing branch: <title>)` entry at the top of AGENTS.md
(`python3 tools/testers/version.py` does both). Docs-only changes aren't versions. One system per
version, no edits to unrelated code.
