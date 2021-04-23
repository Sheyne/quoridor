use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WallState {
    Wall,
    Open,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DisplayWallState {
    Wall,
    Open,
    Candidate,
    Collision,
}

impl WallState {
    fn to_diplay(self) -> DisplayWallState {
        match self {
            WallState::Wall => DisplayWallState::Wall,
            WallState::Open => DisplayWallState::Open,
        }
    }
}

impl DisplayWallState {
    fn to_color(self) -> termion::color::Rgb {
        match self {
            DisplayWallState::Wall => termion::color::Rgb(0xff, 0xff, 0xff),
            DisplayWallState::Candidate => termion::color::Rgb(0x60, 0xff, 0x60),
            DisplayWallState::Open => termion::color::Rgb(0x60, 0x60, 0x60),
            DisplayWallState::Collision => termion::color::Rgb(0xff, 0x60, 0x60),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cell<WS> {
    right: WS,
    bottom: WS,
    joint: WS,
}

type RegularCell = Cell<WallState>;
type DisplayCell = Cell<DisplayWallState>;

#[derive(Clone, PartialEq, Eq)]
struct Board {
    cells: [[RegularCell; 9]; 9],
}

impl Board {
    fn add_wall(&mut self, x: usize, y: usize, horizontal: bool) {
        if horizontal {
            self.cells[y][x].bottom = WallState::Wall;
            self.cells[y][x].joint = WallState::Wall;
            self.cells[y][x + 1].bottom = WallState::Wall;
        } else {
            self.cells[y][x].right = WallState::Wall;
            self.cells[y][x].joint = WallState::Wall;
            self.cells[y + 1][x].right = WallState::Wall;
        }
    }

    fn is_legal(&self, loc: &(usize, usize), candidate_move: &CandidateMove) -> bool {
        match candidate_move {
            CandidateMove::Wall { x, y, horizontal } => {
                self.cells[*y][*x].joint == WallState::Open
                    && if *horizontal {
                        self.cells[*y][*x].bottom == WallState::Open
                            && self.cells[*y][*x + 1].bottom == WallState::Open
                    } else {
                        self.cells[*y][*x].right == WallState::Open
                            && self.cells[*y + 1][*x].right == WallState::Open
                    }
            }
            CandidateMove::Move(direction) => {
                direction.shift(loc).is_some()
                    && match direction {
                        Direction::Down => self.cells[loc.1][loc.0].bottom == WallState::Open,
                        Direction::Up => self.cells[loc.1 - 1][loc.0].bottom == WallState::Open,
                        Direction::Left => self.cells[loc.1][loc.0 - 1].right == WallState::Open,
                        Direction::Right => self.cells[loc.1][loc.0].right == WallState::Open,
                    }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn shift(&self, position: &(usize, usize)) -> Option<(usize, usize)> {
        fn add((ax, ay): &(usize, usize), (bx, by): &(isize, isize)) -> Option<(usize, usize)> {
            let res = (*ax as isize + bx, *ay as isize + by);

            if res.0 >= 0 && res.0 < 9 && res.1 >= 0 && res.1 < 9 {
                Some((res.0 as usize, res.1 as usize))
            } else {
                None
            }
        }

        match self {
            Direction::Up => add(position, &(0, -1)),
            Direction::Down => add(position, &(0, 1)),
            Direction::Left => add(position, &(-1, 0)),
            Direction::Right => add(position, &(1, 0)),
        }
    }
}

enum CandidateMove {
    Wall {
        horizontal: bool,
        x: usize,
        y: usize,
    },
    Move(Direction),
}

fn display_cell<W: Write>(
    screen: &mut W,
    cell: &DisplayCell,
    (x, y): (usize, usize),
) -> Result<(), std::io::Error> {
    if x != 8 {
        write!(
            screen,
            "{}{}|",
            termion::cursor::Goto((2 * x + 2) as u16, (2 * y + 1) as u16),
            cell.right.to_color().fg_string()
        )?;
    }
    if y != 8 {
        write!(
            screen,
            "{}{}-",
            termion::cursor::Goto((2 * x + 1) as u16, (2 * y + 2) as u16),
            cell.bottom.to_color().fg_string()
        )?;
        if x != 8 {
            write!(screen, "{}+", cell.joint.to_color().fg_string())?;
        }
    }

    Ok(())
}

fn display<W: Write>(
    screen: &mut W,
    board: &Board,
    player: &(usize, usize),
    candidate_move: &CandidateMove,
) -> Result<(), std::io::Error> {
    for (y, cells) in board.cells.iter().enumerate() {
        for (x, cell) in cells.iter().enumerate() {
            let mut cell = DisplayCell {
                right: cell.right.to_diplay(),
                bottom: cell.bottom.to_diplay(),
                joint: cell.joint.to_diplay(),
            };

            match candidate_move {
                CandidateMove::Wall {
                    horizontal: false,
                    x: cx,
                    y: cy,
                } if x == *cx && (y == *cy || y == *cy + 1) => {
                    cell.right = if cell.right == DisplayWallState::Open {
                        DisplayWallState::Candidate
                    } else {
                        DisplayWallState::Collision
                    };
                }
                CandidateMove::Wall {
                    horizontal: true,
                    x: cx,
                    y: cy,
                } if y == *cy && (x == *cx || x == *cx + 1) => {
                    cell.bottom = if cell.bottom == DisplayWallState::Open {
                        DisplayWallState::Candidate
                    } else {
                        DisplayWallState::Collision
                    };
                }
                _ => (),
            }
            if let CandidateMove::Wall {
                horizontal: _,
                x: cx,
                y: cy,
            } = candidate_move
            {
                if x == *cx && y == *cy {
                    cell.joint = if cell.joint == DisplayWallState::Open {
                        DisplayWallState::Candidate
                    } else {
                        DisplayWallState::Collision
                    };
                }
            }

            display_cell(screen, &cell, (x, y))?;
        }
    }
    write!(
        screen,
        "{}X",
        termion::cursor::Goto((2 * player.0 + 1) as u16, (2 * player.1 + 1) as u16),
    )?;

    if board.is_legal(player, candidate_move) {
        if let CandidateMove::Move(d) = candidate_move {
            if let Some(candidate_pos) = d.shift(player) {
                write!(
                    screen,
                    "{}{}#",
                    DisplayWallState::Candidate.to_color().fg_string(),
                    termion::cursor::Goto(
                        (2 * candidate_pos.0 + 1) as u16,
                        (2 * candidate_pos.1 + 1) as u16
                    ),
                )?;
            }
        }
    }

    write!(screen, "{}", termion::cursor::Goto(0, 18),)?;

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let open_cell = RegularCell {
        right: WallState::Open,
        bottom: WallState::Open,
        joint: WallState::Open,
    };

    let open_cells = [
        open_cell, open_cell, open_cell, open_cell, open_cell, open_cell, open_cell, open_cell,
        open_cell,
    ];

    let mut b = Board {
        cells: [
            open_cells, open_cells, open_cells, open_cells, open_cells, open_cells, open_cells,
            open_cells, open_cells,
        ],
    };

    let mut candidate_move = CandidateMove::Move(Direction::Down);

    let mut player_loc = (4, 0);

    let mut screen =
        termion::cursor::HideCursor::from(AlternateScreen::from(stdout().into_raw_mode().unwrap()));

    let stdin = stdin();
    write!(screen, "{}", termion::clear::All,)?;
    display(&mut screen, &b, &player_loc, &candidate_move)?;
    screen.flush().unwrap();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char(' ') => {
                if b.is_legal(&player_loc, &candidate_move) {
                    match candidate_move {
                        CandidateMove::Wall { x, y, horizontal } => {
                            b.add_wall(x, y, horizontal);
                        }
                        CandidateMove::Move(d) => {
                            if let Some(new_loc) = d.shift(&player_loc) {
                                player_loc = new_loc;
                            }
                        }
                    }
                }
            }
            Key::Char('m') => {
                candidate_move = match &mut candidate_move {
                    CandidateMove::Wall {
                        x: _,
                        y: _,
                        horizontal: _,
                    } => CandidateMove::Move(Direction::Up),
                    CandidateMove::Move(_) => CandidateMove::Wall {
                        x: 4,
                        y: 4,
                        horizontal: true,
                    },
                }
            }
            Key::Char('/') | Key::Char('r') => match &mut candidate_move {
                CandidateMove::Wall {
                    horizontal,
                    x: _,
                    y: _,
                } => *horizontal = !*horizontal,

                CandidateMove::Move(_) => (),
            },
            Key::Left => match &mut candidate_move {
                CandidateMove::Wall {
                    horizontal: _,
                    x,
                    y: _,
                } => *x = if *x > 0 { *x - 1 } else { 0 },
                CandidateMove::Move(d) => *d = Direction::Left,
            },
            Key::Right => match &mut candidate_move {
                CandidateMove::Wall {
                    horizontal: _,
                    x,
                    y: _,
                } => *x = if *x < 7 { *x + 1 } else { 7 },
                CandidateMove::Move(d) => *d = Direction::Right,
            },
            Key::Up => match &mut candidate_move {
                CandidateMove::Wall {
                    horizontal: _,
                    x: _,
                    y,
                } => *y = if *y > 0 { *y - 1 } else { 0 },
                CandidateMove::Move(d) => *d = Direction::Up,
            },
            Key::Down => match &mut candidate_move {
                CandidateMove::Wall {
                    horizontal: _,
                    x: _,
                    y,
                } => *y = if *y < 7 { *y + 1 } else { 7 },
                CandidateMove::Move(d) => *d = Direction::Down,
            },
            _ => {}
        }
        write!(screen, "{}", termion::clear::All,)?;
        display(&mut screen, &b, &player_loc, &candidate_move)?;
        screen.flush().unwrap();
    }

    Ok(())
}
