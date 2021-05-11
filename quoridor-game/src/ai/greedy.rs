use crate::{Board, Direction, Move, Orientation, Player};
// use fxhash::FxHashMap;
use std::hash::Hash;

pub struct GreedyAiPlayer<B: Board + Clone> {
    board: B,
    current_player: Player,
}

impl<B: Board + Clone + Hash + Eq> GreedyAiPlayer<B> {
    pub fn new(board: B) -> Self {
        Self {
            board,
            current_player: Player::Player1,
        }
    }
}

impl<B: Board + Clone + Hash + Eq> GreedyAiPlayer<B> {
    pub fn send(&mut self, m: &Move) {
        self.board.apply_move(m, self.current_player);
        self.current_player = self.current_player.other();
    }

    pub fn receive(&mut self) -> Move {
        let m = best_move(self.board.clone(), self.current_player);
        self.board.apply_move(&m, self.current_player);
        self.current_player = self.current_player.other();
        m
    }
}

fn all_moves() -> impl Iterator<Item = Move> {
    let adds_walls = [Orientation::Horizontal, Orientation::Vertical]
        .iter()
        .map(|x| *x)
        .flat_map(|o| {
            (0..8).flat_map(move |y| {
                (0..8).map(move |x| Move::AddWall {
                    orientation: o,
                    location: (x, y),
                })
            })
        });

    let shifts = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ]
    .iter()
    .map(|x| Move::MoveToken(*x));

    shifts.chain(adds_walls)
}

fn best_move<B: Board + Clone + Hash + Eq>(board: B, player: Player) -> Move {
    // let mut hashmap = FxHashMap::<(Player, B), Option<i8>>::default();

    // fn find_all_moves<B: Board + Clone + Hash + Eq>(
    //     map: &mut FxHashMap<(Player, B), Option<i8>>,
    //     depth: u8,
    //     board_state: &(Player, B),
    // ) {
    //     if depth > 3 {
    //         return;
    //     }

    //     if !map.contains_key(board_state) {
    //         let (player, board) = board_state.clone();
    //         map.insert((player, board.clone()), None);

    //         let legal_moves = all_moves().filter(|mov| board.is_legal(player, mov));
    //         for mov in legal_moves {
    //             let mut board = board.clone();
    //             board.apply_move(&mov, player);
    //             find_all_moves(map, depth + 1, &(player.other(), board));
    //         }
    //     }
    // }

    // find_all_moves(&mut hashmap, 0, &(player, board.clone()));

    let legal_moves = all_moves().filter(|mov| board.is_legal(player, mov));

    let boards = legal_moves.map(|mov| {
        let mut nb = board.clone();
        nb.apply_move(&mov, player);
        (mov, nb)
    });

    let scores = boards.map(|(mov, board)| {
        (
            mov,
            board.distance_to_goal(player.other()).unwrap() as i8
                - board.distance_to_goal(player).unwrap() as i8,
        )
    });

    scores.max_by_key(|(_, score)| *score).unwrap().0
}
