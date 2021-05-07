use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WallState {
    Wall,
    Open,
}

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell<WS> {
    pub right: WS,
    pub bottom: WS,
    pub joint: WS,
}

pub type RegularCell = Cell<WallState>;

pub trait Board {
    fn empty() -> Self;

    fn add_wall(&mut self, location: (u8, u8), orientation: Orientation);
    fn move_token(&mut self, player: Player, direction: Direction);
    fn is_legal(&self, player: Player, candidate_move: &Move) -> bool;

    fn apply_move(&mut self, candidate: &Move, player: Player) {
        match candidate {
            Move::AddWall {
                location,
                orientation,
            } => {
                self.add_wall(*location, *orientation);
            }
            Move::MoveToken(d) => {
                self.move_token(player, *d);
            }
        }
    }

    fn player_location(&self, player: super::Player) -> (u8, u8);

    fn distance_to_goal(&self, player: super::Player) -> Option<u8> {
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

impl Board for BoardV1 {
    fn empty() -> Self {
        let open_cell = RegularCell {
            right: WallState::Open,
            bottom: WallState::Open,
            joint: WallState::Open,
        };

        let open_cells = [
            open_cell, open_cell, open_cell, open_cell, open_cell, open_cell, open_cell, open_cell,
            open_cell,
        ];

        BoardV1 {
            cells: [
                open_cells, open_cells, open_cells, open_cells, open_cells, open_cells, open_cells,
                open_cells, open_cells,
            ],
            player1_loc: (4, 0),
            player2_loc: (4, 8),
            player1_walls: 10,
            player2_walls: 10,
        }
    }

    fn is_passible(&self, loc: (u8, u8), direction: Direction) -> bool {
        direction.shift(loc).is_some()
            && match direction {
                Direction::Down => self.cell(&(loc.0, loc.1)).bottom == WallState::Open,
                Direction::Up => self.cell(&(loc.0, loc.1 - 1)).bottom == WallState::Open,
                Direction::Left => self.cell(&(loc.0 - 1, loc.1)).right == WallState::Open,
                Direction::Right => self.cell(&(loc.0, loc.1)).right == WallState::Open,
            }
    }

    fn player_location(&self, player: super::Player) -> (u8, u8) {
        let loc = self.location(&player);
        (loc.0 as u8, loc.1 as u8)
    }

    fn move_token(&mut self, player: Player, direction: Direction) {
        let loc = self.location(&player);
        let loc = (loc.0 as u8, loc.1 as u8);
        if let Some(new_loc) = direction.shift(loc) {
            *self.location_mut(&player) = new_loc;
        }
    }

    fn add_wall(&mut self, location: (u8, u8), orientation: Orientation) {
        match orientation {
            Orientation::Horizontal => {
                self.cell_mut(&location).bottom = WallState::Wall;
                self.cell_mut(&location).joint = WallState::Wall;
                self.cell_mut(&(location.0 + 1, location.1)).bottom = WallState::Wall;
            }
            Orientation::Vertical => {
                self.cell_mut(&location).right = WallState::Wall;
                self.cell_mut(&location).joint = WallState::Wall;
                self.cell_mut(&(location.0, location.1 + 1)).right = WallState::Wall;
            }
        }
    }

    fn is_legal(&self, player: Player, candidate_move: &Move) -> bool {
        match candidate_move {
            Move::AddWall {
                location,
                orientation,
            } => {
                let (x, y) = location;
                let unfilled = self.cell(&location).joint == WallState::Open
                    && match orientation {
                        Orientation::Horizontal => {
                            self.cell(&location).bottom == WallState::Open
                                && self.cell(&(x + 1, *y)).bottom == WallState::Open
                        }
                        Orientation::Vertical => {
                            self.cell(&location).right == WallState::Open
                                && self.cell(&(*x, y + 1)).right == WallState::Open
                        }
                    };

                let mut hypo = self.clone();
                hypo.add_wall(*location, *orientation);

                unfilled
                    && hypo.distance_to_goal(Player::Player1).is_some()
                    && hypo.distance_to_goal(Player::Player2).is_some()
            }
            Move::MoveToken(direction) => {
                let (x, y) = self.location(&player);
                self.is_passible((*x as u8, *y as u8), *direction)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct BoardV1 {
    pub cells: [[RegularCell; 9]; 9],
    pub player1_loc: (u8, u8),
    pub player2_loc: (u8, u8),
    pub player1_walls: u8,
    pub player2_walls: u8,
}

impl BoardV1 {
    pub fn cell(&self, (x, y): &(u8, u8)) -> &RegularCell {
        &self.cells[*y as usize][*x as usize]
    }

    pub fn cell_mut(&mut self, (x, y): &(u8, u8)) -> &mut RegularCell {
        &mut self.cells[*y as usize][*x as usize]
    }

    pub fn location(&self, player: &Player) -> &(u8, u8) {
        match player {
            Player::Player1 => &self.player1_loc,
            Player::Player2 => &self.player2_loc,
        }
    }

    pub fn location_mut(&mut self, player: &Player) -> &mut (u8, u8) {
        match player {
            Player::Player1 => &mut self.player1_loc,
            Player::Player2 => &mut self.player2_loc,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug)]
pub enum Move {
    AddWall {
        orientation: Orientation,
        location: (u8, u8),
    },
    MoveToken(Direction),
}
