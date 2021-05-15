use crate::{Board, Direction, Move, Orientation, Player};
use fxhash::FxHasher;
use std::{
    convert::{TryFrom, TryInto},
    hash::{Hash, Hasher},
    num::NonZeroU8,
};

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

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

    fn add_wall(
        &mut self,
        player: Player,
        location: (u8, u8),
        orientation: crate::Orientation,
    ) -> Result<(), ()> {
        BoardV2::bit_mask(location)
            .and_then(|mask| {
                if (self.vertical | self.horizontal) & mask != 0 {
                    return None;
                }
                let bitset = match orientation {
                    crate::Orientation::Horizontal => &mut self.horizontal,
                    crate::Orientation::Vertical => &mut self.vertical,
                };
                *bitset |= mask;
                match player {
                    Player::Player1 => self.player1_walls -= 1,
                    Player::Player2 => self.player2_walls -= 1,
                }
                Some(())
            })
            .ok_or(())
    }

    fn move_token(&mut self, player: Player, direction: crate::Direction) -> Result<(), ()> {
        self.set_player_location(
            player,
            direction.shift(self.player_location(player)).ok_or(())?,
        )
    }

    fn get_wall_state(&self, location: (u8, u8)) -> Option<Orientation> {
        if BoardV2::bit_mask(location)
            .map(|x| x & self.horizontal != 0)
            .unwrap_or(false)
        {
            Some(Orientation::Horizontal)
        } else if BoardV2::bit_mask(location)
            .map(|x| x & self.vertical != 0)
            .unwrap_or(false)
        {
            Some(Orientation::Vertical)
        } else {
            None
        }
    }

    fn is_probably_legal(&self, player: Player, candidate_move: &Move) -> bool {
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

                if BoardV2::bit_mask(*location).is_none() {
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
                unfilled
            }
            Move::MoveToken(direction) => {
                self.is_passible(self.player_location(player), *direction)
            }
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

                if BoardV2::bit_mask(*location).is_none() {
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
                let added_wall = hypo.add_wall(player, *location, *orientation).is_ok();
                let p1_can_exit = hypo.distance_to_goal(Player::Player1).is_some();
                let p2_can_exit = hypo.distance_to_goal(Player::Player2).is_some();

                added_wall && unfilled && p1_can_exit && p2_can_exit
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
    pub fn fx_hash(&self, hasher: &mut FxHasher) {
        hasher.write_u64(self.horizontal);
        hasher.write_u64(self.vertical);
        hasher.write_u8(self.player1_pos.0.into());
        hasher.write_u8(self.player2_pos.0.into());
        hasher.write_u8(self.player2_walls);
        hasher.write_u8(self.player2_walls);
    }

    pub fn flip(&self) -> BoardV2 {
        fn flip_bytes(n: u64) -> u64 {
            let a = (n >> 56) & 0xff;
            let b = (n >> 48) & 0xff;
            let c = (n >> 40) & 0xff;
            let d = (n >> 32) & 0xff;
            let e = (n >> 24) & 0xff;
            let f = (n >> 16) & 0xff;
            let g = (n >> 8) & 0xff;
            let h = (n >> 0) & 0xff;

            (a << 0)
                | (b << 8)
                | (c << 16)
                | (d << 24)
                | (e << 32)
                | (f << 40)
                | (g << 48) << (h << 56)
        }

        fn flip_player(l: Position) -> Position {
            let (x, y) = l.into();
            (x, 8 - y).try_into().unwrap()
        }

        BoardV2 {
            horizontal: flip_bytes(self.horizontal),
            vertical: flip_bytes(self.vertical),
            player1_pos: flip_player(self.player2_pos),
            player2_pos: flip_player(self.player1_pos),
            player1_walls: self.player2_walls,
            player2_walls: self.player1_walls,
        }
    }

    pub fn repr_string(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.horizontal,
            self.vertical,
            self.player1_pos,
            self.player2_pos,
            self.player1_walls,
            self.player2_walls,
        )
    }

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

    fn set_player_location(&mut self, player: Player, pos: (u8, u8)) -> Result<(), ()> {
        *match player {
            super::Player::Player1 => &mut self.player1_pos,
            super::Player::Player2 => &mut self.player2_pos,
        } = pos.try_into()?;
        Ok(())
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

pub fn can_reach_goal_v2(board: &impl Board, player: Player) -> bool {
    let y_range = match player {
        Player::Player1 => [8u8, 7, 6, 5, 4, 3, 2, 1, 0],
        Player::Player2 => [0, 1, 2, 3, 4, 5, 6, 7, 8u8],
    };

    let goal_y = match player {
        Player::Player1 => 8u8,
        Player::Player2 => 0,
    };

    let mut hit_map = [[false; 9]; 9];
    let mut solved_map = [[false; 9]; 9];
    let (player_x, player_y) = board.player_location(player);
    hit_map[player_y as usize][player_x as usize] = true;

    'outer: loop {
        for y in &y_range {
            let y = *y;
            for x in 0..9u8 {
                if hit_map[y as usize][x as usize] && !solved_map[y as usize][x as usize] {
                    solved_map[y as usize][x as usize] = true;
                    let directions = [
                        Direction::Down,
                        Direction::Up,
                        Direction::Left,
                        Direction::Right,
                    ]
                    .iter();

                    let mut found_one = false;
                    for direction in directions {
                        let direction = *direction;
                        if let Some((nx, ny)) = direction.shift((x, y)) {
                            if !hit_map[ny as usize][nx as usize] {
                                if board.is_passible((x, y), direction) {
                                    if ny == goal_y {
                                        return true;
                                    }
                                    hit_map[ny as usize][nx as usize] = true;
                                    found_one = true;
                                }
                            }
                        }
                    }
                    if found_one {
                        continue 'outer;
                    }
                }
            }
        }
        return false;
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
    fn can_reach_goal_works() {
        let board = BoardV2 {
            horizontal: 18015223143202816,
            vertical: 2147483648,
            player1_pos: Position(23.try_into().unwrap()),
            player2_pos: Position(68.try_into().unwrap()),
            player1_walls: 8,
            player2_walls: 8,
        };

        board.is_legal(
            Player::Player1,
            &Move::AddWall {
                location: (2, 2),
                orientation: Orientation::Horizontal,
            },
        );
    }

    #[test]
    fn cant_create_overlapping_walls() {
        let mut board = BoardV2::empty();
        board
            .add_wall(Player::Player1, (5, 5), Orientation::Horizontal)
            .unwrap();
        assert!(!board.is_legal(
            Player::Player2,
            &Move::AddWall {
                orientation: Orientation::Horizontal,
                location: (5, 5)
            }
        ));
        assert!(board
            .add_wall(Player::Player2, (5, 5), Orientation::Vertical)
            .is_err());
    }
    #[test]
    fn cant_create_overlapping_walls_anywhere() {
        fn check_location(l: (u8, u8), first_orientation: Orientation) {
            let mut board = BoardV2::empty();
            board
                .add_wall(Player::Player1, l, first_orientation)
                .unwrap();
            assert!(!board.is_legal(
                Player::Player2,
                &Move::AddWall {
                    orientation: first_orientation,
                    location: l
                }
            ));
            assert!(board
                .add_wall(Player::Player2, l, first_orientation)
                .is_err());
            assert!(!board.is_legal(
                Player::Player2,
                &Move::AddWall {
                    orientation: first_orientation.other(),
                    location: l
                }
            ));
            assert!(board
                .add_wall(Player::Player2, l, first_orientation.other())
                .is_err());
        }

        for y in 0..=7 {
            for x in 0..=7 {
                for o in &[Orientation::Vertical, Orientation::Horizontal] {
                    check_location((x, y), *o);
                }
            }
        }
    }

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
    fn cant_add_walls_past_edge() {
        let board = BoardV2::empty();
        assert!(!board.is_legal(
            Player::Player1,
            &Move::AddWall {
                location: (2, 8),
                orientation: Orientation::Horizontal
            }
        ))
    }

    #[test]
    fn test_h_walls_edge() {
        let mut board = BoardV2::empty();
        board
            .add_wall(Player::Player1, (7, 2), Orientation::Horizontal)
            .unwrap();
        assert!(!board.is_passible_down((7, 2)));
        assert!(!board.is_passible_down((8, 2)));
    }
    #[test]
    fn test_h_walls_block_things() {
        let mut board = BoardV2::empty();
        board
            .add_wall(Player::Player1, (1, 2), Orientation::Horizontal)
            .unwrap();
        assert!(board.is_passible_down((0, 2)));
        assert!(!board.is_passible_down((1, 2)));
        assert!(!board.is_passible_down((2, 2)));
        assert!(board.is_passible_down((3, 2)));
        assert!(board.is_passible_down((1, 3)));
    }

    #[test]
    fn test_v_walls_edge() {
        let mut board = BoardV2::empty();
        board
            .add_wall(Player::Player1, (2, 7), Orientation::Vertical)
            .unwrap();
        assert!(!board.is_passible_right((2, 7)));
        assert!(!board.is_passible_right((2, 8)));
    }

    #[test]
    fn test_v_walls_block_things() {
        let mut board = BoardV2::empty();
        board
            .add_wall(Player::Player1, (1, 2), Orientation::Vertical)
            .unwrap();
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
        board
            .add_wall(Player::Player1, (1, 2), Orientation::Vertical)
            .unwrap();
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

        board
            .add_wall(Player::Player1, (3, 7), Orientation::Horizontal)
            .unwrap();
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
                        crate::v1::WallState::Open
                    } else {
                        crate::v1::WallState::Wall
                    };
                }
                if y != 8 {
                    cell.bottom = if board.is_passible_down(loc) {
                        crate::v1::WallState::Open
                    } else {
                        crate::v1::WallState::Wall
                    };
                }
                if x != 8 && y != 8 {
                    cell.joint = if BoardV2::bit_mask((x, y))
                        .map(|m| (m & (board.horizontal | board.vertical)) == 0)
                        .unwrap_or(false)
                    {
                        crate::v1::WallState::Open
                    } else {
                        crate::v1::WallState::Wall
                    };
                }
            }
        }

        res
    }
}

impl From<crate::v1::WallState> for State {
    fn from(s: crate::v1::WallState) -> Self {
        match s {
            crate::v1::WallState::Open => State::Open,
            crate::v1::WallState::Wall => State::Occupied,
        }
    }
}

impl From<State> for crate::v1::WallState {
    fn from(s: State) -> Self {
        match s {
            State::Open => crate::v1::WallState::Open,
            State::Occupied => crate::v1::WallState::Wall,
        }
    }
}
