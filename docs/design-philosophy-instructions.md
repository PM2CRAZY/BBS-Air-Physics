# Core instructions

Short core block. Detail lives in `engineering-rules.md` and `systems-to-port.md` in this folder.
Read them when a task touches them.

```
PROJECT GOAL
Add an in-game Config setting to Kingdom Hearts Birth by Sleep Final Mix (PC, Steam) that switches
the player's air physics between BBS, KH1 and KH2 (all Rust). With the setting on KH1 the air
physics must feel 1:1 with KH1 Final Mix; on KH2, 1:1 with KH2 Final Mix; on BBS, exactly like BBS.
It applies to Terra, Ventus and Aqua unless a tester or the owner says otherwise.

NATIVE ALWAYS
BBS's own systems own the player character. Native means giving BBS's engine more abilities so
KH1's and KH2's air physics run 1:1 inside it: new values in its tables, new states in its state /
action tables, new functions or wrappers patched into BBS's own movement functions. BBS then steps
the jump, the fall and the landing itself.

Not native: our own physics step running beside BBS's and copying positions or velocities into
BBS's fields every frame, or a KH1/KH2 controller taking the character. That code is "running
along". If something can only run along for now, say so in the thread and ask before shipping it.
Never offer hybrid versions and never ask which version to build.

BUILD WHAT BBS LACKS
If KH1 or KH2 has an air-physics feature BBS lacks, build it into BBS's engine without asking each
time. Ask only about open design choices, and before choosing between native designs that change
how it plays.

1:1 TARGET
- With the setting on KH1 or KH2, every value matches that game's exe exactly: jump speed, gravity,
  fall-speed cap, air steering, momentum, timings. No approximating.
- Confirm BBS's tick rate and frame step in its decomp before converting frame counts. Don't
  assume it matches KH1's or KH2's, and don't assume it differs.
- A feature is 1:1 when the tester's trace matches the source game frame for frame, or the source
  game's decomp confirms the numbers. Say which you checked and what you couldn't.
- If a real native attempt still can't match, stop and ask: what differs, why, the native options.

BBS STAYS WHOLE
- With the setting on BBS, nothing changes: every character's own jumps, Glide, Doubleflight,
  air dodges, air Command Styles, Shotlocks, D-Links and minigames behave exactly as in BBS.
- Whatever BBS's events, scripts and cutscenes read from the movement code, they still get.
- BBS's, KH1's and KH2's paths share the same native code wherever they can.
- A choice that strongly affects feel becomes a Config option, not a silent change.

LITTLE OF OURS PER FRAME
Prefer data BBS's own code reads (values, table rows) over per-frame hooks that steer it. Measure
before proposing a performance fix; don't trade native integration for a small speed win without
asking.

SOURCES OF TRUTH
- BBS: the Steam BBS Final Mix decomp and exe.
- KH2: the OG Rust rewrite first (OG-KH2-Combat-Rust-Rewrite), then the Steam KH2 FM decomp.
- KH1: the Steam KH1 FM decomp (and the KH2-Combat-in-KH1 repo's notes, which are checked
  findings about KH1's movement on the same exe).
- Constants, tables and prologue bytes: the Steam exes, checked against the decomp.
- What the tester sees: their traces and logs are ground truth.
- Mark every assumption as an assumption and check it. Never present a guess as fact.

WORKFLOW
- No code until a tester (or the owner) writes "go ahead" after the assistant's latest message.
- One system per version. Any change under crates/ is a version (t + next number).
- Work only on the testing branch. Never commit to main or any other branch.
```
