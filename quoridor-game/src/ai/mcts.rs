use crate::{bitpacked::BoardV2, Board, Direction, Move, Orientation, Player};
use fxhash::FxHasher;
use mcts::transposition_table::*;
use mcts::tree_policy::*;
use mcts::*;
use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[derive(Clone, Debug)]
struct QuoridorState<B: Board + Clone> {
    board: B,
    current_player: Player,
}

pub struct MctsAiPlayer {
    state: QuoridorState<BoardV2>,
    mcts: MCTSManager<QuoridorSpec<BoardV2>>,
}

struct QuoridorEvaluator;

impl Evaluator<QuoridorSpec<BoardV2>> for QuoridorEvaluator {
    type StateEvaluation = i8;

    fn evaluate_new_state(
        &self,
        state: &QuoridorState<BoardV2>,
        moves: &Vec<Move>,
        _: Option<SearchHandle<QuoridorSpec<BoardV2>>>,
    ) -> (Vec<()>, i8) {
        let score = state.board.distance_to_goal(Player::Player2).unwrap() as i8
            - state.board.distance_to_goal(Player::Player1).unwrap() as i8;

        (vec![(); moves.len()], score)
    }
    fn interpret_evaluation_for_player(&self, evaln: &i8, player: &Player) -> i64 {
        let score = match player {
            Player::Player1 => *evaln,
            Player::Player2 => -1 * *evaln,
        };
        score as i64
    }
    fn evaluate_existing_state(
        &self,
        _: &QuoridorState<BoardV2>,
        evaln: &i8,
        _: SearchHandle<QuoridorSpec<BoardV2>>,
    ) -> i8 {
        *evaln
    }
}

impl TranspositionHash for QuoridorState<BoardV2> {
    fn hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
        hasher.write_u8(self.current_player as u8);
        self.board.fx_hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Default)]
struct QuoridorSpec<B>(PhantomData<B>);

impl MCTS for QuoridorSpec<BoardV2> {
    type State = QuoridorState<BoardV2>;
    type Eval = QuoridorEvaluator;
    type NodeData = ();
    type ExtraThreadData = ();
    type TreePolicy = UCTPolicy;
    type TranspositionTable = ApproxTable<Self>;

    fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
        CycleBehaviour::UseCurrentEvalWhenCycleDetected
    }
}

impl<B: Board + Clone + Hash + Eq> QuoridorState<B> {
    pub fn new(board: B) -> Self {
        Self {
            board,
            current_player: Player::Player1,
        }
    }
}

impl MctsAiPlayer {
    pub fn new(board: BoardV2) -> Self {
        Self {
            state: QuoridorState::new(board.clone()),
            mcts: MCTSManager::new(
                QuoridorState::new(board),
                QuoridorSpec(PhantomData::default()),
                QuoridorEvaluator,
                UCTPolicy::new(0.2),
                ApproxTable::new(1024 * 16),
            ),
        }
    }
}

impl MctsAiPlayer {
    pub fn send(&mut self, m: &Move) {
        self.state.board.apply_move(m, self.state.current_player);
        self.state.current_player = self.state.current_player.other();
    }

    pub fn receive(&mut self) -> Move {
        let m = self.mcts.best_move().unwrap();
        self.state.board.apply_move(&m, self.state.current_player);
        self.state.current_player = self.state.current_player.other();
        m
    }

    pub fn debug(&mut self) {
        self.mcts.playout_n_parallel(10000, 16); // 10000 playouts, 4 search threads
        dbg!(self.mcts.principal_variation(100));
    }
}

impl<B: Board + Clone + Hash + Eq + Clone> GameState for QuoridorState<B> {
    type Move = Move;
    type Player = Player;
    type MoveList = Vec<Move>;

    fn current_player(&self) -> Self::Player {
        self.current_player
    }
    fn available_moves(&self) -> Vec<Move> {
        if self.board.player_location(Player::Player1).1 == 8 {
            return vec![];
        }
        if self.board.player_location(Player::Player2).1 == 0 {
            return vec![];
        }
        if self.board.available_walls(Player::Player1) == 0
            && self.board.available_walls(Player::Player2) == 0
        {
            return vec![];
        }
        all_moves()
            .filter(|mov| self.board.is_legal(self.current_player, mov))
            .collect()
    }
    fn make_move(&mut self, mov: &Self::Move) {
        self.board.apply_move(mov, self.current_player);
        self.current_player = self.current_player.other();
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
