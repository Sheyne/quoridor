mod utils;

use quoridor_ai::{greedy::GreedyAiPlayer, rubot::QuoridorGame};
use quoridor_game::{bitpacked::BoardV2, Board, Player};
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

    pub fn send(&mut self, mov: JsValue) {
        let mov = mov.into_serde().unwrap();
        match &mut self.0 {
            AiHolder::Greedy(g) => g.send(&mov).unwrap(),
            AiHolder::Rubot(r) => r.apply_move(&mov).unwrap(),
        }
    }

    pub fn receive(&mut self) -> JsValue {
        JsValue::from_serde(&match &mut self.0 {
            AiHolder::Greedy(g) => g.receive().unwrap(),
            AiHolder::Rubot(r) => {
                let mov = if let Some(mov) =
                    rubot::Bot::new(r.current_player()).select(r, rubot::Depth(3))
                {
                    mov
                } else {
                    quoridor_ai::greedy::best_move(r.board().clone(), r.current_player()).unwrap()
                };
                r.apply_move(&mov);
                mov
            }
        }).unwrap()
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

    pub fn apply_move(&mut self, mov: JsValue) -> bool {
        let mov = mov.into_serde().unwrap();
        if self.board.is_legal(self.current_player, &mov) {
            if self.board.apply_move(&mov, self.current_player).is_ok() {
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
