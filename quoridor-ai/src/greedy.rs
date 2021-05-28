use quoridor_game::{Board, Direction, Move, Orientation, Player};
// use fxhash::FxHashMap;
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

pub fn best_move<B: Board + Clone + Hash + Eq>(board: B, player: Player) -> Result<Move, ()> {
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

    let boards = legal_moves.filter_map(|mov| {
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

    let scores = distances.map(|(mov, my_dist, their_dist)| {
        (
            mov,
            their_dist - my_dist
        )
    });

    scores
        .max_by_key(|(_, score)| *score)
        .map(|x| x.0)
        .ok_or(())
}
