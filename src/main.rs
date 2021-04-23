#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WallState {
    Wall,
    Open,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell<WS> {
    right: WS,
    bottom: WS,
    joint: WS,
}

pub type RegularCell = Cell<WallState>;

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    cells: [[RegularCell; 9]; 9],
    player1_loc: (usize, usize),
    player2_loc: (usize, usize),
}

impl Board {
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

        Board {
            cells: [
                open_cells, open_cells, open_cells, open_cells, open_cells, open_cells, open_cells,
                open_cells, open_cells,
            ],
            player1_loc: (4, 0),
            player2_loc: (4, 8),
        }
    }

    fn add_wall(&mut self, location: &(usize, usize), horizontal: bool) {
        if horizontal {
            self.cell_mut(location).bottom = WallState::Wall;
            self.cell_mut(location).joint = WallState::Wall;
            self.cell_mut(&(location.0 + 1, location.1)).bottom = WallState::Wall;
        } else {
            self.cell_mut(location).right = WallState::Wall;
            self.cell_mut(location).joint = WallState::Wall;
            self.cell_mut(&(location.0, location.1 + 1)).right = WallState::Wall;
        }
    }

    pub fn cell(&self, (x, y): &(usize, usize)) -> &RegularCell {
        &self.cells[*y][*x]
    }

    pub fn cell_mut(&mut self, (x, y): &(usize, usize)) -> &mut RegularCell {
        &mut self.cells[*y][*x]
    }

    pub fn location(&self, player: &Player) -> &(usize, usize) {
        match player {
            Player::Player1 => &self.player1_loc,
            Player::Player2 => &self.player2_loc,
        }
    }

    pub fn location_mut(&mut self, player: &Player) -> &mut (usize, usize) {
        match player {
            Player::Player1 => &mut self.player1_loc,
            Player::Player2 => &mut self.player2_loc,
        }
    }

    fn is_legal(&self, player: &Player, candidate_move: &Move) -> bool {
        match candidate_move {
            Move::AddWall {
                location,
                horizontal,
            } => {
                let (x, y) = *location;
                self.cell(&location).joint == WallState::Open
                    && if *horizontal {
                        self.cell(&location).bottom == WallState::Open
                            && self.cell(&(x + 1, y)).bottom == WallState::Open
                    } else {
                        self.cell(&location).right == WallState::Open
                            && self.cell(&(x, y + 1)).right == WallState::Open
                    }
            }
            Move::MoveToken(direction) => {
                let loc = self.location(player);
                direction.shift(loc).is_some()
                    && match direction {
                        Direction::Down => self.cell(&(loc.0, loc.1)).bottom == WallState::Open,
                        Direction::Up => self.cell(&(loc.0, loc.1 - 1)).bottom == WallState::Open,
                        Direction::Left => self.cell(&(loc.0 - 1, loc.1)).right == WallState::Open,
                        Direction::Right => self.cell(&(loc.0, loc.1)).right == WallState::Open,
                    }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
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
    fn shift(&self, position: &(usize, usize)) -> Option<(usize, usize)> {
        fn add((ax, ay): &(usize, usize), (bx, by): &(isize, isize)) -> Option<(usize, usize)> {
            let res = (*ax as isize + bx, *ay as isize + by);

            if res.0 >= 0 && res.0 < 9 && res.1 >= 0 && res.1 < 9 {
                Some((res.0 as usize, res.1 as usize))
            } else {
                None
            }
        }

        match self {
            Direction::Up => add(position, &(0, -1)),
            Direction::Down => add(position, &(0, 1)),
            Direction::Left => add(position, &(-1, 0)),
            Direction::Right => add(position, &(1, 0)),
        }
    }
}

pub enum Move {
    AddWall {
        horizontal: bool,
        location: (usize, usize),
    },
    MoveToken(Direction),
}

mod display;

fn main() -> Result<(), display::DisplayError> {
    let mut b = Board::empty();
    let mut candidate = Move::MoveToken(Direction::Down);
    let mut d = display::Display::new();
    let mut player = Player::Player1;

    loop {
        d.show(&b)?;
        d.get_move(&b, &player, &mut candidate)?;
        match candidate {
            Move::AddWall {
                location,
                horizontal,
            } => {
                b.add_wall(&location, horizontal);
            }
            Move::MoveToken(d) => {
                if let Some(new_loc) = d.shift(&b.location(&player)) {
                    *b.location_mut(&player) = new_loc;
                }
            }
        }
        player = player.other();
    }
}
