use std::num::NonZeroU8;
use std::{
    convert::{TryFrom, TryInto},
    usize,
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Board([u8; 31], Position, Position);
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
    pub fn get(&self, pos: Position, kind: Kind) -> State {
        let position: usize = pos.idx().into();
        let bit = position * 3 + (kind as usize);
        let byte = bit / 8;
        let byte_bit = bit % 8;
        let bit_state = self.0[byte] >> byte_bit & 1;
        unsafe { std::mem::transmute(bit_state) }
    }
    pub fn set(&mut self, pos: Position, kind: Kind, state: State) {
        let position: usize = pos.idx().into();
        let bit = position * 3 + (kind as usize);
        let byte = bit / 8;
        let byte_bit = bit % 8;

        match state {
            State::Open => self.0[byte] &= 0xff ^ (1 << byte_bit),
            State::Occupied => self.0[byte] |= 1 << byte_bit,
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
}

impl From<super::Board> for Board {
    fn from(board: super::Board) -> Self {
        let mut res = Board(
            [0; 31],
            board.player1_loc.try_into().unwrap(),
            board.player2_loc.try_into().unwrap(),
        );

        for y in 0..9 {
            for x in 0..9 {
                let loc = (x, y);
                let cell = board.cell(&loc);
                let loc: Position = loc.try_into().unwrap();
                res.set(loc, Kind::Right, cell.right.into());
                res.set(loc, Kind::Bottom, cell.bottom.into());
                res.set(loc, Kind::Joint, cell.joint.into());
            }
        }

        res
    }
}

impl From<Board> for super::Board {
    fn from(board: Board) -> Self {
        let mut res = super::Board::empty();
        res.player1_loc = board.1.into();
        res.player2_loc = board.2.into();

        for y in 0..9 {
            for x in 0..9 {
                let loc = (x, y);
                let cell = res.cell_mut(&loc);
                cell.right = board.get(loc.try_into().unwrap(), Kind::Right).into();
                cell.bottom = board.get(loc.try_into().unwrap(), Kind::Bottom).into();
                cell.joint = board.get(loc.try_into().unwrap(), Kind::Joint).into();
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
