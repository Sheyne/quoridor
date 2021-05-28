mod utils;

use quoridor_ai::{greedy::GreedyAiPlayer, rubot::QuoridorGame};
use quoridor_game::{bitpacked::BoardV2, Board, Move::*, Player};
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

pub enum AiHolder {
    Greedy(GreedyAiPlayer<BoardV2>),
    Rubot(QuoridorGame<BoardV2>),
}

#[wasm_bindgen]
pub struct Ai(AiHolder);

#[wasm_bindgen]
impl Ai {
    #[wasm_bindgen(constructor)]
    pub fn new(kind: &str) -> Ai {
        Ai(match kind {
            "greedy" => AiHolder::Greedy(GreedyAiPlayer::new(BoardV2::empty(), Player::Player1)),
            "rubot" => AiHolder::Rubot(QuoridorGame::new()),
            _ => panic!(),
        })
    }

    pub fn send(&mut self, mov: &Move) {
        match &mut self.0 {
            AiHolder::Greedy(g) => g.send(&mov.0).unwrap(),
            AiHolder::Rubot(r) => r.apply_move(&mov.0).unwrap(),
        }
    }

    pub fn receive(&mut self) -> Move {
        Move(match &mut self.0 {
            AiHolder::Greedy(g) => g.receive().unwrap(),
            AiHolder::Rubot(r) => {
                if let Some(mov) =
                    rubot::Bot::new(r.current_player()).select(r, std::time::Duration::from_secs(1))
                {
                    mov
                } else {
                    quoridor_ai::greedy::best_move(r.board().clone(), r.current_player()).unwrap()
                }
            }
        })
    }
}

#[wasm_bindgen]
pub struct Move(quoridor_game::Move);

#[wasm_bindgen]
impl Move {
    pub fn add_wall(x: u8, y: u8, orientation: Orientation) -> Move {
        Move(AddWall {
            location: (x, y),
            orientation: match orientation {
                Orientation::Horizontal => quoridor_game::Orientation::Horizontal,
                Orientation::Vertical => quoridor_game::Orientation::Vertical,
            },
        })
    }

    pub fn move_token(direction: Direction) -> Move {
        Move(MoveToken(match direction {
            Direction::Up => quoridor_game::Direction::Up,
            Direction::Down => quoridor_game::Direction::Down,
            Direction::Left => quoridor_game::Direction::Left,
            Direction::Right => quoridor_game::Direction::Right,
        }))
    }
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Game {
        Game {
            board: BoardV2::empty(),
            current_player: Player::Player1,
        }
    }

    pub fn copy(&self) -> Game {
        Game {
            board: self.board.clone(),
            current_player: self.current_player,
        }
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

    pub fn available_walls(&self, player: u8) -> u8 {
        let player = match player {
            1 => Player::Player1,
            2 => Player::Player2,
            _ => panic!(),
        };

        self.board.available_walls(player)
    }

    pub fn apply_move(&mut self, mov: &Move) -> bool {
        if self.board.is_legal(self.current_player, &mov.0) {
            if self.board.apply_move(&mov.0, self.current_player).is_ok() {
                self.current_player = self.current_player.other();
                return true;
            }
        }
        false
    }

    pub fn get_wall_status(&self, x: u8, y: u8) -> WallState {
        match self.board.get_wall_state((x, y)) {
            None => WallState::Empty,
            Some(quoridor_game::Orientation::Horizontal) => WallState::Horizontal,
            Some(quoridor_game::Orientation::Vertical) => WallState::Vertical,
        }
    }
}
