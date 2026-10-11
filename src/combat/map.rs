use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::hash::Hash;

use bevy::prelude::*;
use serde::Deserialize;

const SQRT_3: f32 = 1.7320508076;
const HEX_SIZE: f32 = 64.0;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Component)]
pub struct MapPos {
    q: i32,
    r: i32,
}

impl MapPos {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn as_axial_coordinates(&self) -> (i32, i32) {
        (self.q, self.r)
    }

    pub fn from_axial_coordinates(v: (i32, i32)) -> Self {
        Self::new(v.0, v.1)
    }

    pub fn from_oddr(col: i32, row: i32) -> Self {
        let q = col - (row - (row & 1)) / 2;
        let r = row;
        Self::new(q, r)
    }

    pub fn from_xy(x: f32, y: f32) -> Self {
        let frac_q = (SQRT_3 / 3.0 * x - y / 3.0) / HEX_SIZE;
        let frac_r = (2.0 / 3.0 * y) / HEX_SIZE;
        let frac_s = -frac_q - frac_r;

        Self::round_hex(frac_q, frac_r, frac_s)
    }

    pub fn into_vec3(self) -> Vec3 {
        let q = self.q as f32;
        let r = self.r as f32;
        let x = HEX_SIZE * (SQRT_3 * q + SQRT_3 * r / 2.0);
        let y = HEX_SIZE * 3.0 * r / 2.0;

        Vec3::new(x, y, 0.0)
    }

    fn round_hex(frac_q: f32, frac_r: f32, frac_s: f32) -> Self {
        let mut q = frac_q.round();
        let mut r = frac_r.round();
        let s = frac_s.round();

        let q_diff = f32::abs(q - frac_q);
        let r_diff = f32::abs(r - frac_r);
        let s_diff = f32::abs(s - frac_s);

        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        }

        Self::new(q as i32, r as i32)
    }

    pub fn distance(&self, other: &MapPos) -> i32 {
        let self_s = -self.q - self.r;
        let other_s = -other.q - other.r;

        ((self.q - other.q).abs() + (self.r - other.r).abs() + (self_s - other_s).abs()) / 2
    }

    fn scale(self, factor: i32) -> Self {
        Self::new(self.q * factor, self.r * factor)
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.q + other.q, self.r + other.r)
    }

    fn neighbor(self, direction: usize) -> Self {
        assert!(direction < 6);
        let nd = NEIGHBOR_DELTAS[direction];
        self.add(Self::new(nd.0, nd.1))
    }
    // fn lerp(&self, other: &MapPos, delta: f32) -> Self {
    //     let fraq_q = self.q as f32 + (other.q - self.q) as f32 * delta;
    //     let fraq_r = self.r as f32 + (other.r - self.r) as f32 * delta;
    //     let fraq_s = self.s as f32 + (other.s - self.s) as f32 * delta;
    //     Self::round_hex(fraq_q, fraq_r, fraq_s)
    // }
}

impl From<Vec3> for MapPos {
    fn from(point: Vec3) -> Self {
        MapPos::from_xy(point.x, point.y)
    }
}

impl From<Vec2> for MapPos {
    fn from(point: Vec2) -> Self {
        MapPos::from_xy(point.x, point.y)
    }
}

#[test]
fn test_can_convert_screen_coord_to_hex_and_back() {
    let hex1 = MapPos::new(-1, -2);
    let hex2 = MapPos::new(1, -1);
    let hex3 = MapPos::new(-2, 3);

    assert_eq!(hex1, MapPos::from(hex1.into_vec3()));
    assert_eq!(hex2, MapPos::from(hex2.into_vec3()));
    assert_eq!(hex3, MapPos::from(hex3.into_vec3()));
}

#[derive(Asset, TypePath, Debug, Default, Deserialize)]
pub struct MapSpec {
    grid: Vec<String>,
    legend: BTreeMap<String, TileSpec>,
    focus_pos: (i32, i32),
}

#[derive(Debug, Deserialize)]
pub struct TileSpec {
    tile_type: TileType,
    spawn: Option<SpawnSpec>,
}

#[derive(Debug, Deserialize)]
pub struct SpawnSpec {
    id: String,
}

#[derive(Resource)]
pub struct HexMap {
    tiles: HashMap<MapPos, TileType>,
    obstacles: HashMap<MapPos, Obstacle>,
    spawn_points: HashMap<String, Vec<MapPos>>,

    pub camera_focus: MapPos,
    pub scroll_limit_x: (f32, f32),
    pub scroll_limit_y: (f32, f32),
}

impl HexMap {
    pub fn from_spec(spec: &MapSpec) -> Self {
        let mut tiles = HashMap::new();
        let mut spawn_points = HashMap::new();

        for (row, row_str) in spec.grid.iter().rev().enumerate() {
            for (idx, tile_char) in row_str.chars().enumerate() {
                if tile_char.is_whitespace() {
                    // ignore whitespace
                    continue;
                }

                let col = idx / 2;
                let tile_spec = spec.legend.get(&tile_char.to_string()).unwrap();
                let map_pos = MapPos::from_oddr(col as i32, row as i32);

                if let Some(spawn_spec) = tile_spec.spawn.as_ref() {
                    if !spawn_points.contains_key(&spawn_spec.id) {
                        spawn_points.insert(spawn_spec.id.clone(), vec![]);
                    }

                    spawn_points.get_mut(&spawn_spec.id).unwrap().push(map_pos);
                }

                tiles.insert(map_pos, tile_spec.tile_type);
            }
        }

        let camera_focus = MapPos::from_oddr(spec.focus_pos.0, spec.focus_pos.1); // TODO
        let focus_vec = camera_focus.into_vec3();

        const SCROLL_PADDING: f32 = 500.0;
        Self {
            tiles,
            obstacles: HashMap::new(), // TODO
            spawn_points,
            camera_focus,
            scroll_limit_x: (focus_vec.x - SCROLL_PADDING, focus_vec.x + SCROLL_PADDING),
            scroll_limit_y: (focus_vec.y - SCROLL_PADDING, focus_vec.y + SCROLL_PADDING),
        }
    }

    pub fn tiles(&self) -> impl Iterator<Item = (&MapPos, &TileType)> {
        self.tiles.iter()
    }

    pub fn find_tile(&self, hex: &MapPos) -> Option<Tile> {
        self.tiles
            .get(hex)
            .map(|tt| (*hex, *tt, self.obstacles.get(hex).cloned()))
    }

    pub fn neighbors(&self, center: MapPos, radius: i32) -> impl Iterator<Item = Tile> {
        NeighborTileIter::new(self, center, radius)
    }

    pub fn find_path(&self, from: MapPos, to: MapPos) -> Option<Path> {
        if self.find_tile(&from).is_none() || self.find_tile(&to).is_none() {
            return None;
        }

        find_path_astar(from, to, self)
    }

    pub fn spawn_points(&self, team_ref: &str) -> &Vec<MapPos> {
        self.spawn_points.get(team_ref).as_ref().unwrap()
    }
}

#[rustfmt::skip]
#[cfg(test)]
pub fn dummy_hex() -> HexMap {
    HexMap::from_spec(&MapSpec {
        grid: vec![
            /*        0 1 2 3 4 5 6 7 8 9 */
            /*  0 */ "    . . . . . . . .    ".to_string(),
            /*  1 */ "   . . . . . . . . .   ".to_string(),
            /*  2 */ "  . . . . . . . . . .  ".to_string(),
            /*  3 */ " . . . . . . . . . . . ".to_string(),
            /*  4 */ ". . . . . . . . . . . .".to_string(),
            /*  5 */ " . . . . . . . . . . . ".to_string(),
            /*  6 */ ". . . . . . . . . . . .".to_string(),
            /*  7 */ " . . . . . . . . . . . ".to_string(),
            /*  8 */ "  . . . . . . . . . .  ".to_string(),
            /*  9 */ "   . . . . . . . . .   ".to_string(),
            /* 10 */ "    . . . . . . . .    ".to_string(),
        ],
        legend: BTreeMap::from_iter([(
            ".".to_string(),
            TileSpec {
                tile_type: TileType::Floor,
                spawn: None,
            },
        )]),
        ..Default::default()
    })
}

#[test]
fn test_can_build_hex_map() {
    let map = dummy_hex();
    assert!(map.tiles().count() > 0);
}

#[rustfmt::skip]
#[test]
fn test_map_can_determine_neighbor_tiles() {
    let map = dummy_hex();
    let pos = MapPos::from_oddr(3, 3);
    let neighbors = map
        .neighbors(pos, 2)
        .map(|(p, ..)| (p.q, p.r))
        .collect::<Vec<_>>();

    assert_eq!(18, neighbors.len());
    assert_eq!(vec![
    // first ring
    (1, 4), (2, 4), (3, 3), (3, 2), (2, 2), (1, 3),
    // second ring
    (0, 5), (1, 5), (2, 5), (3, 4), (4, 3), (4, 2),
    (4, 1), (3, 1), (2, 1), (1, 2), (0, 3), (0, 4),
    ], neighbors);
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum TileType {
    Floor,
    // Void,
}

pub type Tile = (MapPos, TileType, Option<Obstacle>);
pub type Path = Vec<MapPos>;

#[derive(Component, Clone, Copy)]
pub struct Obstacle(pub f32);

pub fn update_obstacles_in_map(mut map: ResMut<HexMap>, obstacles_q: Query<(&Obstacle, &MapPos)>) {
    map.obstacles.clear();
    for (o, mpos) in obstacles_q.iter() {
        map.obstacles.insert(*mpos, o.clone());
    }
}

const NEIGHBOR_DELTAS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];
pub struct NeighborTileIter<'a> {
    map: &'a HexMap,
    curr_ring: i32,
    center: MapPos,
    curr_pos: MapPos,
    max_steps: i32,
    steps_taken: i32,
    steps_in_dir: i32,
    steps_in_ring: i32,
    curr_dir: usize,
}

impl<'a> NeighborTileIter<'a> {
    fn new(map: &'a HexMap, center: MapPos, radius: i32) -> Self {
        let max_steps = 3 * radius * (radius + 1);

        Self {
            map,
            center,
            curr_pos: center.neighbor(4),
            max_steps,
            curr_ring: 1,
            steps_taken: 0,
            steps_in_dir: 0,
            steps_in_ring: 1,
            curr_dir: 0,
        }
    }
}

impl<'a> Iterator for NeighborTileIter<'a> {
    type Item = Tile;
    fn next(&mut self) -> Option<Tile> {
        while self.steps_taken < self.max_steps {
            let candidate_tile = self.map.find_tile(&self.curr_pos);
            let hexes_per_ring = 6 * self.curr_ring;

            self.steps_taken += 1;

            if self.steps_in_ring >= hexes_per_ring {
                // We are at the end of the current ring
                // => step up to the next ring
                self.steps_in_ring = 1;
                self.curr_dir = 0;
                self.steps_in_dir = 0;
                self.curr_ring += 1;
                self.curr_pos = self
                    .center
                    .add(MapPos::from_axial_coordinates(NEIGHBOR_DELTAS[4]).scale(self.curr_ring));
            } else {
                // There are still hexes in the current ring to explore
                // => take a step along the current ring and try the next hex
                self.steps_in_ring += 1;

                if self.steps_in_dir < self.curr_ring {
                    // take another step anlog the current edge
                    self.steps_in_dir += 1;
                    self.curr_pos = self.curr_pos.neighbor(self.curr_dir);
                } else {
                    // We are at the end of the current edge
                    // => turn around and walk along the next edge
                    self.steps_in_dir = 1;
                    self.curr_dir += 1;
                    self.curr_pos = self.curr_pos.neighbor(self.curr_dir);
                }
            }

            if candidate_tile.is_some_and(|(_, _, obs)| obs.is_none()) {
                return candidate_tile;
            }
        }
        None
    }
}

struct Node(MapPos, f32);

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }

        match self.1 > other.1 {
            true => Ordering::Less,
            false => Ordering::Greater,
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn find_path_astar(start: MapPos, goal: MapPos, m: &HexMap) -> Option<Path> {
    // println!("find_path_astar from={:?} to={:?}", start, goal);
    let mut costs_so_far: HashMap<MapPos, (f32, Path)> = HashMap::new();
    let mut open: BinaryHeap<Node> = BinaryHeap::new();
    let start_node = Node(start, 0.0);

    open.push(start_node);
    costs_so_far.insert(start, (0.0, vec![start]));

    while let Some(Node(current_pos, _)) = open.pop() {
        let (current_costs, path) = costs_so_far.get(&current_pos).unwrap().clone();

        // println!("  > explore {:?} (costs={})", current_pos, current_costs);
        if current_pos == goal {
            // the current candidat is the goal tile
            // -> return result
            return Some(path.clone());
        } else {
            // the current candidate is not the goal
            // -> ... and look its at its neighbors
            for neighbor_tile @ (neighor_pos, _, _) in m.neighbors(current_pos, 1) {
                let new_costs = current_costs + costs(&neighbor_tile);
                // let new_costs = current_costs + 1.0; // TODO: consider tile type and obstacles
                let costs = costs_so_far.get(&neighor_pos);

                if costs.is_none() || costs.unwrap().0 > new_costs {
                    let mut new_path = path.clone();
                    let priority = new_costs + euclid_dis(&neighor_pos, &goal);

                    new_path.push(neighor_pos);

                    open.push(Node(neighor_pos, priority));
                    costs_so_far.insert(neighor_pos, (new_costs, new_path));
                }
            }
        }
    }

    // there is no path
    None
}

fn costs((_, _, obs): &Tile) -> f32 {
    obs.map(|Obstacle(c)| c).unwrap_or(1.0)
}

fn euclid_dis(a: &MapPos, b: &MapPos) -> f32 {
    let dq = (b.q - a.q) as f32;
    let dr = (b.r - a.r) as f32;

    f32::sqrt(dq * dq + dr * dr + dq * dr)
}

#[test]
fn test_map_syntax() {
    let raw_string = std::fs::read_to_string("assets/data/maps/arena01.map.ron").unwrap();
    let map_spec: Result<MapSpec, _> = ron::from_str(&raw_string);
    assert!(map_spec.is_ok());
}
