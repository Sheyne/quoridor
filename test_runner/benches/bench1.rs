use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quoridor_game::{
    bitpacked::{can_reach_goal_v2, BoardV2},
    Board, Move, Orientation, Player,
};

fn criterion_benchmark(c: &mut Criterion) {
    let mut board = BoardV2::empty();
    c.bench_function("is_passible", |b| {
        b.iter(|| board.is_passible(black_box((2, 2)), black_box(quoridor_game::Direction::Down)))
    });

    board
        .add_wall(Player::Player1, (4, 6), Orientation::Horizontal)
        .unwrap();

    c.bench_function("is_legal_1", |b| {
        b.iter(|| {
            board.is_legal(
                Player::Player2,
                black_box(&Move::AddWall {
                    location: (6, 7),
                    orientation: Orientation::Horizontal,
                }),
            )
        })
    });

    board
        .add_wall(Player::Player2, (6, 7), Orientation::Horizontal)
        .unwrap();

    c.bench_function("is_legal_2", |b| {
        b.iter(|| {
            assert!(can_reach_goal_v2(
                black_box(&board),
                black_box(Player::Player2)
            ));
        })
    });

    c.bench_function("all_moves", |b| {
        use mcts::GameState;
        use quoridor_game::ai::mcts::QuoridorState;

        let qstate = QuoridorState::Clean {
            board: board.clone(),
            current_player: Player::Player1,
        };

        b.iter(|| {
            qstate.available_moves();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
