use quoridor_game::{Board, Move, Player};
use std::hash::Hash;

pub struct GreedyAiPlayer<B: Board + Clone> {
    board: B,
    current_player: Player,
}

impl<B: Board + Clone + Hash + Eq> GreedyAiPlayer<B> {
    pub fn new(board: B, current_player: Player) -> Self {
        Self {
            board,
            current_player: current_player,
        }
    }
}

impl<B: Board + Clone + Hash + Eq> GreedyAiPlayer<B> {
    pub fn send(&mut self, m: &Move) -> Result<(), ()> {
        self.board.apply_move(m, self.current_player)?;
        self.current_player = self.current_player.other();
        Ok(())
    }

    pub fn receive(&mut self) -> Result<Move, ()> {
        let m = best_move(self.board.clone(), self.current_player)?;
        self.board.apply_move(&m, self.current_player)?;
        self.current_player = self.current_player.other();
        Ok(m)
    }
}

pub fn best_move<B: Board + Clone + Hash + Eq>(board: B, player: Player) -> Result<Move, ()> {
    let legal_moves = board.legal_moves(player);
    let boards = legal_moves.into_iter().filter_map(|mov| {
        let mut nb = board.clone();
        nb.apply_move(&mov, player).ok()?;
        Some((mov, nb))
    });

    let distances = boards.map(|(mov, board)| {
        (
            mov,
            board.distance_to_goal(player).unwrap() as i8,
            board.distance_to_goal(player.other()).unwrap() as i8,
        )
    });

    let scores = distances.map(|(mov, my_dist, their_dist)| (mov, their_dist - my_dist));

    scores
        .max_by_key(|(_, score)| *score)
        .map(|x| x.0)
        .ok_or(())
}
