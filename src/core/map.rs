use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use crate::core::model::*;

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Floor,
    Void,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile(u32, u32, TileType);

impl Tile {
    pub fn column(self: &Self) -> u32 {
        self.0
    }

    pub fn row(self: &Self) -> u32 {
        self.1
    }

    pub fn tile_type(self: &Self) -> TileType {
        self.2.clone()
    }

    pub fn to_world_pos(self: &Self) -> WorldPos {
        WorldPos::new(self.0 as f32, self.1 as f32, 0.0)
    }

    pub fn to_map_pos(self: &Self) -> MapPos {
        MapPos(self.0 as i32, self.1 as i32)
    }

    pub fn distance(&self, o: &Self) -> f32 {
        distance(self.to_map_pos(), o.to_map_pos())
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.column() == other.column() && self.row() == other.row()
    }
}

impl Hash for Tile {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl Eq for Tile {}

pub type Path = Vec<Tile>;

// impl Path {
//     pub fn len(&self) -> usize {
//         self.0.len()
//     }

//     // pub fn take_step(mut self) -> Option<(Tile, Path)> {
//     //     if self.0.is_empty() {
//     //         return None;
//     //     }

//     //     let t = self.0.remove(0);
//     //     Some((t, self))
//     // }

//     pub fn iter(&self) -> std::slice::Iter<Tile> {
//          self.0.iter()
//     }
// }

// impl Iterator for Path {
//     type Item = Tile;

//     fn next(&mut self) -> Option<Tile> {
//         if self.0.is_empty() {
//             return None;
//         }

//         Some(self.0.remove(0))
//     }
// }

// #[derive(Debug)]
// pub struct PathIter(Vec<Tile>);


// impl Iterator for PathIter {
//     type Item = Tile;

//     fn next(&mut self) -> Option<Tile> {
//         if self.0.is_empty() {
//             return None;
//         }

//         Some(self.0.remove(0))
//     }
// }

#[derive(Default)]
pub struct Map(Vec<Vec<TileType>>);

impl Map {
    pub fn num_columns(self: &Self) -> u32 {
        self.0[0].len() as u32
    }

    pub fn num_rows(self: &Self) -> u32 {
        self.0.len() as u32
    }

    pub fn find_tile(self: &Self, wpos: WorldPos) -> Option<Tile> {
        self.get_tile(MapPos::from_world_pos(wpos))
    }

    pub fn get_tile(self: &Self, MapPos(i, j): MapPos) -> Option<Tile> {
        let rows = &self.0;

        if j >= 0 && j < rows.len() as i32 {
            let cols = &rows[j as usize];

            if i >= 0 && i < cols.len() as i32 {
                let tt = cols[i as usize].clone();
                return Some(Tile(i as u32, j as u32, tt));
            }
        }

        None
    }

    pub fn tiles(&self) -> TileIter {
        TileIter {
            map: self,
            cur_col: 0,
            cur_row: 0,
        }
    }

    pub fn neighbors<'a>(
        &'a self,
        tile: Tile,
        obstacles: &'a ObstacleSet,
    ) -> NeighborTileIter<'a> {
        NeighborTileIter {
            map: self,
            center_col: tile.column() as i32,
            center_row: tile.row() as i32,
            step: 0,
            obstacles: obstacles,
        }
    }

    pub fn find_path(
        &self,
        from: MapPos,
        to: MapPos,
        obstacles: &ObstacleSet,
    ) -> Option<Path> {
        let straight_path = self.find_straight_path(from, to, obstacles);
        if straight_path.is_some() {
            return straight_path;
        }

        if let (Some(s), Some(g)) = (self.get_tile(from), self.get_tile(to)) {
            return find_path_astar(self, s, g, obstacles);
        }

        None
    }

    pub fn find_straight_path(
        &self,
        start: MapPos,
        goal: MapPos,
        obstacles: &ObstacleSet,
    ) -> Option<Path> {
        let mut p = Path::new();
        let d = distance(start, goal).floor() as i32;

        for i in 1..=d {
            let delta = i as f32 / d as f32;
            let next_pos = MapPos::lerp(&start, &goal, delta);

            if let Some(next_tile) = self.get_tile(next_pos) {
                if let Tile(_, _, TileType::Void) = next_tile {
                    return None;
                }

                if obstacles.0.get(&next_tile.to_map_pos()).is_some() {
                    return None;
                } else {
                    p.push(next_tile);
                }
            } else {
                return None;
            }
        }

        Some(p)
    }

    pub fn tiles_along_line(&self, p1: MapPos, p2: MapPos) -> LineIter {
        LineIter::new(self, p1, p2)
    }

    // pub fn find_path_neighborhood(
    //     &self,
    //     from: WorldPos,
    //     to: WorldPos,
    //     min_distance: u8,
    //     max_distance: u8,
    //     obstacles: &HashMap<Tile, Obstacle>,
    // ) -> Option<Path> {
    //     let neighbors = NeighborTileIter::new(self, to, obstacles);
    //     // let neighbors = NeighborTileIter::new(self, to, obstacles, min_distance, max_distance);
    //     let mut result = None;
    //     let mut length = usize::MAX;
        
    //     for n in neighbors {
    //         let n_pos = WorldPos(n.column() as f32, n.row() as f32);
    //         let p = self.find_path(from, n_pos, obstacles);
    //         if let Some(p) = p {
    //             if p.len() < length {
    //                 length = p.len();
    //                 result = Some(p);
    //             }
    //         }
    //     }

    //     result
    // }
}

#[derive(Debug, Clone)]
pub struct Obstacle(pub i8);

// pub enum Obstacle {
//     Inaccessible(),
//     Impediment(f32),
// }

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Hash)]
pub struct MapPos(pub i32, pub i32);

impl MapPos {
    pub fn from_world_pos(pos: WorldPos) -> Self {
        Self(pos.x().round() as i32, pos.y().round() as i32)
    }

    pub fn to_world_pos(self) -> WorldPos {
        WorldPos::new(self.0 as f32, self.1 as f32, 0.0)
    }

    pub fn distance(self, other: MapPos) -> usize {
        let dx = i32::abs(other.0 - self.0) as usize;
        let dy = i32::abs(other.1 - self.1) as usize;
        usize::max(dx, dy)
    }

    pub fn lerp(Self(x1, y1): &Self, Self(x2, y2): &Self, t: f32) -> Self {
        let xn = *x1 as f32 + t * (x2 - x1) as f32;
        let yn = *y1 as f32 + t * (y2 - y1) as f32;
        Self(xn.round() as i32, yn.round() as i32)
    }
}
                          

pub struct ObstacleSet(pub HashMap<MapPos, Obstacle>);

impl ObstacleSet {
    pub fn ignore(self, pos: MapPos) -> Self {
        let mut hm = self.0;
        hm.remove(&pos);
        Self(hm)
    }
}

pub struct LineIter<'a> {
    map: &'a Map,
    p1: MapPos,
    p2: MapPos,
    distance: usize,
    step: usize,
}

impl<'a> LineIter<'a> {
    pub fn new(map: &'a Map, p1: MapPos, p2: MapPos) -> Self {
        Self {
            map,
            p1,
            p2,
            distance: p1.distance(p2),
            step: 0,
        }
    }
}

impl<'a> Iterator for LineIter<'a> {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        let delta = self.step as f32 / self.distance as f32;
        let next_pos = MapPos::lerp(&self.p1, &self.p2, delta);

        self.step += 1;
        self.map.get_tile(next_pos)
    }
}

pub struct TileIter<'a> {
    map: &'a Map,
    cur_col: usize,
    cur_row: usize,
}

impl<'a> Iterator for TileIter<'a> {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        let rows = &self.map.0;

        if self.cur_row < rows.len() {
            let cols = &rows[self.cur_row];

            if self.cur_col < cols.len() {
                let tt = cols[self.cur_col].clone();
                self.cur_col = self.cur_col + 1;
                return Some(Tile(self.cur_col as u32, self.cur_row as u32, tt));
            } else {
                self.cur_row = self.cur_row + 1;
                self.cur_col = 0;
                return self.next();
            }
        }

        None
    }
}

const NEIGHBOR_CANDIDATES: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub struct NeighborTileIter<'a> {
    map: &'a Map,
    center_col: i32,
    center_row: i32,
    step: usize,
    obstacles: &'a ObstacleSet,
}

impl<'a> Iterator for NeighborTileIter<'a> {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        while self.step < NEIGHBOR_CANDIDATES.len() {
            self.step += 1;

            let (dx, dy) = NEIGHBOR_CANDIDATES[self.step - 1];
            let p = MapPos(self.center_col + dx, self.center_row + dy);

            if let Some(t) = self.map.get_tile(p) {
                if let TileType::Void = t.tile_type() {
                    // you cannot pass the void
                    continue;
                }

                if self.obstacles.0.contains_key(&p) {
                // if !self.obstacles.0.get(&p).map(|o| o.allow_movement).unwrap_or(true) {
                // if let Some(Obstacle::Inaccessible()) = self.obstacles.0.get(&p) {
                    // the way is blocked by an inpenetrable obstacle
                    continue;
                }

                return Some(t);
            }
        }

        None
    }
}

#[test]
fn it_can_find_a_path() {
    let m = Map(vec![
        row(vec![1, 0, 1, 1, 1]),
        row(vec![1, 0, 1, 1, 1]),
        row(vec![1, 1, 1, 1, 1]),
    ]);
     
    let obstacles = ObstacleSet(HashMap::new());
    let from = MapPos(0, 0);
    let to = MapPos(3, 1);
    let p = m.find_path(from, to, &obstacles).unwrap();
    let p2 = m.find_straight_path(from, to, &obstacles);

    // assert!(p.is_some());
    assert_eq!(p.len(), 4);
    assert_eq!(p2, None);
    let mut p = p.iter();
    assert_eq!(p.next(), Some(&Tile(0, 1, TileType::Floor)));
    assert_eq!(p.next(), Some(&Tile(1, 2, TileType::Floor)));
    assert_eq!(p.next(), Some(&Tile(2, 2, TileType::Floor)));
    assert_eq!(p.next(), Some(&Tile(3, 1, TileType::Floor)));
}

#[test]
fn it_can_find_a_staight_path() {
    let m = Map(vec![
        row(vec![1, 1, 1, 1, 1, 1]),
        row(vec![1, 1, 1, 1, 1, 1]),
        row(vec![1, 1, 1, 1, 1, 1]),
    ]);
     
    let from = MapPos(0, 0);
    let to = MapPos(5, 2);
    let p = m.find_path(from, to, &ObstacleSet(HashMap::new())).unwrap();
    println!("{:?}", p);

    // assert!(p.is_some());
    assert_eq!(p.len(), 5);
    let mut p = p.iter();
    assert_eq!(p.next(), Some(&Tile(1, 0, TileType::Floor)));
    assert_eq!(p.next(), Some(&Tile(2, 1, TileType::Floor)));
    assert_eq!(p.next(), Some(&Tile(3, 1, TileType::Floor)));
    assert_eq!(p.next(), Some(&Tile(4, 2, TileType::Floor)));
    assert_eq!(p.next(), Some(&Tile(5, 2, TileType::Floor)));
}

pub fn dummy() -> Map {
    Map(vec![
        row(vec![0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0]),
        row(vec![0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0]),
        row(vec![0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0]),
        row(vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        row(vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        row(vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        row(vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        row(vec![0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0]),
        row(vec![0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0]),
        row(vec![0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0]),
    ])
}

fn row(row_tiles: Vec<u8>) -> Vec<TileType> {
    Vec::from_iter(row_tiles.iter().map(|&i| {
        if i > 0 {
            TileType::Floor
        } else {
            TileType::Void
        }
    }))
}

struct Node(Tile, f32);

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 == other.0 {
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

fn find_path_astar(
    m: &Map,
    start: Tile,
    goal: Tile,
    obstacles: &ObstacleSet,
) -> Option<Path> {
    let start_node = Node(start, 0.0);
    let mut costs_so_far: HashMap<Tile, (f32, Path)> = HashMap::new();
    let mut open: BinaryHeap<Node> = BinaryHeap::new();

    open.push(start_node);
    costs_so_far.insert(start, (0.0, Vec::new()));

    while let Some(Node(tile, _)) = open.pop() {
        let (current_costs, path) = costs_so_far.get(&tile).unwrap().clone();

        // println!("  > explore {:?} (costs={})", tile, current_costs);
        if tile == goal {
            // the current candidat is the goal tile
            // -> return result
            return Some(path.clone());
        } else {
            // the current candidate is not the goal
            // -> ... and look its at its neighbors
            for n in m.neighbors(tile, obstacles) {
                let new_costs = current_costs + costs(&n, obstacles);
                let costs = costs_so_far.get(&n);

                if costs.is_none() || costs.unwrap().0 > new_costs {
                    let mut new_path = path.clone();
                    let priority = new_costs + n.distance(&goal);

                    new_path.push(n);

                    open.push(Node(n, priority));
                    costs_so_far.insert(n, (new_costs, new_path));
                }
            }
        }
    }

    // there is no path
    None
}

const DP: f32 = 1.0; // distance for perpendicular (non-diaginal) steps
const DD: f32 = 1.0; // distance for diaginal steps; sqrt(2)

/// Estimates the distance between two map points (tiles) A and B ignoring
/// possible obstacles
/// (based on http://theory.stanford.edu/~amitp/GameProgramming/Heuristics.html#heuristics-for-grid-maps)
fn distance(MapPos(x1, y1): MapPos, MapPos(x2, y2): MapPos) -> f32 {
    let dx = (x1 - x2).abs() as f32;
    let dy = (y1 - y2).abs() as f32;

    DP * (dx + dy) + (DD - 2.0 * DP) * f32::min(dx, dy)
}

fn costs(t: &Tile, obstacles: &ObstacleSet) -> f32 {
    obstacles.0.get(&t.to_map_pos()).map(|Obstacle(costs)| *costs as f32).unwrap_or(1.0)
}
