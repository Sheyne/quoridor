pub mod ai;
pub mod bitpacked;
pub mod v1;

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

    fn add_wall(&mut self, player: Player, location: (u8, u8), orientation: Orientation);
    fn move_token(&mut self, player: Player, direction: Direction);
    fn is_legal(&self, player: Player, candidate_move: &Move) -> bool;
    fn get_wall_state(&self, location: (u8, u8)) -> Option<Orientation>;
    fn available_walls(&self, player: Player) -> u8;
    fn apply_move(&mut self, candidate: &Move, player: Player) {
        match candidate {
            Move::AddWall {
                location,
                orientation,
            } => {
                self.add_wall(player, *location, *orientation);
            }
            Move::MoveToken(d) => {
                self.move_token(player, *d);
            }
        }
    }

    fn player_location(&self, player: Player) -> (u8, u8);

    fn distance_to_goal(&self, player: Player) -> Option<u8> {
        let (x, y) = self.player_location(player);
        let initial = (x as u8, y as u8);
        fn p1_heuristic((_, y): &(u8, u8)) -> u8 {
            8 - *y
        }
        fn p2_heuristic((_, y): &(u8, u8)) -> u8 {
            *y
        }

        let heuristic = match player {
            Player::Player1 => p1_heuristic,
            Player::Player2 => p2_heuristic,
        };

        pathfinding::prelude::astar(
            &initial,
            |p| neighbors(self, *p).map(|p| (p, 1)),
            heuristic,
            |p| heuristic(p) == 0,
        )
        .map(|(p, _)| p.len() as u8)
    }

    fn is_passible(&self, location: (u8, u8), direction: Direction) -> bool;
}

fn neighbors<'a, B: Board + ?Sized>(
    board: &'a B,
    (x, y): (u8, u8),
) -> impl Iterator<Item = (u8, u8)> + 'a {
    [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ]
    .iter()
    .map(|x| *x)
    .filter(move |d| board.is_passible((x, y), *d))
    .map(move |d| match d {
        Direction::Right => (x + 1, y),
        Direction::Left => (x - 1, y),
        Direction::Up => (x, y - 1),
        Direction::Down => (x, y + 1),
    })
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Move {
    AddWall {
        orientation: Orientation,
        location: (u8, u8),
    },
    MoveToken(Direction),
}
