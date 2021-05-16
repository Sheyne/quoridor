use pyo3::prelude::*;

use quoridor_game::{bitpacked::BoardV2, Board, Move, Player};
#[pyclass]
#[derive(Clone)]
pub struct Game {
    board: BoardV2,
    current_player: Player,
    swapped: bool,
}

#[pymethods]
impl Game {
    #[new]
    pub fn new() -> Game {
        Game {
            board: BoardV2::empty(),
            current_player: Player::Player1,
            swapped: false,
        }
    }

    pub fn swap_players(&mut self) {
        self.swapped = !self.swapped;
    }

    pub fn clone(&self) -> Game {
        std::clone::Clone::clone(self)
    }

    pub fn as_str(&self) -> String {
        format!("{} {} {}", self.swapped, self.current_player, self.board.repr_string())
    }

    #[staticmethod]
    pub fn from_str(repr: &str) -> Option<Game> {
        let (swapped, repr) = repr.split_once(' ')?;
        let (current_player, repr) = repr.split_once(' ')?;

        Some(Game {
            swapped: swapped.parse().ok()?,
            current_player: current_player.parse().ok()?,
            board: BoardV2::from_repr_string(repr)?,
        })
    }

    pub fn add_wall(&mut self, x: u8, y: u8, orientation: u8) -> bool {
        apply_move(
            self,
            Move::AddWall {
                location: (x, y),
                orientation: match orientation {
                    0 => quoridor_game::Orientation::Horizontal,
                    1 => quoridor_game::Orientation::Vertical,
                    _ => return false,
                },
            },
        )
    }

    pub fn available_walls(&self, player: u8) -> u8 {
        self.board.available_walls(self.map_player(player))
    }

    pub fn can_add_wall(&self, x: u8, y: u8, orientation: u8) -> bool {
        self.board.is_legal(
            self.current_player,
            &Move::AddWall {
                location: (x, y),
                orientation: match orientation {
                    0 => quoridor_game::Orientation::Horizontal,
                    1 => quoridor_game::Orientation::Vertical,
                    _ => return false,
                },
            },
        )
    }

    pub fn can_move_to(&self, new_location: (u8, u8)) -> bool {
        let dirs = [
            quoridor_game::Direction::Up,
            quoridor_game::Direction::Down,
            quoridor_game::Direction::Left,
            quoridor_game::Direction::Right,
        ];

        let current_loc = self.board.player_location(self.current_player);

        let direction = dirs
            .iter()
            .filter_map(|d| {
                if let Some(nl) = d.shift(current_loc) {
                    if nl == new_location {
                        return Some(d);
                    }
                }
                None
            })
            .next();

        if let Some(direction) = direction {
            self.board
                .is_legal(self.current_player, &Move::MoveToken(*direction))
        } else {
            false
        }
    }

    pub fn distance_to_goal(&self, player: u8) -> i8 {
        self.board
            .distance_to_goal(self.map_player(player))
            .map(|a| a as i8)
            .unwrap_or(-1)
    }

    pub fn move_token_to(&mut self, new_location: (u8, u8)) -> bool {
        let dirs = [
            quoridor_game::Direction::Up,
            quoridor_game::Direction::Down,
            quoridor_game::Direction::Left,
            quoridor_game::Direction::Right,
        ];

        let current_loc = self.board.player_location(self.current_player);

        let direction = dirs
            .iter()
            .filter_map(|d| {
                if let Some(nl) = d.shift(current_loc) {
                    if nl == new_location {
                        return Some(d);
                    }
                }
                None
            })
            .next();

        if let Some(direction) = direction {
            apply_move(self, Move::MoveToken(*direction))
        } else {
            false
        }
    }

    pub fn move_token(&mut self, direction: u8) -> bool {
        apply_move(
            self,
            Move::MoveToken(match direction {
                0 => quoridor_game::Direction::Up,
                1 => quoridor_game::Direction::Down,
                2 => quoridor_game::Direction::Left,
                3 => quoridor_game::Direction::Right,
                _ => return false,
            }),
        )
    }

    pub fn current_player(&self) -> u8 {
        let mut player = self.current_player;
        if self.swapped {
            player = player.other();
        }

        match player {
            Player::Player1 => 1,
            Player::Player2 => 2,
        }
    }

    pub fn get_location(&self, player: u8) -> (u8, u8) {
        self.board.player_location(self.map_player(player))
    }

    pub fn get_wall_status(&self, x: u8, y: u8) -> u8 {
        match self.board.get_wall_state((x, y)) {
            None => 0,
            Some(quoridor_game::Orientation::Horizontal) => 1,
            Some(quoridor_game::Orientation::Vertical) => 2,
        }
    }

    pub fn is_swapped(&self) -> bool {
        self.swapped
    }

    pub fn is_passible(&self, x: u8, y: u8, direction: u8) -> bool {
        let direction = match direction {
            0 => quoridor_game::Direction::Up,
            1 => quoridor_game::Direction::Down,
            2 => quoridor_game::Direction::Left,
            3 => quoridor_game::Direction::Right,
            _ => return false,
        };

        self.board.is_passible((x, y), direction)
    }


    // pub fn canonical_form(&self) -> Game {
    //     if self.current_player == Player::Player1 {
    //         self.clone()
    //     } else {
    //         Game {
    //             board: self.board.flip(),
    //             current_player: Player::Player1,
    //             false
    //         }
    //     }
    // }
}

impl Game {
    fn map_player(&self, player: u8) -> Player {
        let player = match player {
            1 => Player::Player1,
            2 => Player::Player2,
            _ => todo!(),
        };

        if self.swapped {
            player.other()
        } else {
            player
        }
    }
}

fn apply_move(game: &mut Game, mov: Move) -> bool {
    if game.board.is_legal(game.current_player, &mov) {
        if game.board.apply_move(&mov, game.current_player).is_ok() {
            game.current_player = game.current_player.other();
            return true;
        }
    }
    false
}

#[pymodule]
fn quoridor_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Game>()?;
    Ok(())
}
