# BBS Air Physics

A Rust mod (a DLL) for **Kingdom Hearts Birth by Sleep Final Mix** (PC, Steam build) that adds a
Config-menu setting to switch the player's air physics between three games:

- **BBS**: Birth by Sleep's own air physics, unchanged.
- **KH1**: Kingdom Hearts Final Mix's air physics, 1:1.
- **KH2**: Kingdom Hearts II Final Mix's air physics, 1:1.

The setting applies to Terra, Ventus and Aqua. KH1's and KH2's values run inside BBS's own
movement code (native), not in a second physics system beside it.

Work happens on the `testing` branch. Testers talk to the assistant through
`docs/tester-requests.md` and the thread files in `testers/`.

Nothing from the games (exes, extracted files, decompiled code) is in this repo.
