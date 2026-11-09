# Air physics in BBS, KH1 and KH2 (findings)

Plain-language notes only. Addresses are Steam (image base `0x140000000`). Tags: **Checked
(decomp)**, **Checked (exe)**, **Checked (trace)**, **Assumption**.

Sources: the Steam BBS FM decomp made 2026-11-09 (Ghidra 12.1.4, `local\decomps\steam\bbs`), the
Steam KH1 / KH2 decomps (`steam\kh1`, `steam\kh2`), and the KH2-Combat-in-KH1 repo's checked notes
(`docs/kh1-physics.md` for KH1, `docs/movement-model.md` for KH2).

## 1. BBS's time step

- **Checked (decomp).** The frame clock (`0x1400E3460`, once a frame) counts the vsyncs since the
  last game frame. A vsync callback (`0x1400E3A50`) adds one per vsync, at 59.94 Hz. The count,
  capped (default 6, set by `0x1400E39E0` to an even number), becomes the global frame step
  `0x14086A3FC` (read by `0x1400E3870`, script `GetVsyncFrameRate`).
- **Checked (decomp).** `0x1400E1DA0` looks a name up in a table of modes and picks 60 fps or
  30 fps mode. In 30 fps mode the clock waits two vsyncs a frame and the minimum step is 2
  (`0x14080E150` = 2.0); in 60 fps mode it is 1.0.
- So **BBS's movement values are per 60 Hz frame**, scaled by the frame step: the same time unit
  as KH1's (per 60 Hz frame times its frame scale) and KH2's (per 60 Hz frame, velocities
  recomputed on a 30 Hz tick). No frame-count conversion is needed between them, only the
  distance scale (section 4).
- **Assumption, to check in a trace:** normal field and battle play on PC runs in 60 fps mode
  (step 1). Which areas or minigames use 30 fps mode is not read yet.
- Each entity keeps its own step at entity `+0x20` (script `GetFrameRate`, `0x1402B8570`), which
  is what its movement steps by.

## 2. BBS's movement object (`Entity::CMovement`)

Every entity's movement object is at entity `+0x80`. Constructor `0x1401E19D0`. **Checked
(decomp)** unless noted.

| Field | What | Default |
|---|---|---|
| `+0x30` | position (x, y, z), y up | |
| `+0xA0` | acceleration (x, y, z) | |
| `+0xB0` | velocity (x, y, z), units per 60 Hz frame | |
| `+0x170` | flags; bits 0..2 pick the step function; bit `0x100` gravity on | |
| `+0x184` | gravity, per 60 Hz frame squared | −0.0081667 (−0.49 / 60) |
| `+0x188` | fall-speed cap (lowest y velocity) | −0.75 |
| `+0x18C` | (not read yet) | 0.75 |
| `+0x190` | (not read yet) | 0.5 |
| `+0x194` | gravity scale (per entity) | 1.0 |
| `+0x198`, `+0x19C` | movement scales (per entity) | 1.0, 1.0 |
| `+0x1B8` | owner entity (its `+0x20` is the frame step) | |

Globals: `0x140817A90` gravity coefficient for every entity (1.0; script
`Get/SetGravityCoefficient`, `0x1401E39C0` / `0x1401E39F0`), `0x140817A94` global movement
scale (1.0).

**The step table** `0x140817A98`: six step functions, picked by flag bits 0..2, called once a
frame with the frame step (by the movement update `0x1401E3B90`):

| Index | Function | What |
|---|---|---|
| 0 | `0x1401E12B0` | the ballistic step (below) |
| 1..4 | `0x1401E1800`, `0x1401E1860`, `0x1401E18D0`, `0x1401E1950` | not read yet (`0x1401E1950` eases toward a target with a cosine) |
| 5 | `0x1401E1600` | not read yet |

The next slot (`0x140817AC8`) is other data, so a new movement kind needs the table moved
(section 5).

**The ballistic step `0x1401E12B0`** (Checked (decomp)): with gravity on, the y acceleration gets
`gravity × gravity scale × global coefficient`; velocity += acceleration × step; y velocity is
held at the fall cap; position += (velocity × step + acceleration × step² / 2 + this frame's
pushes) × the movement scales. The jump is a plain parabola from start to landing.

**Jump launch helpers** (Checked (decomp)): `0x1401E3890` (speed for a height at an angle:
√(−2 g h) / sin), `0x1401E3840` (speed for a distance at an angle), `0x1401E38E0` (speed and angle
to reach a point). The player's own jumps don't use them; they set the y velocity directly
(below).

Script functions that touch it (pointers in the script table at `0x14064DAF0`..):
`SetGravity` `0x1402B8AD0` (writes `+0x184`), `SetGravityCoefficient` `0x1402B8B20`,
`GetGravityCoefficient` `0x1402BE0E0`, `EnableGravity` `0x1402B8E10`, `EndZeroGravity`
`0x1402B8110`, `GetJumpStandPower` `0x1402C25D0`, `SetJumpStand` `0x1402C2380`. Events and
scripts use these: whatever we change must keep them working.

## 3. BBS's player jump

- **Checked (decomp).** The player classes are `PL::CPlayer` (vtable `0x140646D48`) and
  `PL::CPlayerVentus` / `CPlayerAqua` / `CPlayerTerra`. Their per-character movement numbers
  are a game file, **`PPM_VENT`, `PPM_AQUA`, `PPM_TERR`** (names at `0x140819670`), loaded by
  `0x14021A7F0` into player `+0x370` (the pointer `PPM` below).
  `PAT_*` files load next to them.
- **Checked (decomp).** A jump sets the take-off y speed (player `+0x504`, written into the
  movement's y velocity) from **`PPM +0xC`**, times modifiers: player `+0x330` (a multiplier),
  and per move a factor (×1.65 for one action, ×2 for another, `(level × 0.25 + 0.75)` for one
  with levels; `0x140224310`, `0x1402552D0`, `0x14025E8F0`). Which action is the plain jump, and which are
  High Jump / Doubleflight / air moves, is **not read yet**.
- Other `PPM` fields read by the player code: `+0x10`, `+0x14`, `+0x1C`, `+0x20`, `+0x24`,
  `+0x28`, `+0x2C` (meaning not read yet; likely walk / run / air speeds, **Assumption**).
- The values in `PPM_*` come from the game's data, not the exe. **Not read yet:** they need
  the player's files or a log of them at runtime.

## 4. KH1's and KH2's air physics (from the KH2-Combat-in-KH1 notes)

Both are per 60 Hz frame. Their distance unit is the same family (KH2 Sora's hips 88.5, KH1
Sora's 78.9 units high).

**KH2** (`movement-model.md`, Checked (exe)):
- Rise: `y = h · sin(π/2 · sin(π/2 · t / T))`, `T = √(2h / g)`, `g = 20/49` per frame².
- Fall: a parabola at `g` from the apex, each tick's drop capped at 16 a frame.
- Held jump: released after 6+ frames of rise and 5+ frames before the apex cuts the rise to a
  small hop. Sora: about 115 tapped, 185 held.
- Air steering on the 30 Hz tick: with the stick `v = 0.9 v + 0.1 target`, without `v = 0.95 v`.

**KH1** (`kh1-physics.md`, Checked (decomp), Steam addresses there):
- Same rise curve as KH2. Fall at `g = 0.48469388` per frame², capped at 20 a frame.
- Jump height from Sora's stats (High Jump raises the stat).
- Air steering: without the stick, velocity × drag (per 30 Hz frame); with the stick, adds
  `direction × air acceleration × tilt`, its length kept to the larger of its old length and the
  run speed. Walk-off momentum capped at 8. Takeoff waits 4 frames.
- KH1's air acceleration and drag values and Sora's base jump height are runtime stats, not read
  yet.

## 5. What this means for the setting (not decided; see the thread)

- **The rise is different in kind.** BBS's jump is velocity plus constant gravity. KH1's and
  KH2's rise is a height curve against time. A native port gives BBS's movement a new step kind
  (a "KH arc": rise on the curve, then fall with that game's gravity and cap), added to the step
  table at `0x140817A98` (moved to make room), and picked by the player's jump when the setting
  is KH1 or KH2. BBS's own step 0 stays as it is.
- **The distance scale is open.** BBS's gravity (0.0082 a frame²) and fall cap (0.75 a frame) are
  about 50 to 60 times smaller than KH1's and KH2's, so one BBS unit is roughly a metre against
  KH's roughly a centimetre (**Assumption** until characters' heights are compared from the
  files). How KH heights and speeds are converted is a design choice to settle with the testers.
- **What each character keeps** (Glide, Doubleflight, air dodges, air Command Styles) is open
  too.
