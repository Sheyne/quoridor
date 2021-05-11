use quoridor_game::{Direction::*, Move, Move::*, Orientation::*};

pub struct Replay(pub usize, pub Vec<Move>);

impl Replay {
    pub fn next(&mut self) -> Move {
        let next = self.1[self.0].clone();
        self.0 += 1;
        next
    }

    pub fn one() -> Replay {
        Self(
            0,
            vec![
                MoveToken(Down),
                MoveToken(Up),
                MoveToken(Down),
                MoveToken(Up),
                MoveToken(Down),
                MoveToken(Up),
                MoveToken(Down),
                MoveToken(Up),
                MoveToken(Down),
                AddWall {
                    orientation: Horizontal,
                    location: (4, 5),
                },
                AddWall {
                    orientation: Horizontal,
                    location: (4, 3),
                },
                AddWall {
                    orientation: Horizontal,
                    location: (2, 5),
                },
                AddWall {
                    orientation: Horizontal,
                    location: (2, 3),
                },
                AddWall {
                    orientation: Vertical,
                    location: (5, 5),
                },
                AddWall {
                    orientation: Horizontal,
                    location: (6, 3),
                },
                AddWall {
                    orientation: Horizontal,
                    location: (0, 5),
                },
                AddWall {
                    orientation: Vertical,
                    location: (1, 4),
                },
                AddWall {
                    orientation: Horizontal,
                    location: (4, 4),
                },
                AddWall {
                    orientation: Vertical,
                    location: (7, 7),
                },
                AddWall {
                    orientation: Vertical,
                    location: (6, 7),
                },
            ],
        )
    }
}
