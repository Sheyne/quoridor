mod utils;

use quoridor_game::{bitpacked::BoardV2, Board, Move, Player};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
#[wasm_bindgen]
pub struct Game {
    board: BoardV2,
    current_player: Player,
}

#[wasm_bindgen]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
#[wasm_bindgen]
pub enum WallState {
    Empty,
    Horizontal,
    Vertical,
}

#[wasm_bindgen]
pub struct Location {
    pub x: u8,
    pub y: u8,
}

#[wasm_bindgen]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Game {
        Game {
            board: BoardV2::empty(),
            current_player: Player::Player1,
        }
    }

    pub fn add_wall(&mut self, x: u8, y: u8, orientation: Orientation) -> bool {
        apply_move(
            self,
            Move::AddWall {
                location: (x, y),
                orientation: match orientation {
                    Orientation::Horizontal => quoridor_game::Orientation::Horizontal,
                    Orientation::Vertical => quoridor_game::Orientation::Vertical,
                },
            },
        )
    }

    pub fn move_token(&mut self, direction: Direction) -> bool {
        apply_move(
            self,
            Move::MoveToken(match direction {
                Direction::Up => quoridor_game::Direction::Up,
                Direction::Down => quoridor_game::Direction::Down,
                Direction::Left => quoridor_game::Direction::Left,
                Direction::Right => quoridor_game::Direction::Right,
            }),
        )
    }

    pub fn current_player(&self) -> u8 {
        match self.current_player {
            Player::Player1 => 1,
            Player::Player2 => 2,
        }
    }

    pub fn get_location(&self, player: u8) -> Location {
        let player = match player {
            1 => Player::Player1,
            2 => Player::Player2,
            _ => panic!(),
        };

        let loc = self.board.player_location(player);
        Location { x: loc.0, y: loc.1 }
    }

    pub fn get_wall_status(&self, x: u8, y: u8) -> WallState {
        match self.board.get_wall_state((x, y)) {
            None => WallState::Empty,
            Some(quoridor_game::Orientation::Horizontal) => WallState::Horizontal,
            Some(quoridor_game::Orientation::Vertical) => WallState::Vertical,
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
