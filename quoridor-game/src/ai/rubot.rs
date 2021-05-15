use crate::{Board, Move, Player};

#[derive(Clone)]
pub struct QuoridorGame<B: Board> {
    board: B,
    current_player: Player,
}

impl<B: Board> QuoridorGame<B> {
    pub fn new() -> QuoridorGame<B> {
        QuoridorGame {
            board: B::empty(),
            current_player: Player::Player1,
        }
    }
    pub fn apply_move(&mut self, mov: &Move) -> Result<(), ()> {
        self.board.apply_move(mov, self.current_player)?;
        self.current_player = self.current_player.other();
        Ok(())
    }
    pub fn current_player(&self) -> Player {
        self.current_player
    }
}

impl<B: Board + Clone> rubot::Game for QuoridorGame<B> {
    type Player = Player;
    type Action = Move;
    /// did you choose a 10?
    type Fitness = i8;
    type Actions = Vec<Move>;

    fn actions(&self, player: Self::Player) -> (bool, Self::Actions) {
        (
            player == self.current_player,
            self.board.legal_moves(self.current_player),
        )
    }

    fn execute(&mut self, action: &Self::Action, player: Self::Player) -> Self::Fitness {
        self.apply_move(action).unwrap();
        self.board
            .distance_to_goal(player.other())
            .zip(self.board.distance_to_goal(player))
            .map(|(them, me)| them as i8 - me as i8)
            .unwrap_or(100)
    }
}
