use pyo3::prelude::*;

use quoridor_game::{bitpacked::BoardV2, Board, Move, Player};
#[pyclass]
pub struct Game {
    board: BoardV2,
    current_player: Player,
}


#[pymethods]
impl Game {
    #[new]
    pub fn new() -> Game {
        Game {
            board: BoardV2::empty(),
            current_player: Player::Player1,
        }
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
        match self.current_player {
            Player::Player1 => 1,
            Player::Player2 => 2,
        }
    }

    pub fn get_location(&self, player: u8) -> (u8, u8) {
        let player = match player {
            1 => Player::Player1,
            2 => Player::Player2,
            _ => panic!(),
        };

        self.board.player_location(player)
    }

    pub fn get_wall_status(&self, x: u8, y: u8) -> u8 {
        match self.board.get_wall_state((x, y)) {
            None => 0,
            Some(quoridor_game::Orientation::Horizontal) => 1,
            Some(quoridor_game::Orientation::Vertical) => 2,
        }
    }
}

fn apply_move(game: &mut Game, mov: Move) -> bool {
    if game.board.is_legal(game.current_player, &mov) {
        game.board.apply_move(&mov, game.current_player);
        game.current_player = game.current_player.other();
        true
    } else {
        false
    }
}

#[pymodule]
fn quoridor_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Game>()?;
    Ok(())
}
