use quoridor_game::{Board, Move, Player};

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
    pub fn board(&self) -> &B {
        &self.board
    }
}

impl<B: Board + Clone> rubot::Game for QuoridorGame<B> {
    type Player = Player;
    type Action = Move;
    type Fitness = i8;
    type Actions = Vec<Move>;

    fn actions(&self, player: Self::Player) -> (bool, Self::Actions) {
        (
            player == self.current_player,
            if self.board.player_location(Player::Player1).1 == 8 {
                vec![]
            } else if self.board.player_location(Player::Player2).1 == 0 {
                vec![]
            } else if self.board.available_walls(Player::Player1) == 0
                && self.board.available_walls(Player::Player2) == 0
            {
                vec![]
            } else {
                self.board.legal_moves(self.current_player)
            },
        )
    }

    fn execute(&mut self, action: &Self::Action, player: Self::Player) -> Self::Fitness {
        self.apply_move(action).unwrap();
        self.board
            .distance_to_goal(player.other())
            .zip(self.board.distance_to_goal(player))
            .map(|(them, me)| {
                if them < 2 && player.other() == self.current_player {
                    -100
                } else if me < 2 && player == self.current_player {
                    100
                } else {
                    them as i8 - me as i8
                }
            })
            .unwrap_or(100)
    }
}
