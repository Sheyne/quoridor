use super::*;
use crossterm::{
    event::{read, Event, KeyCode},
    execute, queue,
    style::{Print, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ErrorKind,
};
use std::io::{stdout, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
enum DisplayWallState {
    Wall,
    Open,
    Candidate,
    Collision,
}

impl DisplayWallState {
    fn to_color(self) -> crossterm::style::Color {
        match self {
            DisplayWallState::Wall => crossterm::style::Color::Yellow,
            DisplayWallState::Candidate => crossterm::style::Color::Green,
            DisplayWallState::Open => crossterm::style::Color::Blue,
            DisplayWallState::Collision => crossterm::style::Color::Red,
        }
    }

    fn from_wall_state(o: WallState) -> DisplayWallState {
        match o {
            WallState::Wall => DisplayWallState::Wall,
            WallState::Open => DisplayWallState::Open,
        }
    }
}

impl From<std::io::Error> for DisplayError {
    fn from(e: std::io::Error) -> DisplayError {
        DisplayError::IoError(e)
    }
}
impl From<ErrorKind> for DisplayError {
    fn from(e: ErrorKind) -> DisplayError {
        DisplayError::CrosstermError(e)
    }
}

type DisplayCell = Cell<DisplayWallState>;

fn display_cell(cell: &DisplayCell, (x, y): (usize, usize)) -> Result<(), DisplayError> {
    if x != 8 {
        queue!(
            stdout(),
            crossterm::cursor::MoveTo((2 * x + 1) as u16, (2 * y) as u16),
            SetForegroundColor(cell.right.to_color()),
            Print("|")
        )?;
    }
    if y != 8 {
        queue!(
            stdout(),
            crossterm::cursor::MoveTo((2 * x + 0) as u16, (2 * y + 1) as u16),
            SetForegroundColor(cell.bottom.to_color()),
            Print("-")
        )?;
        if x != 8 {
            queue!(
                stdout(),
                SetForegroundColor(cell.joint.to_color()),
                Print("+")
            )?;
        }
    }

    stdout().flush()?;

    Ok(())
}

#[derive(Debug)]
pub enum DisplayError {
    Quit,
    CrosstermError(ErrorKind),
    IoError(std::io::Error),
}

pub struct Display;

impl Drop for Display {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(stdout(), LeaveAlternateScreen).unwrap();
    }
}

impl Display {
    pub fn new() -> Result<Self, DisplayError> {
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(Self)
    }

    pub fn get_move(
        &mut self,
        board: &Board,
        player: &Player,
        candidate_move: &mut Move,
    ) -> Result<(), DisplayError> {
        queue!(stdout(), Clear(ClearType::All))?;
        display(&board, Some((player, candidate_move)))?;
        stdout().flush()?;
        loop {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        if board.is_legal(player, candidate_move) {
                            return Ok(());
                        }
                    }
                    KeyCode::Char('m') => {
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
                    KeyCode::Char('/') | KeyCode::Char('r') => match candidate_move {
                        Move::AddWall {
                            horizontal,
                            location: _,
                        } => *horizontal = !*horizontal,

                        Move::MoveToken(_) => (),
                    },
                    KeyCode::Left => match candidate_move {
                        Move::AddWall {
                            horizontal: _,
                            location: (x, _),
                        } => *x = if *x > 0 { *x - 1 } else { 0 },
                        Move::MoveToken(d) => *d = Direction::Left,
                    },
                    KeyCode::Right => match candidate_move {
                        Move::AddWall {
                            horizontal: _,
                            location: (x, _),
                        } => *x = if *x < 7 { *x + 1 } else { 7 },
                        Move::MoveToken(d) => *d = Direction::Right,
                    },
                    KeyCode::Up => match candidate_move {
                        Move::AddWall {
                            horizontal: _,
                            location: (_, y),
                        } => *y = if *y > 0 { *y - 1 } else { 0 },
                        Move::MoveToken(d) => *d = Direction::Up,
                    },
                    KeyCode::Down => match candidate_move {
                        Move::AddWall {
                            horizontal: _,
                            location: (_, y),
                        } => *y = if *y < 7 { *y + 1 } else { 7 },
                        Move::MoveToken(d) => *d = Direction::Down,
                    },
                    _ => {}
                }
            }
            queue!(stdout(), Clear(ClearType::All),)?;
            display(&board, Some((player, &candidate_move)))?;
            stdout().flush()?;
        }
        Err(DisplayError::Quit)
    }

    pub fn show(&mut self, board: &Board) -> Result<(), DisplayError> {
        queue!(stdout(), Clear(ClearType::All))?;
        display(board, None)?;
        stdout().flush()?;
        Ok(())
    }
}

fn display(board: &Board, player_and_move: Option<(&Player, &Move)>) -> Result<(), DisplayError> {
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

            display_cell(&cell, (x, y))?;
        }
    }

    for player in [Player::Player1, Player::Player2].iter() {
        let loc = board.location(player);
        queue!(
            stdout(),
            crossterm::cursor::MoveTo((2 * loc.0) as u16, (2 * loc.1) as u16),
            Print("X")
        )?;
    }

    if let Some((player, candidate_move)) = player_and_move {
        if board.is_legal(player, candidate_move) {
            if let Move::MoveToken(d) = candidate_move {
                if let Some(candidate_pos) = d.shift(board.location(player)) {
                    queue!(
                        stdout(),
                        SetForegroundColor(DisplayWallState::Candidate.to_color()),
                        crossterm::cursor::MoveTo(
                            (2 * candidate_pos.0) as u16,
                            (2 * candidate_pos.1) as u16
                        ),
                        Print("#")
                    )?;
                }
            }
        }
    }

    queue!(stdout(), crossterm::cursor::MoveTo(0, 17),)?;
    stdout().flush()?;

    Ok(())
}
