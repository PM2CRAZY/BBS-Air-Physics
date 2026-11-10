# AGENTS.md

Version log for the testing branch, newest first. Every change under `crates/` is a version: it
bumps VERSION in `crates/bbs-hook/src/lib.rs` (t + the next number) and adds an entry at the top
of this list (`python3 tools/testers/version.py` does both). Each entry quotes the tester's ask
and gives the addresses, the change, and Checked / Assumption tags.

Read first, every session: `docs/design-philosophy-instructions.md`, `docs/engineering-rules.md`,
`docs/tester-requests.md`, then the top entries below.

### 2026-11-09: Stage 2 (testing branch: High Jump is the chosen game's High Jump)
The owner, testing t1: "I feel like nothing much has changed". Their log: every jump was command `0xFA` (BBS's rise, then KH2's fall).

- **Found:** `0xFA` is High Jump (Checked (exe): its command row at `0x140811080 + 0xFA × 0x1E` gives 100 / 114 / 123 / 132 / 143 % jump speed by level and +2.25 max rise; Jump `0xF9` is 100 %). With it equipped it is the only ground jump, so t1's "other jumps rise as BBS's" took every jump.
- **Change:** a ground jump (motion `+0x422` 9 or 0xC, Checked (decomp) `0x1402645F0`) is the game's: Jump its jump, High Jump its High Jump by BBS's level `+0x34E`: KH2 235 / 310 / 385 / 585 (Checked (files): `03system.bin` `fmab`), KH1 290 (Checked (trace): KH2-Combat-in-KH1's log). Holdable (early release) as KH2's button jumps. A fall after an air move goes on from the arc he had.
- **Assumptions:** BBS's High Jump level 0 counts as level 1; KH1's High Jump is one level.

### 2026-11-09: Stage 1 (testing branch: KH1 / KH2 jump and fall arcs in BBS's air state)
The owner: "I want to create a mod for Birth by Sleep that adds a setting in game to switch between the air physics of KH1, KH2, and default BBS"; units 1 (a) a fixed ratio checked by model heights; 2 (b) BBS's air moves stay BBS's, their fall is the chosen game's.

- **Loading:** a DLL Panacea loads from the mod's `bbs\dll\` (`mod.yml`), acting only in BBS's exe; log `bbs-air-physics.log`, setting `air_physics = bbs | kh1 | kh2` in `bbs-air-physics.ini` (made on first launch, KH2 by default, reread when saved). The Config-menu row is the next version.
- **Hook:** BBS's player air update `0x140263030` (first bytes `48 8B C4 48 89 70 10`, Checked (exe)), wrapped; jump button `0x140272670` (`40 53 48 83 EC 20`).
- **Change:** the plain Jump (`0xF9`) rises and falls on the chosen game's arc, its speed handed to BBS's own air update in BBS's phases 5 (rising) / 4 (falling) (Checked (decomp): `0x140263DB0` classes them so, and the update leaves their speed alone); every fall is the game's; other jumps rise as BBS's and fall as the game's. Steering, take-off, landing, ledges and air moves stay BBS's.
- **Numbers:** KH2 g 20/49, cap 16, jump 185, 30 Hz tick, release hop on a tick past frame 6 (Checked (exe), KH2-Combat-in-KH1's notes; the 115 tapped reproduced in the tests). KH1 g 0.48469388, cap 20, jump 190, every frame, release at 6+ (Checked (decomp): `0x1402AF960`, `0x1402AF6A0`, `0x1402A4730`, `0x1402A7B30`). 1 BBS unit = 100 KH units (Checked (files): 102.623 / 1.026229).
- **Assumptions:** Sora's KH1 jump is the stats default 190; a ceiling ends the rise in a fall from rest; normal play runs BBS's 60 fps mode (step 1).

