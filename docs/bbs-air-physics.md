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
  `PL::CPlayerVentus` / `CPlayerAqua` / `CPlayerTerra`. The player update `0x140220550` runs
  one function per state (player `+0x304`, set by `0x14021FF60`): 4 jump and 5 fall (with 10,
  11) are `0x140263030`, the air update.
- **Checked (decomp), (files).** Per-character numbers come from `PPM_VENT` / `PPM_AQUA` /
  `PPM_TERR` (64 bytes each, in `arc\pc\p01init.arc` / `p02init.arc` / `p03init.arc`), loaded by
  `0x14021A7F0` into player `+0x370`. All three have `+8` 0.6 (gravity factor), `+0xC` 27.6
  (jump speed), `+0x10` 1.75 (max rise). `+0` / `+4` differ (Ventus 1.3 / 6.2, Aqua 1.3 / 6.2,
  Terra 1.5 / 6.5), and `+0x1C..+0x28` (Ventus 0.2, 0.7, 1.0, −1.5; Aqua 0.3, 0.75, 1.05, −1.6;
  Terra 0.4, 0.8, 1.1, −1.7); `+0x20` / `+0x24` / `+0x28` are the ledge grab's reach.
- **Checked (decomp).** Each frame `0x140220550` sets the player's step `+0x328` (entity
  `+0x20`), jump multiplier `+0x330 = PPM[8] × 0.008166667` and gravity `+0x32C = −PPM[8] ×
  0.008166667 × the global coefficient`, halved while status `0x1B1` is on.
- **Checked (decomp).** The jump starts in `0x1402645F0 (player, kind)`: kind 0 a jump (state 4;
  speed `+0x504 = PPM[0xC]` for the plain Jump command `0xF9` in `+0x34C`, else the command
  row's level value × `PPM[0xC]` / 100; max rise `+0x524`), 2 / 3 an air jump (state 4), others
  a fall (state 5, speed 0). Phase `+0x4E8` = 1.
- **Checked (decomp).** The air update's phases: 1 waits one frame on the ground, then
  `+0x50C = +0x504 × +0x330` and phase 2. Phase 2 rises: past the max rise or at a ceiling
  (movement flags `0x8040000`) the speed halves; speed `+= gravity × step`; with jump let go
  after 6 frames it halves and goes to 3; held, it hangs at speed 0 until 13 / 18 frames
  (motions 9 / 0xC). Phases 3 / 6 fall. 4 / 5 leave the speed alone (the classifier
  `0x140263DB0`: 5 rising, 4 falling; BBS's own phase 4 is a glide-type move from
  `0x140264200`). Then (all but 1 and 7): air actions `0x1402176B0`, y velocity = `+0x510`,
  steering (`+0x51C` eases to the run speed × 0.8 / 60 at 0.05 a step), and in 3 / 4 / 6 / 8 the
  landing (`0x140264A10`) and the ledge grab. The jump button is `0x140272670 (pad + 0x58)`,
  pad at `+0x360`.
- **Checked (decomp).** The movement step adds the movement's own gravity only with its flag
  `0x100`; the player's air update sets the y velocity itself each frame.
- So BBS's plain jump: 27.6 × 0.0049 = 0.135 a frame up, gravity 0.0049, about 1.8 high (cut
  at 1.75), apex near frame 28.

## 4. KH1's and KH2's air physics

Both are per 60 Hz frame, in centimetres. **Checked (files):** KH2 Sora's root bone stands
102.623 high in his rest pose (`obj\P_EX100.mdlx`), Ventus's `center` joint 1.026229 (`p01ex00.pmo`
in `arc\pc\p01base.arc`): 100.0001. So **1 BBS unit = 100 KH units** (Aqua 1.107, Terra 1.154).

**KH2** (`movement-model.md`, Checked (exe)):
- Rise: `y = h · sin(π/2 · sin(π/2 · t / T))`, `T = √(2h / g)`, `g = 20/49` per frame².
- Fall: a parabola at `g` from the apex, each tick's drop capped at 16 a frame.
- Speeds recomputed on the 30 Hz tick (every second frame) and held between.
- Early release: on a tick more than 6 frames into the rise and more than 5 before the apex,
  the rest of the rise becomes a hop `g × 0.5 × 5 × 5` high. The first such tick is frame 8:
  85.9 + 5.1, the "about 115 tapped". Held: 185.
- The take-off is the jump motion's "Jump" event (frame 4 for Sora): part of KH2's animation.

**KH1** (Steam KH1 FM decomp, Checked (decomp)):
- The same rise curve; `g = 0.48469388` (`0x1402AF960`); a frame's drop capped at 20
  (`0x1402AF6A0`); stepped every frame (times the frame scale).
- Jump height: the stats block's `+0x10`, default **190** (`0x1402A4730`, which also sets walk 2,
  run 8, air acceleration 0.1, air drag 0.95). Assumption: Sora's isn't changed from the default
  outside High Jump (KH2-Combat-in-KH1's log: 290 with High Jump).
- Take-off: 4 frames (`0x1402AFDA0`).
- Early release (`0x1402A7B30`): every frame with the button let go, 6 or more frames into the
  rise and the apex more than 5 away: a launch at speed 5 (a hop `g × 12.5` high). Within 5 of the
  apex the window shuts.
- Air steering (`0x1402AF6A0`): stick held, `v += direction × 0.1 × move speed`, its length kept
  to the larger of its old length and the run speed; let go, `v × 0.95` (per 30 Hz frame, its
  square root at 60 fps).

## 5. The setting (t1)

`win::air` wraps BBS's air update `0x140263030` (first bytes `48 8B C4 48 89 70 10`). With
`air_physics` at KH1 or KH2:
- **The plain Jump** (`0xF9`): when BBS's take-off ends, the game's arc (`arc::Arc`): its
  height, curve, gravity, fall cap, tick and early release (the button through `0x140272670`).
  Each frame the arc's speed goes into `+0x50C` / `+0x510` and the phase is 5 (rising) or 4
  (falling); BBS's own update then does everything else.
- **Falls** (walking off, after BBS's air moves, and any fall we find BBS running): the game's
  fall, from the speed BBS had.
- **High Jump** (t2): command `0xFA` (level at player `+0x34E`): the game's High Jump, KH2's
  LV1 235 / LV2 310 / LV3 385 / MAX 585 (`fmab`, Checked (files)) for BBS's levels 1 / 2 / 3 / 4
  (level 0 as 1: Assumption), KH1's 290 at every level. A ground jump is told by its motion
  (`+0x422` 9 or 0xC; an air jump is 0x3E). The owner's first log: every jump was `0xFA`
  (Ventus with High Jump), so t1 changed only the fall.
- **Other jumps** (Doubleflight, air jumps): BBS's rise, the game's fall from its apex (the
  owner: 2 (b)). BBS's apex hang ends there.
- A fall that follows an air attack goes on from the arc he had (t2).
- **Air attacks** (t3): BBS's shared vertical step `0x14021CB00 (player, frame, window end,
  flags)` (Checked (decomp); first bytes `48 83 EC 18 48 8B 91 80 00 00 00`) adds gravity × step
  to `+0x50C`, writes it to the movement, and with the air-attack flag (`+0x318` bit `0x800000`)
  uses gravity × 0.16 and holds the speed at 0 while the swing's frame is before its window end:
  BBS's hover. With KH1 / KH2, while airborne (movement flags `0x2010000` clear) with that flag,
  the step gives the game's arc instead: the arc he jumped or fell on goes on through the swings;
  a positive speed BBS wrote that isn't ours (a swing's hop) starts a rise as high as BBS's would
  go (`v² / 2g` at BBS's gravity) on the game's curve. Assumption: KH1's and KH2's own per-swing
  hops aren't used yet.
- **Kept BBS's:** the take-off wait (BBS's jump motion), steering, landing, ledges, every air
  move. BBS's gravity modifiers (script coefficient, status `0x1B1`) scale the game's gravity.
- **Assumption:** at a ceiling the rise ends in a fall from rest (neither game's ceiling rule was
  read).

## 6. Combo Master for everyone (t4)

- **Checked (decomp).** BBS's combo-continue check `0x14021C140`, for an attack (`+0x5E0` 1),
  goes on only when the deck's queued flag (`+0x390` → `+0x60` bit `0x1000`) is set or the last
  swing hit (player `+0x320` bit `0x20`, set by the hit handler `0x140293D40`); else the combo
  ends (`test r10b, 20h` at `0x14021C376`, `je` `74 33` at `0x14021C37A`, Checked (exe)).
- The ini's `combo_master = on` (default, the owner: "give everyone combo master for right now")
  makes that `je` two NOPs: every combo goes on after a miss, Combo Master's effect. `off` puts
  the bytes back. Live.
- **Checked (decomp).** Abilities: support abilities `0x1C4..0x1E1` count per player at
  `+0x4A3 + slot` (`0x140221900`; slot = command table `0x140814900 + id × 0x18`, byte 7).
  `0x1D3` Combo Plus and `0x1D4` Air Combo Plus (the combo counter `0x1402239B0` adds them to the
  combo's length, ground / air); `0x1D1` Finish Boost (the damage step `0x1402211B0`).
- **Not found:** Combo Master's own id and check (Assumption: next to the combo counter, perhaps
  `0x1D5`, or in `0x14021E490`, which Ghidra couldn't decompile).
