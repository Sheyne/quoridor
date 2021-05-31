pub mod bitpacked;
pub mod v1;

use parse_display::{Display, FromStr};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    pub fn other(self) -> Orientation {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

pub trait Board {
    fn empty() -> Self;

    fn available_walls(&self, player: Player) -> u8;

    fn add_wall(
        &mut self,
        player: Player,
        location: (u8, u8),
        orientation: Orientation,
    ) -> Result<(), ()>;
    fn move_token(&mut self, player: Player, new_location: (u8, u8)) -> Result<(), ()>;
    fn is_probably_legal(&self, player: Player, candidate_move: &Move) -> bool {
        self.is_legal(player, candidate_move)
    }
    fn is_legal(&self, player: Player, candidate_move: &Move) -> bool;
    fn get_wall_state(&self, location: (u8, u8)) -> Option<Orientation>;
    fn apply_move(&mut self, candidate: &Move, player: Player) -> Result<(), ()> {
        match candidate {
            Move::AddWall {
                location,
                orientation,
            } => self.add_wall(player, *location, *orientation),
            Move::MoveTo(x, y) => self.move_token(player, (*x, *y)),
        }
    }

    fn player_location(&self, player: Player) -> (u8, u8);

    fn distance_to_goal(&self, player: Player) -> Option<u8> {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;

        fn p1_d2g((_, y): (u8, u8)) -> u8 {
            8 - y
        }
        fn p2_d2g((_, y): (u8, u8)) -> u8 {
            y
        }
        let d2g = match player {
            Player::Player1 => p1_d2g,
            Player::Player2 => p2_d2g,
        };

        if d2g(self.player_location(player)) == 0 {
            return Some(0);
        }

        let mut costs = [[0xffu8; 9]; 9];
        let mut heap = BinaryHeap::with_capacity(81);

        let starting_loc = self.player_location(player);
        costs[starting_loc.0 as usize][starting_loc.1 as usize] = 0;
        heap.push(Reverse((d2g(starting_loc), starting_loc)));

        while let Some(Reverse((_, loc))) = heap.pop() {
            let neighbors = [
                Direction::Down,
                Direction::Up,
                Direction::Left,
                Direction::Right,
            ]
            .iter()
            .filter_map(|d| d.shift(loc))
            .filter(|(nx, ny)| self.is_passible(loc, (*nx, *ny)));

            let cost = costs[loc.0 as usize][loc.1 as usize] + 1;
            for neighbor in neighbors {
                if d2g(neighbor) == 0 {
                    return Some(cost);
                }
                if costs[neighbor.0 as usize][neighbor.1 as usize] != 0xff {
                    continue;
                }
                costs[neighbor.0 as usize][neighbor.1 as usize] = cost;
                heap.push(Reverse((cost + d2g(neighbor), neighbor)));
            }
        }

        None
    }

    fn is_passible(&self, location: (u8, u8), new_location: (u8, u8)) -> bool;

    fn legal_moves(&self, player: Player) -> Vec<Move> {
        all_moves()
            .filter(|mov| self.is_legal(player, mov))
            .collect()
    }
}

fn all_moves() -> impl Iterator<Item = Move> {
    let adds_walls = [Orientation::Horizontal, Orientation::Vertical]
        .iter()
        .map(|x| *x)
        .flat_map(|o| {
            (0..8).flat_map(move |y| {
                (0..8).map(move |x| Move::AddWall {
                    orientation: o,
                    location: (x, y),
                })
            })
        });

    let locations = (0..9).flat_map(move |x| (0..9).map(move |y| Move::MoveTo(x, y)));

    locations.chain(adds_walls)
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, FromStr, Display)]
#[display(style = "kebab-case")]
pub enum Player {
    Player1,
    Player2,
}
impl Player {
    pub fn other(&self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
}
impl Direction {
    pub fn shift(&self, position: (u8, u8)) -> Option<(u8, u8)> {
        fn add((ax, ay): &(u8, u8), (bx, by): &(i8, i8)) -> Option<(u8, u8)> {
            let res = (*ax as i8 + bx, *ay as i8 + by);

            if res.0 >= 0 && res.0 < 9 && res.1 >= 0 && res.1 < 9 {
                Some((res.0 as u8, res.1 as u8))
            } else {
                None
            }
        }

        match self {
            Direction::Up => add(&position, &(0, -1)),
            Direction::Down => add(&position, &(0, 1)),
            Direction::Left => add(&position, &(-1, 0)),
            Direction::Right => add(&position, &(1, 0)),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Move {
    AddWall {
        orientation: Orientation,
        location: (u8, u8),
    },
    MoveTo(u8, u8),
}
