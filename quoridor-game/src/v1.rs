use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell<WS> {
    pub right: WS,
    pub bottom: WS,
    pub joint: WS,
}

pub type RegularCell = Cell<WallState>;

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

    fn available_walls(&self, player: Player) -> u8 {
        match player {
            Player::Player1 => self.player1_walls,
            Player::Player2 => self.player2_walls,
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

    fn player_location(&self, player: Player) -> (u8, u8) {
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

    fn add_wall(&mut self, player: Player, location: (u8, u8), orientation: Orientation) {
        match player {
            Player::Player1 => self.player1_walls -= 1,
            Player::Player2 => self.player2_walls -= 1,
        }

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
                if match player {
                    Player::Player1 => self.player1_walls,
                    Player::Player2 => self.player2_walls,
                } == 0
                {
                    return false;
                }

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
                hypo.add_wall(player, *location, *orientation);

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
