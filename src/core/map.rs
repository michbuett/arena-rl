use std::cmp::Ordering;
use std::iter::FromIterator;
use std::hash::{Hash, Hasher};
use std::collections::{HashMap, BinaryHeap};

use crate::core::model::*;

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Floor,
    Void
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
        WorldPos(self.0 as f32, self.1 as f32)
    }

    pub fn distance(&self, o: &Self) -> f32 {
        distance(self.0 as f32, self.1 as f32, o.0 as f32, o.1 as f32)
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.column() == other.column() && self.row() == other.row()
    }
}

impl Hash for Tile {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl Eq for Tile { }

#[derive(Debug)]
pub struct Path(Vec<Tile>);

impl Path {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn take_step(mut self) -> Option<(Tile, Path)> {
        if self.0.is_empty() {
            return None
        }

        let t = self.0.remove(0);
        Some((t, self))
    }

    // pub fn iter(&self) -> std::slice::Iter<Tile> {
    //     self.0.iter()
    // }
}

#[derive(Default)]
pub struct Map(Vec<Vec<TileType>>);

impl Map {
    pub fn num_columns(self: &Self) -> u32 {
        self.0[0].len() as u32
    }

    pub fn num_rows(self: &Self) -> u32 {
        self.0.len() as u32
    }

    pub fn find_tile(self: &Self, WorldPos(x, y): WorldPos) -> Option<Tile> {
        let i = x.floor() as i32;
        let j = y.floor() as i32;
        let rows = &self.0;

        if j >= 0 && j < rows.len() as i32 {
            let cols = &rows[j as usize];

            if i >= 0 && i < cols.len() as i32 {
                let tt = cols[i as usize].clone();
                return Some(Tile(i as u32, j as u32, tt))
            }
        }

        None
    }

    pub fn tiles(&self) -> TileIter {
        TileIter { map: self, cur_col: 0, cur_row: 0 }
    }


    pub fn neighbors<'a>(
        &'a self,
        tile: Tile,
        obstacles: &'a HashMap<Tile, Obstacle>
) -> NeighborTileIter<'a> {
        NeighborTileIter { map: self, center: tile, step: 0, obstacles: obstacles }
    }

    pub fn find_path(
        &self,
        from: WorldPos,
        to: WorldPos,
        obstacles: &HashMap<Tile, Obstacle>,
    ) -> Option<Path> {
        if let (Some(s), Some(g)) = (self.find_tile(from), self.find_tile(to)) {
            let straight_path = find_path_straight(self, s, g, obstacles);
            // println!("  - use straight line: {:?}", straight_path);

            if straight_path.is_some() {
                return straight_path;
            }

            let astar_path = find_path_astar(self, s, g, obstacles);
            // println!("  - use A*: {:?}", astar_path);
            return astar_path;
        }

        None
    }
}

pub enum Obstacle {
    Inaccessible(),
    Impediment(f32),
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
                return Some(Tile(self.cur_col as u32, self.cur_row as u32, tt))
            } else {
                self.cur_row = self.cur_row + 1;
                self.cur_col = 0;
                return self.next()
            }
        }

        None
    }
}

const NEIGHBOR_CANDIDATES: [(i8, i8); 8] = [
    (-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)
];

pub struct NeighborTileIter<'a> {
    map: &'a Map,
    center: Tile,
    step: u8,
    obstacles: &'a HashMap::<Tile, Obstacle>,
}

impl<'a> Iterator for NeighborTileIter<'a> {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        while self.step < 8 {
            self.step += 1;

            let (dc, dr) = NEIGHBOR_CANDIDATES[self.step as usize - 1];
            let c = self.center;
            let p = WorldPos (
                c.column() as f32 + dc as f32,
                c.row() as f32 + dr as f32,
            );


            if let Some(t) = self.map.find_tile(p) {
                if let TileType::Void = t.tile_type() {
                    // you cannot pass the void
                    continue;
                }

                if let Some(Obstacle::Inaccessible()) = self.obstacles.get(&t) {
                    // the way is blocked by an inpenetrable obstacle
                    continue;
                }

                return Some(t)
            }
        }

        None
    }
}


pub fn dummy() -> Map {
    Map(vec!(
        row([0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0]),
        row([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0]),
        row([0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0]),
        row([0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        row([0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        row([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        row([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        row([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        row([0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        row([0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        row([0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0]),
        row([0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0]),
        row([0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0]),
    ))
}

fn row(row_tiles: [u8; 13]) -> Vec<TileType> {
    Vec::from_iter(row_tiles.iter().map(|&i| {
        if i > 0 { TileType::Floor } else { TileType::Void }
    }))
}

struct Node(Tile, f32);

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Node { }

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 == other.0 {
            return Ordering::Equal
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
    obstacles: &HashMap::<Tile, Obstacle>
) -> Option<Path> {
    let start_node = Node(start, 0.0);
    let mut costs_so_far: HashMap<Tile, (f32, Vec<Tile>)> = HashMap::new();
    let mut open: BinaryHeap<Node> = BinaryHeap::new();

    open.push(start_node);
    costs_so_far.insert(start, (0.0, Vec::new()));

    while let Some(Node(tile, _)) = open.pop() {
        let (current_costs, path) = costs_so_far.get(&tile).unwrap().clone();

        // println!("  > explore {:?} (costs={})", tile, current_costs);
        if tile == goal {
            // the current candidat is the goal tile
            // -> return result
            return Some(Path(path.clone()))
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


fn find_path_straight(
    m: &Map,
    start: Tile,
    goal: Tile,
    obstacles: &HashMap::<Tile, Obstacle>
) -> Option<Path> {
    let mut p = Vec::new();
    let d = start.distance(&goal).floor() as i32;

    // let WorldPos(xs, ys) = start.to_world_pos();
    // let WorldPos(xg, yg) = goal.to_world_pos();
    // let start_pos = WorldPos(xs + 0.5, ys + 0.5);
    // let goal_pos = WorldPos(xg + 0.5, yg + 0.5);
    let start_pos = start.to_world_pos();
    let goal_pos = goal.to_world_pos();

    for i in 1..=d {
        let delta = i as f32 / d as f32;
        // let next_pos = WorldPos::lerp(&start_pos, &goal_pos, delta);
        let WorldPos(xn, yn) = WorldPos::lerp(&start_pos, &goal_pos, delta);
        let next_pos = WorldPos(xn.round(), yn.round());

        if let Some(next_tile) = m.find_tile(next_pos) {
            if let Tile(_, _, TileType::Void) = next_tile {
                return None
            }

            if obstacles.get(&next_tile).is_some() {
                return None

            } else {
                p.push(next_tile);
            }
        } else {
            return None
        }
    }

    Some(Path(p))
}

const DP: f32 = 1.0; // distance for perpendicular (non-diaginal) steps
const DD: f32 = 1.0; // distance for diaginal steps; sqrt(2)

/// Estimates the distance between two map points (tiles) A and B ignoring
/// possible obstacles
/// (based on http://theory.stanford.edu/~amitp/GameProgramming/Heuristics.html#heuristics-for-grid-maps)
fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = (x1.floor() - x2.floor()).abs();
    let dy = (y1.floor() - y2.floor()).abs();
    (DP * (dx + dy) + (DD - 2.0 * DP) * f32::min(dx, dy))
}

fn costs(t: &Tile, obstacles: &HashMap<Tile, Obstacle>) -> f32 {
    if let Some(Obstacle::Impediment(i)) = obstacles.get(t) {
        return *i
    }
    1.0
}
