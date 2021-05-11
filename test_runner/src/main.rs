use quoridor_game::{ai::mcts::MctsAiPlayer, bitpacked::BoardV2, Board};

fn main() {
    let board = BoardV2::empty();
    let mut p = MctsAiPlayer::new(board);
    p.debug();
}
