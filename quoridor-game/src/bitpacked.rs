use std::{
    convert::{TryFrom, TryInto},
    hash::Hash,
    num::NonZeroU8,
};

use crate::{Board, Direction, Move, Orientation, Player};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct BoardV2 {
    horizontal: u64,
    vertical: u64,
    player1_pos: Position,
    player2_pos: Position,
    player1_walls: u8,
    player2_walls: u8,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct Position(NonZeroU8);

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum State {
    Open = 0,
    Occupied = 1,
}

impl Board for BoardV2 {
    fn empty() -> Self {
        Self {
            horizontal: 0,
            vertical: 0,
            player1_pos: (4, 0).try_into().unwrap(),
            player2_pos: (4, 8).try_into().unwrap(),
            player1_walls: 10,
            player2_walls: 10,
        }
    }

    fn available_walls(&self, player: Player) -> u8 {
        match player {
            Player::Player1 => self.player1_walls,
            Player::Player2 => self.player2_walls,
        }
    }

    fn add_wall(&mut self, player: Player, location: (u8, u8), orientation: crate::Orientation) {
        let bitset = match orientation {
            crate::Orientation::Horizontal => &mut self.horizontal,
            crate::Orientation::Vertical => &mut self.vertical,
        };

        if let Some(mask) = BoardV2::bit_mask(location) {
            *bitset |= mask;
            match player {
                Player::Player1 => self.player1_walls -= 1,
                Player::Player2 => self.player2_walls -= 1,
            }
        }
    }

    fn move_token(&mut self, player: Player, direction: crate::Direction) {
        if let Some(location) = direction.shift(self.player_location(player)) {
            self.set_player_location(player, location);
        }
    }

    fn is_legal(&self, player: Player, candidate_move: &Move) -> bool {
        match candidate_move {
            Move::AddWall {
                location,
                orientation,
            } => {
                if match player {
                    Player::Player1 => self.player1_walls,
                    Player::Player2 => self.player2_walls,
                } == 0
                {
                    return false;
                }

                fn directions_to_mask(poses: impl Iterator<Item = Option<(u8, u8)>>) -> u64 {
                    poses
                        .filter_map(|x| x)
                        .filter_map(|x| BoardV2::bit_mask(x))
                        .fold(0, |acc, x| acc | x)
                }

                let h_mask = directions_to_mask(
                    [
                        Direction::Left.shift(*location),
                        Some(*location),
                        Direction::Right.shift(*location),
                    ]
                    .iter()
                    .map(|x| *x),
                );

                let v_mask = directions_to_mask(
                    [
                        Direction::Up.shift(*location),
                        Some(*location),
                        Direction::Down.shift(*location),
                    ]
                    .iter()
                    .map(|x| *x),
                );

                let unfilled = match orientation {
                    Orientation::Vertical => (self.vertical & v_mask) == 0,
                    Orientation::Horizontal => (self.horizontal & h_mask) == 0,
                };

                let mut hypo = self.clone();
                hypo.add_wall(player, *location, *orientation);

                unfilled
                    && hypo.distance_to_goal(Player::Player1).is_some()
                    && hypo.distance_to_goal(Player::Player2).is_some()
            }
            Move::MoveToken(direction) => {
                self.is_passible(self.player_location(player), *direction)
            }
        }
    }

    fn player_location(&self, player: Player) -> (u8, u8) {
        match player {
            super::Player::Player1 => self.player1_pos,
            super::Player::Player2 => self.player2_pos,
        }
        .into()
    }

    fn is_passible(&self, (x, y): (u8, u8), direction: crate::Direction) -> bool {
        match direction {
            Direction::Right => x < 8 && self.is_passible_right((x, y)),
            Direction::Down => y < 8 && self.is_passible_down((x, y)),
            Direction::Up => y > 0 && self.is_passible_down((x, y - 1)),
            Direction::Left => x > 0 && self.is_passible_right((x - 1, y)),
        }
    }
}

impl BoardV2 {
    fn bit_mask(p: (u8, u8)) -> Option<u64> {
        BoardV2::bit_idx(p).map(|b| 1 << b)
    }

    fn bit_idx((x, y): (u8, u8)) -> Option<u8> {
        if x < 8 && y < 8 {
            Some(x * 8 + y)
        } else {
            None
        }
    }

    fn set_player_location(&mut self, player: Player, pos: (u8, u8)) {
        *match player {
            super::Player::Player1 => &mut self.player1_pos,
            super::Player::Player2 => &mut self.player2_pos,
        } = pos.try_into().unwrap();
    }

    fn is_passible_down(&self, pos: (u8, u8)) -> bool {
        let neighbor = Direction::Left
            .shift(pos)
            .and_then(|np| BoardV2::bit_mask(np).map(|mask| mask & self.horizontal == 0))
            .unwrap_or(true);

        let this = BoardV2::bit_mask(pos)
            .map(|mask| mask & self.horizontal == 0)
            .unwrap_or(true);

        neighbor && this
    }
    fn is_passible_right(&self, pos: (u8, u8)) -> bool {
        let neighbor = Direction::Up
            .shift(pos)
            .and_then(|np| BoardV2::bit_mask(np).map(|mask| mask & self.vertical == 0))
            .unwrap_or(true);

        let this = BoardV2::bit_mask(pos)
            .map(|mask| mask & self.vertical == 0)
            .unwrap_or(true);

        neighbor && this
    }
}

impl From<Position> for (u8, u8) {
    fn from(p: Position) -> Self {
        let Position(p) = p;
        let p = p.get() - 1;
        ((p % 9) as u8, (p / 9) as u8)
    }
}
impl TryFrom<(u8, u8)> for Position {
    type Error = ();

    fn try_from((x, y): (u8, u8)) -> Result<Self, ()> {
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
    use crate::*;

    #[test]
    fn test_position_conversions() {
        fn check(a: u8, b: u8) {
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
    fn test_h_walls_edge() {
        let mut board = BoardV2::empty();
        board.add_wall(Player::Player1, (7, 2), Orientation::Horizontal);
        assert!(!board.is_passible_down((7, 2)));
        assert!(!board.is_passible_down((8, 2)));
    }
    #[test]
    fn test_h_walls_block_things() {
        let mut board = BoardV2::empty();
        board.add_wall(Player::Player1, (1, 2), Orientation::Horizontal);
        assert!(board.is_passible_down((0, 2)));
        assert!(!board.is_passible_down((1, 2)));
        assert!(!board.is_passible_down((2, 2)));
        assert!(board.is_passible_down((3, 2)));
        assert!(board.is_passible_down((1, 3)));
    }

    #[test]
    fn test_v_walls_edge() {
        let mut board = BoardV2::empty();
        board.add_wall(Player::Player1, (2, 7), Orientation::Vertical);
        assert!(!board.is_passible_right((2, 7)));
        assert!(!board.is_passible_right((2, 8)));
    }

    #[test]
    fn test_v_walls_block_things() {
        let mut board = BoardV2::empty();
        board.add_wall(Player::Player1, (1, 2), Orientation::Vertical);
        assert!(board.is_passible_right((1, 1)));
        assert!(!board.is_passible_right((1, 2)));
        assert!(!board.is_passible_right((1, 3)));
        assert!(board.is_passible_right((1, 4)));
        assert!(board.is_passible_right((2, 2)));
        assert!(board.is_passible_right((0, 0)));
    }

    #[test]
    fn test_v_walls_block_things_1() {
        let mut board = crate::v1::BoardV1::empty();
        board.add_wall(Player::Player1, (1, 2), Orientation::Vertical);
        assert!(board.is_passible((1, 1), Direction::Right));
        assert!(!board.is_passible((1, 2), Direction::Right));
        assert!(!board.is_passible((1, 3), Direction::Right));
        assert!(board.is_passible((1, 4), Direction::Right));
        assert!(board.is_passible((2, 2), Direction::Right));
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
    fn test_calculating_distances() {
        let mut board = BoardV2::empty();
        let packed: BoardV2 = board.clone().into();
        assert_eq!(Some(9), packed.distance_to_goal(Player::Player1));
        assert_eq!(Some(9), packed.distance_to_goal(Player::Player2));

        board.add_wall(Player::Player1, (3, 7), Orientation::Horizontal);
        let packed: BoardV2 = board.clone().into();
        assert_eq!(Some(10), packed.distance_to_goal(Player::Player2));
        assert_eq!(Some(10), packed.distance_to_goal(Player::Player1));
    }
}

impl From<BoardV2> for crate::v1::BoardV1 {
    fn from(board: BoardV2) -> Self {
        let mut res = crate::v1::BoardV1::empty();
        res.player1_loc = board.player1_pos.into();
        res.player2_loc = board.player2_pos.into();
        res.player1_walls = board.player1_walls;
        res.player2_walls = board.player2_walls;

        for y in 0..9u8 {
            for x in 0..9u8 {
                let loc = (x, y);
                let cell = res.cell_mut(&loc);
                if x != 8 {
                    cell.right = if board.is_passible_right(loc) {
                        crate::WallState::Open
                    } else {
                        crate::WallState::Wall
                    };
                }
                if y != 8 {
                    cell.bottom = if board.is_passible_down(loc) {
                        crate::WallState::Open
                    } else {
                        crate::WallState::Wall
                    };
                }
                if x != 8 && y != 8 {
                    cell.joint = if BoardV2::bit_mask((x, y))
                        .map(|m| (m & (board.horizontal | board.vertical)) == 0)
                        .unwrap_or(false)
                    {
                        crate::WallState::Open
                    } else {
                        crate::WallState::Wall
                    };
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
