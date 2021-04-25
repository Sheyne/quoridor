use std::{
    convert::{TryFrom, TryInto},
    num::NonZeroU8,
    usize,
};

use crate::Player;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Board {
    cells: [u8; 9 * 2 + 8],
    player1_pos: Position,
    player2_pos: Position,
    player1_walls: u8,
    player2_walls: u8,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Position(NonZeroU8);

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum State {
    Open = 0,
    Occupied = 1,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Kind {
    Right = 0,
    Bottom = 1,
    Joint = 2,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Board {
    pub fn is_passible(&self, (x, y): (u8, u8), direction: Direction) -> bool {
        match direction {
            Direction::Right => x < 8 && State::Open == self.get((x, y), Kind::Right),
            Direction::Down => y < 8 && State::Open == self.get((x, y), Kind::Bottom),
            Direction::Up => y > 0 && State::Open == self.get((x, y - 1), Kind::Bottom),
            Direction::Left => x > 0 && State::Open == self.get((x - 1, y), Kind::Right),
        }
    }

    pub fn neighbors<'a>(&'a self, (x, y): (u8, u8)) -> impl Iterator<Item = (u8, u8)> + 'a {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .iter()
        .map(|x| *x)
        .filter(move |d| self.is_passible((x, y), *d))
        .map(move |d| match d {
            Direction::Right => (x + 1, y),
            Direction::Left => (x - 1, y),
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
        })
    }

    pub fn player_location(&self, player: super::Player) -> (usize, usize) {
        match player {
            super::Player::Player1 => self.player1_pos,
            super::Player::Player2 => self.player2_pos,
        }
        .into()
    }

    pub fn distance_to_goal(&self, player: super::Player) -> Option<u8> {
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
            |p| self.neighbors(*p).map(|p| (p, 1)),
            heuristic,
            |p| heuristic(p) == 0,
        )
        .map(|(p, _)| p.len() as u8)
    }

    pub fn bit_idx(pos: (u8, u8), kind: Kind) -> u8 {
        let (x, y) = pos.into();
        let offset = (kind as u8) * 9 * 8;
        // offset
        //     + match kind {
        //         Kind::Right | Kind::Joint => y * 8 + x,
        //         Kind::Bottom => y * 9 + x,
        //     }
        offset + y * ((kind as u8) % 2 + 8) + x
    }

    pub fn get(&self, pos: (u8, u8), kind: Kind) -> State {
        let bit = Board::bit_idx(pos, kind);
        let byte = bit / 8;
        let byte_bit = bit % 8;
        let bit_state = self.cells[byte as usize] >> byte_bit & 1;
        unsafe { std::mem::transmute(bit_state) }
    }
    pub fn set(&mut self, pos: (u8, u8), kind: Kind, state: State) {
        let bit = Board::bit_idx(pos, kind);
        let byte = bit / 8;
        let byte_bit = bit % 8;

        match state {
            State::Open => self.cells[byte as usize] &= 0xff ^ (1 << byte_bit),
            State::Occupied => self.cells[byte as usize] |= 1 << byte_bit,
        }
    }
}

impl From<Position> for (usize, usize) {
    fn from(p: Position) -> Self {
        let Position(p) = p;
        let p = p.get() - 1;
        ((p % 9) as usize, (p / 9) as usize)
    }
}
impl TryFrom<(usize, usize)> for Position {
    type Error = ();

    fn try_from((x, y): (usize, usize)) -> Result<Self, ()> {
        let position = (y * 9 + x + 1).try_into().map_err(|_| ())?;
        NonZeroU8::new(position).map(Self).ok_or(())
    }
}

impl Position {
    pub unsafe fn new_unchecked(idx: u8) -> Position {
        Position(NonZeroU8::new_unchecked(idx + 1))
    }
    pub fn new(idx: u8) -> Option<Position> {
        if idx < 9 * 9 {
            unsafe { Some(Position(NonZeroU8::new_unchecked(idx + 1))) }
        } else {
            None
        }
    }

    pub fn idx(self) -> u8 {
        self.0.get() - 1
    }

    pub fn trans(self, d: Direction) -> Option<Position> {
        let idx = self.idx();
        match d {
            Direction::Down => {
                if idx + 9 < 9 * 9 {
                    Some(unsafe { Position::new_unchecked(idx + 9) })
                } else {
                    None
                }
            }
            Direction::Up => idx
                .checked_sub(9)
                .map(|idx| unsafe { Position::new_unchecked(idx) }),
            Direction::Left => {
                if idx % 9 != 0 {
                    Some(unsafe { Position::new_unchecked(idx - 1) })
                } else {
                    None
                }
            }
            Direction::Right => {
                if idx % 9 != 8 {
                    Some(unsafe { Position::new_unchecked(idx + 1) })
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_conversions() {
        fn check(a: usize, b: usize) {
            let p: Position = (a, b).try_into().unwrap();
            let (x, y) = p.into();
            assert_eq!((x, y), (a, b));
        }
        check(1, 2);
        check(0, 0);
        check(5, 8);
        check(8, 8);
    }

    #[test]
    fn test_position_movement() {
        let initial: Position = (4, 5).try_into().unwrap();
        assert_eq!((4, 6), initial.trans(Direction::Down).unwrap().into());

        let initial: Position = (4, 8).try_into().unwrap();
        assert!(initial.trans(Direction::Down).is_none());

        let initial: Position = (4, 5).try_into().unwrap();
        assert_eq!((4, 4), initial.trans(Direction::Up).unwrap().into());

        let initial: Position = (4, 0).try_into().unwrap();
        assert!(initial.trans(Direction::Up).is_none());

        let initial: Position = (4, 5).try_into().unwrap();
        assert_eq!((3, 5), initial.trans(Direction::Left).unwrap().into());

        let initial: Position = (0, 5).try_into().unwrap();
        assert!(initial.trans(Direction::Left).is_none());

        let initial: Position = (4, 5).try_into().unwrap();
        assert_eq!((5, 5), initial.trans(Direction::Right).unwrap().into());

        let initial: Position = (8, 5).try_into().unwrap();
        assert!(initial.trans(Direction::Right).is_none());
    }

    #[test]
    fn test_copying_board() {
        let mut board = crate::Board::empty();
        board.add_wall(&(7, 7), true);
        assert_eq!(board.cell(&(8, 7)).bottom, crate::WallState::Wall);

        let packed: Board = board.into();
        assert_eq!(packed.get((8, 7), Kind::Bottom), State::Occupied);

        let returned: crate::Board = packed.into();

        assert_eq!(returned.cell(&(8, 7)).bottom, crate::WallState::Wall);
    }

    #[test]
    fn test_calculating_distances() {
        let mut board = crate::Board::empty();
        let packed: Board = board.clone().into();
        assert_eq!(Some(9), packed.distance_to_goal(Player::Player1));
        assert_eq!(Some(9), packed.distance_to_goal(Player::Player2));

        board.add_wall(&(3, 7), true);
        let packed: Board = board.clone().into();
        assert_eq!(Some(10), packed.distance_to_goal(Player::Player2));
        assert_eq!(Some(10), packed.distance_to_goal(Player::Player1));
    }
}

impl From<super::Board> for Board {
    fn from(board: super::Board) -> Self {
        let mut res = Board {
            cells: [0; 26],
            player1_pos: board.player1_loc.try_into().unwrap(),
            player2_pos: board.player2_loc.try_into().unwrap(),
            player1_walls: board.player1_walls as u8,
            player2_walls: board.player2_walls as u8,
        };

        for y in 0..9u8 {
            for x in 0..9u8 {
                let loc = (x, y);
                let cell = board.cell(&(x as usize, y as usize));
                if x != 8 {
                    res.set(loc, Kind::Right, cell.right.into());
                }
                if y != 8 {
                    res.set(loc, Kind::Bottom, cell.bottom.into());
                }
                if x != 8 && y != 8 {
                    res.set(loc, Kind::Joint, cell.joint.into());
                }
            }
        }

        res
    }
}

impl From<Board> for super::Board {
    fn from(board: Board) -> Self {
        let mut res = super::Board::empty();
        res.player1_loc = board.player1_pos.into();
        res.player2_loc = board.player2_pos.into();
        res.player1_walls = board.player1_walls as usize;
        res.player2_walls = board.player2_walls as usize;

        for y in 0..9u8 {
            for x in 0..9u8 {
                let loc = (x, y);
                let cell = res.cell_mut(&(x as usize, y as usize));
                if x != 8 {
                    cell.right = board.get(loc, Kind::Right).into();
                }
                if y != 8 {
                    cell.bottom = board.get(loc, Kind::Bottom).into();
                }
                if x != 8 && y != 8 {
                    cell.joint = board.get(loc, Kind::Joint).into();
                }
            }
        }

        res
    }
}

impl From<super::WallState> for State {
    fn from(s: super::WallState) -> Self {
        match s {
            super::WallState::Open => State::Open,
            super::WallState::Wall => State::Occupied,
        }
    }
}

impl From<State> for super::WallState {
    fn from(s: State) -> Self {
        match s {
            State::Open => super::WallState::Open,
            State::Occupied => super::WallState::Wall,
        }
    }
}
