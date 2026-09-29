# AGENTS.md

Roguelike game in **Rust** using **Bevy 0.19** (single crate, edition 2024). "A roguelike version of Onslaught! Arena."

## Layout
- **`src/`** — live code. All changes go here.
  - `main.rs` — entrypoint; defines the `GameState` enum (`Start`/`MainMenu`/`Combat`) and registers all plugins (`assets`, `animations`, `start`, `combat`).
  - `core/` — rules engine: `cards`, `checks`, `feats`, `keywords`, `types`.
  - `combat/` — turn-based combat: `actor`, `ai`, `commands`, `flow`, `generator`, `map`, `fx`/`combat_fx`, `ui`.
  - `start/` — start/menu screen.
  - `assets.rs` — loads `.ron` data and sprites (`DataAssetLoader`).
- **`src_old/`, `assets_old/`** — **dead code** from a prior architecture rewrite (gitignored). Do **not** edit or port from here.
- **`assets/`** — runtime assets. `assets/data/*.ron` (`main.actors.ron`, `main.feats.ron`, `main.maneuvers.ron`) are the live game data; plus fonts and images.

## Build & test
```bash
cargo build   # compile
cargo test    # unit tests live in src/combat/generator.rs, src/core/keywords.rs, src/combat/map.rs
cargo run     # launch the game
```

## Gotchas
- **Bevy `dynamic_linking`**: the Bevy dependency enables this feature (`features = ["dynamic_linking"]`). Keep it; check Bevy docs before changing native/graphics deps.
- **Dev profile strips your code**: `[profile.dev]` sets `opt-level = 1`, `debug = 0`, `strip = "debuginfo"`, `split-debuginfo = "unpacked"` (app code optimized + no debug info). Deps run at `opt-level = 3` via `[profile.dev.package."*"]`. Don't assume a normal debug build when debugging *your* code.
- **CI SDL2 is legacy**: `.github/workflows/rust.yml` apt-installs `libsdl2-dev`/`-image`/`-ttf`. Leftover from the pre-Bevy SDL2 era; Bevy 0.19 uses wgpu, so these are likely unnecessary (harmless to the build).
- **`make-backgrounds.sh`** needs **ImageMagick** (`magick montage`) to tile `team-background-*.png` → `team-backgrounds.png`.
