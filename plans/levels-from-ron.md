# Plan: Load distinct levels (map + spawns) from RON

Status: **refined, ready to execute** (decisions: per-level files, floor-only tiles, baseline reproduces today's behavior, coordinates are oddr `col`/`row`).

## Objective
Load distinct levels — map layout (hex field) + player/enemy spawn list — from RON files in `assets/data/levels/`, replacing the hard-coded `dummy_hex()` map and 4 hard-coded actor spawns. First run must be visually and behaviorally identical to today.

## Decisions (made with user)
1. One RON file per level: `assets/data/levels/*.ron`.
2. Floor-only tiles this iteration. No `TileType::Wall`/obstacles. Note: `TileType` currently only has `Floor` (`src/combat/map.rs:230`), so `"floor"` is the only valid tile value.
3. Baseline file reproduces today's behavior exactly (111-tile field + current spawns).
4. **Coordinates are oddr `col`/`row`, NOT axial `q`/`r`.** RON fields `col`/`row` are oddr; `HexMap::from_level` converts via `MapPos::from_oddr(col, row)` (`src/combat/map.rs:29-33`). The baseline table below is unchanged from the first draft (it already lists oddr column ranges).
5. **Loader suffix = `"ron"`** (not full filenames). Verified in bevy_asset 0.19: type-based lookup wins (`server/loaders.rs::find` returns the single registered loader for an `Asset` type without extension checks); `get_full_extension("level-01.ron") == "ron"` matches anyway; bevy 0.19 has no default `.ron` loader, so no conflict with existing per-type full-filename suffixes (`main.actors.ron`, …).
6. **`SpawnSpec` gains `name` + `player: bool`** so RON is the single source of truth for display names and team assignment (today hard-coded in `setup_actors`, `src/combat/mod.rs:108-145`).
7. **`Levels: BTreeMap<String, Level>`** keyed by file stem — deterministic order, trivially supports N levels. Combat setup uses the first (lexicographic) level.
8. **`Level` derives `Serialize`** in addition to `Debug, Clone, Deserialize, Asset, TypePath` (convention: `src/core/types.rs:35`) for the RON round-trip test. `serde` and `ron = "0.12"` are already dependencies.

## Current state (baseline to migrate)
- `src/combat/mod.rs::combat_plugin` (line 35) — on `OnEnter(GameState::Combat)` runs `setup_map` → `setup_camera` → `setup_actors`, then `setup_combat_flow` + `setup_generators`.
- `setup_map` calls `map::dummy_hex()` (`src/combat/map.rs:155`): 11-row hex field, 111 tiles, all `TileType::Floor` (line 178 `"." => Some(TileType::Floor)`), `camera_focus` = oddr(5,5), scroll limits = focus ± 500. `HexMap` fields: `tiles: HashMap<MapPos, TileType>`, `obstacles: HashMap<MapPos, Obstacle>` (empty, TODO), `camera_focus`, `scroll_limit_x/y`.
- `setup_actors`: team "Player" (is_player = true, `TeamBundle::new("Player", true)` at line 109) + "CPU" (false); `player_leader` @ oddr(5,5) as `"Player (Leader)"`, `player_tank` @ oddr(5,6) as `"Player (Tank)"`, `sucker` @ oddr(2,2) as `"Sucker #1"`, `sucker` @ oddr(8,2) as `"Sucker #2"`; spawned via `ActorGenerator` template names; then `GameDeck(Deck::new_rnd())` (line 153).
- Loading pattern (`src/assets.rs`): `DataAssetLoader<T>` (generic RON asset loader, uses `ron::de`). `assets_plugin`: `app.init_asset::<T>()` + `.register_asset_loader(DataAssetLoader::<T>::new(<suffix>))` per type. `load_data_files` (on `OnEnter(GameState::Start)`): spawns `Loading + AssetHandle(asset_server.load::<T>("data/main.X.ron").untyped())` per file. `check_asset_loading_state` (Update, in Start): polls each `Loading` entity's load state; all `Loaded` → `AllAssetsLoadedEvent` → `handle_all_assets_loaded_event` inserts derived resources and sets `GameState::MainMenu`.

## Data model (new `src/levels.rs`)
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct Level {
    pub map: MapSpec,
    pub spawns: Vec<SpawnSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSpec {
    pub tiles: Vec<TileSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSpec {
    /// Oddr grid coordinate.
    pub col: u32,
    pub row: u32,
    /// TileType name; currently only "floor".
    #[serde(default = "default_floor")]
    pub tile: String,
}

fn default_floor() -> String { "floor".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnSpec {
    /// Key in assets/data/main.actors.ron.
    pub actor: String,
    pub col: u32,
    pub row: u32,
    /// Display name (Name component).
    pub name: String,
    /// true -> "Player" team, false -> "CPU" team.
    pub player: bool,
}

#[derive(Resource)]
pub struct Levels {
    /// Keyed by file stem (name without ".ron"), insertion order = sorted filenames.
    pub levels: BTreeMap<String, Level>,
}

#[derive(Resource)]
pub struct LevelHandles {
    pub handles: BTreeMap<String, UntypedHandle>,
}
```
- `actor` keys reference `assets/data/main.actors.ron` (keys: `player_leader`, `player_tank`, `sucker`).
- `tile` values are `TileType` variant names (currently only `"floor"`); sprite mapping happens in the existing tile->sprite lookup.

## Implementation steps

### 1. `src/levels.rs` (new module)
Types above. Add `mod levels;` to crate root `src/main.rs`.

### 2. `src/assets.rs` — multi-file level loading
- `assets_plugin`: add `app.init_asset::<Level>()` to the init chain (~line 100) and `.register_asset_loader(DataAssetLoader::<Level>::new("ron"))` to the loader registrations.
- `load_data_files`: after the existing spawn files:
  - `std::fs::read_dir("assets/data/levels")` → keep regular files ending in `.ron`, collect names, sort lexicographically. If the dir is missing or empty, insert `LevelHandles` with an empty map (fail loudly instead at `setup_map`).
  - For each sorted name: spawn `Loading + AssetHandle(asset_server.load::<Level>(format!("data/levels/{name}")).untyped()))` (same pattern as existing spawns).
  - Insert `LevelHandles { handles: BTreeMap<stem, UntypedHandle> }` (stem = name minus `.ron`), preserving sorted order.
- `check_asset_loading_state`: **no change** — level entities are just more `Loading` assets; the existing gate still blocks `MainMenu` until all load.
- `handle_all_assets_loaded_event`: before `next_state.set(GameState::MainMenu)`, read `Res<Assets<Level>>`, fetch each handle from `Res<LevelHandles>`, insert `Levels { levels }`.

### 3. `src/combat/map.rs` — `HexMap::from_level`
```rust
impl HexMap {
    pub fn from_level(level: &Level) -> Self {
        let mut tiles = HashMap::new();
        let (min_c, max_c, min_r, max_r) = level.map.tiles.iter()
            .fold((i32::MAX, i32::MIN, i32::MAX, i32::MIN), |acc, t| { ... });
        for t in &level.map.tiles {
            let pos = MapPos::from_oddr(t.col, t.row);
            let tt = match t.tile.as_str() {
                "floor" => TileType::Floor,
                other => panic!("unknown tile \"{other}\" in level"),
            };
            tiles.insert(pos, tt);
        }
        Self {
            tiles,
            obstacles: HashMap::new(),
            camera_focus: MapPos::from_oddr((min_c + max_c) / 2, (min_r + max_r) / 2),
            scroll_limit_x: 500.0,
            scroll_limit_y: 500.0,
        }
    }
}
```
(Exact field names/types of `HexMap` and the scroll-limit constants to be copied from `dummy_hex()` at `src/combat/map.rs:155-184` during implementation.) Keep `dummy_hex()` in place — the equivalence test needs both.

### 4. `src/combat/mod.rs` — wiring
- `setup_map(mut commands, levels: Res<Levels>)` → take the first level (`levels.levels.iter().next().expect("no level loaded")`) → `commands.insert_resource(HexMap::from_level(level))`.
- `setup_actors(mut commands, actor_generator, levels)`: spawn the "Player" (is_player = true) and "CPU" (false) teams as today; iterate `level.spawns` for the first level and spawn template `spec.actor` at `MapPos::from_oddr(spec.col, spec.row)` with `Name::new(spec.name)`, assigned to the `Player`/`CPU` team per `spec.player`; unknown actor key → fail loudly at combat start. Keep `GameDeck(Deck::new_rnd())` insertion.
- `setup_camera`, `setup_combat_flow`, `setup_generators`: unchanged.

### 5. Data — `assets/data/levels/level-01.ron` (baseline)
Encodes today's `dummy_hex()` field (111 tiles, all `tile: "floor"`), oddr row layout (row r, col ranges — unchanged from first draft; verify against the raw grid in `dummy_hex()` at `src/combat/map.rs:155-170` during implementation):
- r=0: col 2..=9 (8 tiles)
- r=1: col 1..=9 (9)
- r=2: col 1..=10 (10)
- r=3: col 0..=10 (11)
- r=4: col 0..=11 (12)
- r=5: col 0..=10 (11)
- r=6: col 0..=11 (12)
- r=7: col 0..=10 (11)
- r=8: col 1..=10 (10)
- r=9: col 1..=9 (9)
- r=10: col 2..=9 (8)

Spawns (reproducing `setup_actors`):
- `player_leader` @ (5,5), name "Player (Leader)", player: true
- `player_tank` @ (5,6), name "Player (Tank)", player: true
- `sucker` @ (2,2), name "Sucker #1", player: false
- `sucker` @ (8,2), name "Sucker #2", player: false

## Tests
New module tests (`src/levels.rs` and/or `src/combat/map.rs`):
- RON round-trip: `ron::ser::to_string::<Level>` then `ron::de::from_str::<Level>` on a literal of the baseline level → all fields equal.
- `HexMap::from_level(baseline)` == `dummy_hex()`: same tile set, same `camera_focus`, same scroll limits; existing `test_map_can_determine_neighbor_tiles` neighbor-ring count (18) still holds.
- Spawn equivalence: 4 spawns with the same actor keys, (col, row), display names, and player flags as today's `setup_actors`.

Full check: `cargo build`, `cargo test`; then `cargo run` and confirm combat looks/behaves exactly as before (same map, same 4 named actors at the same positions, same deck).

## Out of scope (next iterations)
- Level selection UI in the start menu (replace "Press any key" with a level picker; introduce a `SelectedLevel` resource).
- `TileType::Wall` / `MapSpec.obstacles` + tile-texture rendering (sprites `wall-1`, `barricade-NW` already exist).

## Gotchas
- Bevy asset root is `assets/`; `fs::read_dir` uses relative project-root paths (`assets/data/levels`), `AssetServer` paths are `data/levels/{name}`.
- `TileType` has only the `Floor` variant (`src/combat/map.rs:230`) — `"floor"` is the only valid `tile` value today.
- Dev profile: `[profile.dev] opt-level=1, debug=0, strip=debuginfo` — don't expect a normal debug build of app code.
- `src_old/`, `assets_old/` are dead code (gitignored) — do not edit or port from.
- Sprite PNGs are gitignored; `assets/images/compile-sprites/` is empty.

## File checklist
- [ ] `src/levels.rs` (new): `Level` (with `Serialize`), `MapSpec`, `TileSpec`, `SpawnSpec` (name + player), `Levels`, `LevelHandles` + tests
- [ ] `src/main.rs`: `mod levels;`
- [ ] `src/assets.rs`: `init_asset::<Level>()`, `DataAssetLoader::<Level>::new("ron")`, `load_data_files` dir scan, `LevelHandles` resource, collect into `Levels` in `handle_all_assets_loaded_event`
- [ ] `src/combat/map.rs`: `HexMap::from_level` (odd→via `MapPos::from_oddr`) + equivalence test
- [ ] `src/combat/mod.rs`: `setup_map` / `setup_actors` take `Res<Levels>`, drop hard-codes, keep team names / display names from RON
- [ ] `assets/data/levels/level-01.ron`
- [ ] `cargo build` + `cargo test` + `cargo run` manual check
