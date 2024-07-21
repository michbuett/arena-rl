// use std::cmp::Ordering;
use std::collections::HashMap;
// use std::collections::BinaryHeap;
use std::hash::Hash;
// use std::hash::Hasher;
// use std::iter::FromIterator;
// use std::num::NonZeroU8;

use bevy::ecs::component::Component;
use bevy::ecs::system::Resource;
use bevy::math::{Vec2, Vec3};

const SQRT_3: f32 = 1.7320508076;
const HEX_SIZE: f32 = 64.0;
// const HEX_WIDTH: f32 = SQRT_3 * HEX_SIZE; // size * sqrt(3)
// const HEX_HEIGHT: f32 = 2.0 * HEX_SIZE;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Component)]
pub struct MapPosHex {
    q: i32,
    r: i32,
    s: i32,
}

impl MapPosHex {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r, s: -q - r }
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
        let mut s = frac_s.round();

        let q_diff = f32::abs(q - frac_q);
        let r_diff = f32::abs(r - frac_r);
        let s_diff = f32::abs(s - frac_s);

        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        } else {
            s = -q - r;
        }

        Self {
            q: q as i32,
            r: r as i32,
            s: s as i32,
        }
    }
}

impl From<Vec3> for MapPosHex {
    fn from(point: Vec3) -> Self {
        MapPosHex::from_xy(point.x, point.y)
    }
}

impl From<Vec2> for MapPosHex {
    fn from(point: Vec2) -> Self {
        MapPosHex::from_xy(point.x, point.y)
    }
}

#[test]
fn test_can_convert_screen_coord_to_hex_and_back() {
    let hex1 = MapPosHex::new(-1, -2);
    let hex2 = MapPosHex::new(1, -1);
    let hex3 = MapPosHex::new(-2, 3);

    assert_eq!(hex1, MapPosHex::from(hex1.into_vec3()));
    assert_eq!(hex2, MapPosHex::from(hex2.into_vec3()));
    assert_eq!(hex3, MapPosHex::from(hex3.into_vec3()));
}

#[derive(Resource)]
pub struct HexMap {
    tiles: HashMap<MapPosHex, TileType>,
    pub camera_focus: MapPosHex,
    pub scroll_limit_x: (f32, f32),
    pub scroll_limit_y: (f32, f32),
}

impl HexMap {
    pub fn tiles(&self) -> impl Iterator<Item = (&MapPosHex, &TileType)> {
        self.tiles.iter()
    }

    pub fn find_tile(&self, hex: &MapPosHex) -> Option<(MapPosHex, TileType)> {
        self.tiles.get(hex).map(|tt| (*hex, *tt))
    }
}

pub fn dummy_hex() -> HexMap {
    let mut tiles = HashMap::new();
    let raw = vec![
        /*        0 1 2 3 4 5 6 7 8 9 */
        /*  0 */ "    . . . . . . . .    ",
        /*  1 */ "   . . . . . . . . .   ",
        /*  2 */ "  . . . . . . . . . .  ",
        /*  3 */ " . . . . . . . . . . . ",
        /*  4 */ ". . . . . . . . . . . .",
        /*  5 */ " . . . . . . . . . . . ",
        /*  6 */ ". . . . . . . . . . . .",
        /*  7 */ " . . . . . . . . . . . ",
        /*  8 */ "  . . . . . . . . . .  ",
        /*  9 */ "   . . . . . . . . .   ",
        /* 10 */ "    . . . . . . . .    ",
    ];

    // let mut tiles = vec![];
    for (row, row_str) in raw.iter().enumerate() {
        let start_idx = row_str.find(|c| !char::is_whitespace(c)).unwrap() / 2;
        let tiles_in_row = row_str
            .split_whitespace()
            .filter_map(|tile_str| match tile_str {
                "." => Some(TileType::Floor),
                _ => None,
            })
            .enumerate();

        for (col, ttype) in tiles_in_row {
            tiles.insert(
                MapPosHex::from_oddr((start_idx + col) as i32, row as i32),
                ttype,
            );
        }
    }

    let camera_focus = MapPosHex::from_oddr(5, 5);
    let focus_vec = camera_focus.into_vec3();

    HexMap {
        tiles,
        camera_focus,
        scroll_limit_x: (focus_vec.x - 500.0, focus_vec.x + 500.0),
        scroll_limit_y: (focus_vec.y - 500.0, focus_vec.y + 500.0),
    }
}

#[test]
fn test_can_build_hex_map() {
    let map = dummy_hex();
    assert!(map.tiles().count() > 0);
}

// const TILE_SIZE: f32 = 96.0;

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Floor,
    // Void,
}

// #[derive(Debug, Clone, Copy)]
// pub struct Tile(MapPos, TileType);

// impl Tile {
//     pub fn column(self: &Self) -> u32 {
//         self.0.col()
//     }

//     pub fn row(self: &Self) -> u32 {
//         self.0.row()
//     }

//     pub fn tile_type(self: &Self) -> TileType {
//         self.1.clone()
//     }

//     pub fn is_void(&self) -> bool {
//         match self.1 {
//             TileType::Void => true,
//             _ => false,
//         }
//     }

//     pub fn world_pos(self: &Self) -> WorldPos {
//         self.map_pos().into()
//     }

//     pub fn map_pos(self: &Self) -> MapPos {
//         self.0
//     }

//     pub fn distance(&self, o: &Self) -> f32 {
//         distance(self.map_pos(), o.map_pos())
//     }
// }

// impl PartialEq for Tile {
//     fn eq(&self, other: &Self) -> bool {
//         self.column() == other.column() && self.row() == other.row()
//     }
// }

// impl Hash for Tile {
//     fn hash<H>(&self, state: &mut H)
//     where
//         H: Hasher,
//     {
//         self.0.hash(state);
//     }
// }

// impl Eq for Tile {}

// pub type Path = Vec<Tile>;

// #[derive(Resource)]
// pub struct Map(Vec<Vec<TileType>>);

// impl Map {
//     // pub fn new(Vec<Vec<TileType>>) -> Self {

//     // }

//     pub fn num_columns(self: &Self) -> u32 {
//         self.0[0].len() as u32
//     }

//     pub fn num_rows(self: &Self) -> u32 {
//         self.0.len() as u32
//     }

//     // pub fn find_tile(self: &Self, wpos: WorldPos) -> Option<Tile> {
//     //     self.get_tile(MapPos::from_world_pos(wpos))
//     // }

//     pub fn get_tile(self: &Self, mpos: MapPos) -> Option<Tile> {
//         let rows = &self.0;
//         let (i, j) = (mpos.col() as usize, mpos.row() as usize);

//         if j >= 0 && j < rows.len() {
//             let cols = &rows[j];

//             if i >= 0 && i < cols.len() {
//                 let tt = cols[i].clone();
//                 return Some(Tile(mpos, tt));
//             }
//         }

//         None
//     }

//     pub fn tiles(&self) -> TileIter {
//         TileIter {
//             map: self,
//             cur_col: 0,
//             cur_row: 0,
//         }
//     }

//     pub fn neighbors<'a>(
//         &'a self,
//         tile: Tile,
//         distance: NonZeroU8,
//         obstacles: &'a ObstacleSet,
//     ) -> NeighborTileIter<'a> {
//         NeighborTileIter {
//             map: self,
//             center_col: tile.column() as i32,
//             center_row: tile.row() as i32,
//             step: 0,
//             obstacles,
//             distance,
//         }
//     }

//     pub fn find_path(&self, from: MapPos, to: MapPos, obstacles: &ObstacleSet) -> Option<Path> {
//         let straight_path = self.find_straight_path(from, to, obstacles);
//         if straight_path.is_some() {
//             return straight_path;
//         }

//         if let (Some(s), Some(g)) = (self.get_tile(from), self.get_tile(to)) {
//             return find_path_astar(self, s, g, obstacles);
//         }

//         None
//     }

//     pub fn find_straight_path(
//         &self,
//         start: MapPos,
//         goal: MapPos,
//         obstacles: &ObstacleSet,
//     ) -> Option<Path> {
//         let mut p = Path::new();
//         let d = distance(start, goal).floor() as i32;

//         for i in 1..=d {
//             let delta = i as f32 / d as f32;
//             let next_pos = MapPos::lerp(&start, &goal, delta);

//             if let Some(next_tile) = self.get_tile(next_pos) {
//                 if let Tile(_, TileType::Void) = next_tile {
//                     return None;
//                 }

//                 if obstacles.0.get(&next_tile.into()).is_some() {
//                     return None;
//                 } else {
//                     p.push(next_tile);
//                 }
//             } else {
//                 return None;
//             }
//         }

//         Some(p)
//     }

//     // pub fn tiles_along_line(&self, p1: MapPos, p2: MapPos) -> LineIter {
//     //     LineIter::new(self, p1, p2)
//     // }

//     // pub fn find_path_neighborhood(
//     //     &self,
//     //     from: WorldPos,
//     //     to: WorldPos,
//     //     min_distance: u8,
//     //     max_distance: u8,
//     //     obstacles: &HashMap<Tile, Obstacle>,
//     // ) -> Option<Path> {
//     //     let neighbors = NeighborTileIter::new(self, to, obstacles);
//     //     // let neighbors = NeighborTileIter::new(self, to, obstacles, min_distance, max_distance);
//     //     let mut result = None;
//     //     let mut length = usize::MAX;

//     //     for n in neighbors {
//     //         let n_pos = WorldPos(n.column() as f32, n.row() as f32);
//     //         let p = self.find_path(from, n_pos, obstacles);
//     //         if let Some(p) = p {
//     //             if p.len() < length {
//     //                 length = p.len();
//     //                 result = Some(p);
//     //             }
//     //         }
//     //     }

//     //     result
//     // }
// }

// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
// pub enum Obstacle {
//     /// An obstacle that is impossible to overcome
//     Blocker,

//     /// An obstacle which one may overcome with skill and luck
//     /// obscurity (a modifiyer to the success chance of a roll) and blocking (a
//     /// modifiyer of roll quality)
//     Impediment(NonZeroU8, i8),
// }

// pub struct WorldPos(Vec3);

// impl Into<Vec3> for WorldPos {
//     fn into(self) -> Vec3 {
//         return self.0;
//     }
// }

// impl From<MapPos> for WorldPos {
//     fn from(value: MapPos) -> Self {
//         Self(Vec3 {
//             x: value.col() as f32 * TILE_SIZE,
//             y: value.row() as f32 * TILE_SIZE,
//             z: 0.0,
//         })
//     }
// }

// #[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Component)]
// pub struct MapPos(pub u32, pub u32);

// impl MapPos {
//     pub fn col(&self) -> u32 {
//         self.0
//     }

//     pub fn row(&self) -> u32 {
//         self.1
//     }

//     pub fn distance(self, other: MapPos) -> usize {
//         let dx = i32::abs(other.0 as i32 - self.0 as i32) as usize;
//         let dy = i32::abs(other.1 as i32 - self.1 as i32) as usize;
//         usize::max(dx, dy)
//     }

//     pub fn lerp(Self(x1, y1): &Self, Self(x2, y2): &Self, t: f32) -> Self {
//         let xn = *x1 as f32 + t * (x2 - x1) as f32;
//         let yn = *y1 as f32 + t * (y2 - y1) as f32;
//         let col = i32::max(xn.round() as i32, 0) as u32;
//         let row = i32::max(yn.round() as i32, 0) as u32;

//         Self(col, row)
//     }
// }

// impl Into<MapPos> for Tile {
//     fn into(self) -> MapPos {
//         self.0
//     }
// }

// #[derive(Clone)]
// pub struct ObstacleSet(pub HashMap<MapPos, Obstacle>);

// impl ObstacleSet {
//     pub fn ignore(self, pos: MapPos) -> Self {
//         let mut hm = self.0;
//         hm.remove(&pos);
//         Self(hm)
//     }
// }

// // pub struct LineIter<'a> {
// //     map: &'a Map,
// //     p1: MapPos,
// //     p2: MapPos,
// //     distance: usize,
// //     step: usize,
// // }

// // impl<'a> LineIter<'a> {
// //     pub fn new(map: &'a Map, p1: MapPos, p2: MapPos) -> Self {
// //         Self {
// //             map,
// //             p1,
// //             p2,
// //             distance: p1.distance(p2),
// //             step: 0,
// //         }
// //     }
// // }

// // impl<'a> Iterator for LineIter<'a> {
// //     type Item = Tile;

// //     fn next(&mut self) -> Option<Tile> {
// //         let delta = self.step as f32 / self.distance as f32;
// //         let next_pos = MapPos::lerp(&self.p1, &self.p2, delta);

// //         self.step += 1;
// //         self.map.get_tile(next_pos)
// //     }
// // }

// /// An iterator that iterates over all map positions which are crossed by a line between two points p0 and p1
// /// The algorithm is an adaptation of https://www.redblobgames.com/grids/line-drawing.html#supercover
// /// The line itself is endless so it should be used in combination with e.g. take() or take_while()
// /// panics if p0 == p1 in non-optimized builds (debug mode)
// pub struct SuperLineIter {
//     distance: (i32, i32),
//     step: (i32, i32),
//     next_pos: MapPos,
//     sign: (i32, i32),
// }

// impl SuperLineIter {
//     pub fn new(p0: MapPos, p1: MapPos) -> Self {
//         debug_assert!(p0 != p1);

//         let (x0, y0) = (p0.col() as i32, p0.row() as i32);
//         let (x1, y1) = (p1.col() as i32, p1.row() as i32);
//         let (dx, dy) = (x1 - x0, y1 - y0);

//         Self {
//             next_pos: p0,
//             distance: (i32::abs(dx), i32::abs(dy)),
//             step: (0, 0),
//             sign: (i32::signum(dx), i32::signum(dy)),
//         }
//     }
// }

// impl Iterator for SuperLineIter {
//     type Item = MapPos;

//     fn next(&mut self) -> Option<Self::Item> {
//         let next_pos = self.next_pos;
//         let (x, y) = (self.next_pos.col() as i32, self.next_pos.row() as i32);
//         let (sign_x, sign_y) = self.sign;
//         let (ix, iy) = self.step;
//         let (nx, ny) = self.distance;
//         let decision = (1 + 2 * ix) * ny - (1 + 2 * iy) * nx;
//         let (total_x, total_y, next_x, next_y) = if decision == 0 {
//             // next step is diagonal
//             (ix + 1, iy + 1, x + sign_x, y + sign_y)
//         } else if decision < 0 {
//             // next step is horizontal
//             (ix + 1, iy, x + sign_x, y)
//         } else {
//             // next step is vertical
//             (ix, iy + 1, x, y + sign_y)
//         };

//         self.next_pos = MapPos(next_x as u32, next_y as u32);
//         self.step = (total_x, total_y);

//         Some(next_pos)
//     }
// }

// pub struct TileIter<'a> {
//     map: &'a Map,
//     cur_col: usize,
//     cur_row: usize,
// }

// impl<'a> Iterator for TileIter<'a> {
//     type Item = Tile;

//     fn next(&mut self) -> Option<Tile> {
//         let rows = &self.map.0;

//         if self.cur_row < rows.len() {
//             let cols = &rows[self.cur_row];

//             if self.cur_col < cols.len() {
//                 let tt = cols[self.cur_col].clone();
//                 let tile = Tile(MapPos(self.cur_col as u32, self.cur_row as u32), tt);

//                 self.cur_col = self.cur_col + 1;

//                 return Some(tile);
//             } else {
//                 self.cur_row = self.cur_row + 1;
//                 self.cur_col = 0;
//                 return self.next();
//             }
//         }

//         None
//     }
// }

// pub struct NeighborTileIter<'a> {
//     map: &'a Map,
//     distance: NonZeroU8,
//     center_col: i32,
//     center_row: i32,
//     step: usize,
//     obstacles: &'a ObstacleSet,
// }

// impl<'a> Iterator for NeighborTileIter<'a> {
//     type Item = Tile;

//     fn next(&mut self) -> Option<Tile> {
//         let dim = 2 * self.distance.get() as usize + 1;
//         let last_step_num = dim * dim;

//         while self.step < last_step_num {
//             self.step += 1;

//             let dx = ((self.step - 1) % dim) as i32 - self.distance.get() as i32;
//             let dy = ((self.step - 1) / dim) as i32 - self.distance.get() as i32;

//             if dx == 0 && dy == 0 {
//                 // the center is not a neighbor
//                 continue;
//             }

//             let p = MapPos((self.center_col + dx) as u32, (self.center_row + dy) as u32);

//             if let Some(t) = self.map.get_tile(p) {
//                 if let TileType::Void = t.tile_type() {
//                     // you cannot pass the void
//                     continue;
//                 }

//                 if self.obstacles.0.contains_key(&p) {
//                     // if !self.obstacles.0.get(&p).map(|o| o.allow_movement).unwrap_or(true) {
//                     // if let Some(Obstacle::Inaccessible()) = self.obstacles.0.get(&p) {
//                     // the way is blocked by an inpenetrable obstacle
//                     continue;
//                 }

//                 return Some(t);
//             }
//         }

//         None
//     }
// }

// #[test]
// fn it_can_find_a_path() {
//     let m = build_map(vec![
//         vec![1, 0, 1, 1, 1],
//         vec![1, 0, 1, 1, 1],
//         vec![1, 1, 1, 1, 1],
//     ]);

//     let obstacles = ObstacleSet(HashMap::new());
//     let from = MapPos(0, 0);
//     let to = MapPos(3, 1);
//     let p = m.find_path(from, to, &obstacles).unwrap();
//     let p2 = m.find_straight_path(from, to, &obstacles);

//     // assert!(p.is_some());
//     assert_eq!(p.len(), 4);
//     assert_eq!(p2, None);
//     let mut p = p.iter();
//     assert_eq!(p.next(), Some(&Tile(MapPos(0, 1), TileType::Floor)));
//     assert_eq!(p.next(), Some(&Tile(MapPos(1, 2), TileType::Floor)));
//     assert_eq!(p.next(), Some(&Tile(MapPos(2, 2), TileType::Floor)));
//     assert_eq!(p.next(), Some(&Tile(MapPos(3, 1), TileType::Floor)));
// }

// #[test]
// fn it_can_find_a_staight_path() {
//     let m = build_map(vec![
//         vec![1, 1, 1, 1, 1, 1],
//         vec![1, 1, 1, 1, 1, 1],
//         vec![1, 1, 1, 1, 1, 1],
//     ]);

//     let from = MapPos(0, 0);
//     let to = MapPos(5, 2);
//     let p = m.find_path(from, to, &ObstacleSet(HashMap::new())).unwrap();

//     // assert!(p.is_some());
//     assert_eq!(p.len(), 5);
//     let mut p = p.iter();
//     assert_eq!(p.next(), Some(&Tile(MapPos(1, 0), TileType::Floor)));
//     assert_eq!(p.next(), Some(&Tile(MapPos(2, 1), TileType::Floor)));
//     assert_eq!(p.next(), Some(&Tile(MapPos(3, 1), TileType::Floor)));
//     assert_eq!(p.next(), Some(&Tile(MapPos(4, 2), TileType::Floor)));
//     assert_eq!(p.next(), Some(&Tile(MapPos(5, 2), TileType::Floor)));
// }

// pub fn dummy() -> Map {
//     // build_map(vec![
//     //     vec![0, 1, 0, 0],
//     //     vec![1, 1, 1, 1],
//     //     vec![1, 1, 1, 1],
//     //     vec![1, 1, 0, 0],
//     // ])
//     build_map(vec![
//         vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0],
//         vec![0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
//         vec![0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
//         vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
//         vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
//         vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
//         vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
//         vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
//         vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
//         vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
//         vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
//         vec![0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
//         vec![0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
//         vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0],
//     ])
// }

// fn build_map(tiles: Vec<Vec<u8>>) -> Map {
//     let grid = tiles.iter().rev().map(|r| row(r)).collect();
//     Map(grid)
// }

// fn row(row_tiles: &Vec<u8>) -> Vec<TileType> {
//     Vec::from_iter(row_tiles.iter().map(|&i| {
//         if i > 0 {
//             TileType::Floor
//         } else {
//             TileType::Void
//         }
//     }))
// }

// struct Node(Tile, f32);

// impl PartialEq for Node {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0
//     }
// }

// impl Eq for Node {}

// impl Ord for Node {
//     fn cmp(&self, other: &Self) -> Ordering {
//         if self.0 == other.0 {
//             return Ordering::Equal;
//         }

//         match self.1 > other.1 {
//             true => Ordering::Less,
//             false => Ordering::Greater,
//         }
//     }
// }

// impl PartialOrd for Node {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// fn find_path_astar(m: &Map, start: Tile, goal: Tile, obstacles: &ObstacleSet) -> Option<Path> {
//     let start_node = Node(start, 0.0);
//     let mut costs_so_far: HashMap<Tile, (f32, Path)> = HashMap::new();
//     let mut open: BinaryHeap<Node> = BinaryHeap::new();

//     open.push(start_node);
//     costs_so_far.insert(start, (0.0, Vec::new()));

//     while let Some(Node(tile, _)) = open.pop() {
//         let (current_costs, path) = costs_so_far.get(&tile).unwrap().clone();

//         // println!("  > explore {:?} (costs={})", tile, current_costs);
//         if tile == goal {
//             // the current candidat is the goal tile
//             // -> return result
//             return Some(path.clone());
//         } else {
//             // the current candidate is not the goal
//             // -> ... and look its at its neighbors
//             for n in m.neighbors(tile, NonZeroU8::new(1).unwrap(), obstacles) {
//                 let new_costs = current_costs + costs(&n, obstacles);
//                 let costs = costs_so_far.get(&n);

//                 if costs.is_none() || costs.unwrap().0 > new_costs {
//                     let mut new_path = path.clone();
//                     let priority = new_costs + n.distance(&goal);

//                     new_path.push(n);

//                     open.push(Node(n, priority));
//                     costs_so_far.insert(n, (new_costs, new_path));
//                 }
//             }
//         }
//     }

//     // there is no path
//     None
// }

// const DP: f32 = 1.0; // distance for perpendicular (non-diaginal) steps
// const DD: f32 = 1.0; // distance for diaginal steps; sqrt(2)

// /// Estimates the distance between two map points (tiles) A and B ignoring
// /// possible obstacles
// /// (based on http://theory.stanford.edu/~amitp/GameProgramming/Heuristics.html#heuristics-for-grid-maps)
// fn distance(MapPos(x1, y1): MapPos, MapPos(x2, y2): MapPos) -> f32 {
//     let dx = x1.checked_sub(x2).unwrap_or(0) as f32;
//     let dy = y1.checked_sub(y2).unwrap_or(0) as f32;

//     DP * (dx + dy) + (DD - 2.0 * DP) * f32::min(dx, dy)
// }

// fn costs(t: &Tile, obstacles: &ObstacleSet) -> f32 {
//     obstacles
//         .0
//         .get(&t.map_pos())
//         .map(|obs| match obs {
//             Obstacle::Impediment(c, _) => c.get() as f32,
//             Obstacle::Blocker => f32::MAX,
//         })
//         .unwrap_or(1.0)
// }
