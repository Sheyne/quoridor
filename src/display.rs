use super::*;
use std::io::{stdin, stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DisplayWallState {
    Wall,
    Open,
    Candidate,
    Collision,
}

impl DisplayWallState {
    fn to_color(self) -> &'static str {
        match self {
            DisplayWallState::Wall => termion::color::Yellow.fg_str(),
            DisplayWallState::Candidate => termion::color::Green.fg_str(),
            DisplayWallState::Open => termion::color::LightBlue.fg_str(),
            DisplayWallState::Collision => termion::color::Red.fg_str(),
        }
    }

    fn from_wall_state(o: WallState) -> DisplayWallState {
        match o {
            WallState::Wall => DisplayWallState::Wall,
            WallState::Open => DisplayWallState::Open,
        }
    }
}

type DisplayCell = Cell<DisplayWallState>;

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
            cell.right.to_color()
        )?;
    }
    if y != 8 {
        write!(
            screen,
            "{}{}-",
            termion::cursor::Goto((2 * x + 1) as u16, (2 * y + 2) as u16),
            cell.bottom.to_color()
        )?;
        if x != 8 {
            write!(screen, "{}+", cell.joint.to_color())?;
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum DisplayError {
    Quit,
    IoError(std::io::Error),
}

pub struct Display {
    screen: termion::cursor::HideCursor<AlternateScreen<RawTerminal<Stdout>>>,
}

impl Display {
    pub fn new() -> Self {
        Self {
            screen: termion::cursor::HideCursor::from(AlternateScreen::from(
                stdout().into_raw_mode().unwrap(),
            )),
        }
    }

    pub fn get_move(
        &mut self,
        board: &Board,
        player: &Player,
        candidate_move: &mut Move,
    ) -> Result<(), DisplayError> {
        write!(self.screen, "{}", termion::clear::All,).map_err(DisplayError::IoError)?;
        display(&mut self.screen, &board, Some((player, candidate_move)))
            .map_err(DisplayError::IoError)?;
        self.screen.flush().unwrap();
        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => break,
                Key::Char(' ') => {
                    if board.is_legal(player, candidate_move) {
                        return Ok(());
                    }
                }
                Key::Char('m') => {
                    *candidate_move = match candidate_move {
                        Move::AddWall {
                            location: _,
                            horizontal: _,
                        } => Move::MoveToken(Direction::Up),
                        Move::MoveToken(_) => Move::AddWall {
                            location: (4, 4),
                            horizontal: true,
                        },
                    }
                }
                Key::Char('/') | Key::Char('r') => match candidate_move {
                    Move::AddWall {
                        horizontal,
                        location: _,
                    } => *horizontal = !*horizontal,

                    Move::MoveToken(_) => (),
                },
                Key::Left => match candidate_move {
                    Move::AddWall {
                        horizontal: _,
                        location: (x, _),
                    } => *x = if *x > 0 { *x - 1 } else { 0 },
                    Move::MoveToken(d) => *d = Direction::Left,
                },
                Key::Right => match candidate_move {
                    Move::AddWall {
                        horizontal: _,
                        location: (x, _),
                    } => *x = if *x < 7 { *x + 1 } else { 7 },
                    Move::MoveToken(d) => *d = Direction::Right,
                },
                Key::Up => match candidate_move {
                    Move::AddWall {
                        horizontal: _,
                        location: (_, y),
                    } => *y = if *y > 0 { *y - 1 } else { 0 },
                    Move::MoveToken(d) => *d = Direction::Up,
                },
                Key::Down => match candidate_move {
                    Move::AddWall {
                        horizontal: _,
                        location: (_, y),
                    } => *y = if *y < 7 { *y + 1 } else { 7 },
                    Move::MoveToken(d) => *d = Direction::Down,
                },
                _ => {}
            }
            write!(self.screen, "{}", termion::clear::All,).map_err(DisplayError::IoError)?;
            display(&mut self.screen, &board, Some((player, &candidate_move)))
                .map_err(DisplayError::IoError)?;
            self.screen.flush().unwrap();
        }
        Err(DisplayError::Quit)
    }

    pub fn show(&mut self, board: &Board) -> Result<(), DisplayError> {
        write!(self.screen, "{}", termion::clear::All,).map_err(DisplayError::IoError)?;
        display(&mut self.screen, board, None).map_err(DisplayError::IoError)?;
        self.screen.flush().unwrap();
        Ok(())
    }
}

fn display<W: Write>(
    screen: &mut W,
    board: &Board,
    player_and_move: Option<(&Player, &Move)>,
) -> Result<(), std::io::Error> {
    for (y, cells) in board.cells.iter().enumerate() {
        for (x, cell) in cells.iter().enumerate() {
            let mut cell = DisplayCell {
                right: DisplayWallState::from_wall_state(cell.right),
                bottom: DisplayWallState::from_wall_state(cell.bottom),
                joint: DisplayWallState::from_wall_state(cell.joint),
            };

            match player_and_move {
                Some((
                    _,
                    Move::AddWall {
                        horizontal: false,
                        location: (cx, cy),
                    },
                )) if x == *cx && (y == *cy || y == *cy + 1) => {
                    cell.right = if cell.right == DisplayWallState::Open {
                        DisplayWallState::Candidate
                    } else {
                        DisplayWallState::Collision
                    };
                }
                Some((
                    _,
                    Move::AddWall {
                        horizontal: true,
                        location: (cx, cy),
                    },
                )) if y == *cy && (x == *cx || x == *cx + 1) => {
                    cell.bottom = if cell.bottom == DisplayWallState::Open {
                        DisplayWallState::Candidate
                    } else {
                        DisplayWallState::Collision
                    };
                }
                _ => (),
            }
            if let Some((
                _,
                Move::AddWall {
                    horizontal: _,
                    location: (cx, cy),
                },
            )) = player_and_move
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

    for player in [Player::Player1, Player::Player2].iter() {
        let loc = board.location(player);
        write!(
            screen,
            "{}X",
            termion::cursor::Goto((2 * loc.0 + 1) as u16, (2 * loc.1 + 1) as u16),
        )?;
    }

    if let Some((player, candidate_move)) = player_and_move {
        if board.is_legal(player, candidate_move) {
            if let Move::MoveToken(d) = candidate_move {
                if let Some(candidate_pos) = d.shift(board.location(player)) {
                    write!(
                        screen,
                        "{}{}#",
                        DisplayWallState::Candidate.to_color(),
                        termion::cursor::Goto(
                            (2 * candidate_pos.0 + 1) as u16,
                            (2 * candidate_pos.1 + 1) as u16
                        ),
                    )?;
                }
            }
        }
    }

    write!(screen, "{}", termion::cursor::Goto(0, 18),)?;

    Ok(())
}
