mod utils;

use quoridor_ai::rubot::QuoridorGame;
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

#[wasm_bindgen]
pub enum AiKind {
    Greedy,
    Rubot,
}

#[wasm_bindgen]
pub struct Ai(AiKind, QuoridorGame<BoardV2>, u32);

#[wasm_bindgen]
impl Ai {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Ai {
        Ai(AiKind::Greedy, QuoridorGame::new(), 2000)
    }

    pub fn set_greedy(&mut self) {
        self.0 = AiKind::Greedy;
    }

    pub fn set_rubot(&mut self, steps: u32) {
        self.0 = AiKind::Rubot;
        self.2 = steps;
    }

    pub fn send(&mut self, mov: JsValue) {
        let mov = mov.into_serde().unwrap();
        self.1.apply_move(&mov).unwrap();
    }

    pub fn receive(&mut self) -> JsValue {
        JsValue::from_serde(&{
            let mov = match &mut self.0 {
                AiKind::Greedy => {
                    quoridor_ai::greedy::best_move(self.1.board().clone(), self.1.current_player())
                        .unwrap()
                }
                AiKind::Rubot => {
                    if let Some(mov) = rubot::Bot::new(self.1.current_player())
                        .select(&self.1, rubot::Steps(self.2))
                    {
                        mov
                    } else {
                        quoridor_ai::greedy::best_move(
                            self.1.board().clone(),
                            self.1.current_player(),
                        )
                        .unwrap()
                    }
                }
            };
            self.1.apply_move(&mov).unwrap();
            mov
        })
        .unwrap()
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

    pub fn distance_to_goal(&self, player: u8) -> u8 {
        let player = match player {
            1 => Player::Player1,
            2 => Player::Player2,
            _ => panic!(),
        };
        self.board.distance_to_goal(player).unwrap()
    }

    pub fn get_wall_status(&self, x: u8, y: u8) -> WallState {
        match self.board.get_wall_state((x, y)) {
            None => WallState::Empty,
            Some(quoridor_game::Orientation::Horizontal) => WallState::Horizontal,
            Some(quoridor_game::Orientation::Vertical) => WallState::Vertical,
        }
    }
}
