use super::*;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Print, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ErrorKind,
};
use quoridor_game::v1::{BoardV1, Cell, WallState};
use std::{
    io::{stdout, Write},
    time::Duration,
};

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

fn display_cell(cell: &DisplayCell, (x, y): (u8, u8)) -> Result<(), DisplayError> {
    if x != 8 {
        queue!(
            stdout(),
            crossterm::cursor::MoveTo((3 * x + 2) as u16, (2 * y) as u16),
            SetForegroundColor(cell.right.to_color()),
            Print("|")
        )?;
    }
    if y != 8 {
        queue!(
            stdout(),
            crossterm::cursor::MoveTo((3 * x + 0) as u16, (2 * y + 1) as u16),
            SetForegroundColor(cell.bottom.to_color()),
            Print("--")
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
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            disable_raw_mode().unwrap();
            execute!(stdout(), LeaveAlternateScreen).unwrap();

            default_hook(info);
        }));

        Ok(Self)
    }

    pub fn get_move(
        &mut self,
        board: &BoardV1,
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
                        if board.is_legal(*player, candidate_move) {
                            return Ok(());
                        }
                    }
                    KeyCode::Char('m') => {
                        *candidate_move = match candidate_move {
                            Move::AddWall {
                                location: _,
                                orientation: _,
                            } => Move::MoveTo(0, 0),
                            Move::MoveTo(_, _) => Move::AddWall {
                                location: (4, 4),
                                orientation: Orientation::Horizontal,
                            },
                        }
                    }
                    KeyCode::Char('/') | KeyCode::Char('r') => match candidate_move {
                        Move::AddWall {
                            orientation,
                            location: _,
                        } => *orientation = orientation.other(),

                        Move::MoveTo(_, _) => (),
                    },
                    KeyCode::Left => match candidate_move {
                        Move::AddWall {
                            orientation: _,
                            location: (x, _),
                        } => *x = if *x > 0 { *x - 1 } else { 0 },
                        Move::MoveTo(_, _) => {
                            if board.player_location(*player).0 > 0 {
                                *candidate_move = Move::MoveTo(
                                    board.player_location(*player).0 - 1,
                                    board.player_location(*player).1,
                                )
                            }
                        }
                    },
                    KeyCode::Right => match candidate_move {
                        Move::AddWall {
                            orientation: _,
                            location: (x, _),
                        } => *x = if *x < 7 { *x + 1 } else { 7 },
                        Move::MoveTo(_, _) => {
                            *candidate_move = Move::MoveTo(
                                board.player_location(*player).0 + 1,
                                board.player_location(*player).1,
                            )
                        }
                    },
                    KeyCode::Up => match candidate_move {
                        Move::AddWall {
                            orientation: _,
                            location: (_, y),
                        } => *y = if *y > 0 { *y - 1 } else { 0 },
                        Move::MoveTo(_, _) => {
                            if board.player_location(*player).1 > 0 {
                                *candidate_move = Move::MoveTo(
                                    board.player_location(*player).0,
                                    board.player_location(*player).1 - 1,
                                )
                            }
                        }
                    },
                    KeyCode::Down => match candidate_move {
                        Move::AddWall {
                            orientation: _,
                            location: (_, y),
                        } => *y = if *y < 7 { *y + 1 } else { 7 },
                        Move::MoveTo(_, _) => {
                            *candidate_move = Move::MoveTo(
                                board.player_location(*player).0,
                                board.player_location(*player).1 + 1,
                            )
                        }
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

    pub fn show(&mut self, board: &BoardV1) -> Result<(), DisplayError> {
        queue!(stdout(), Clear(ClearType::All))?;
        display(board, None)?;
        stdout().flush()?;
        Ok(())
    }

    pub fn check_exit(&mut self) -> bool {
        if poll(Duration::from_secs(0)).unwrap_or(false) {
            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            })) = read()
            {
                return modifiers.contains(KeyModifiers::CONTROL);
            }
        }
        false
    }
}

fn display(board: &BoardV1, player_and_move: Option<(&Player, &Move)>) -> Result<(), DisplayError> {
    for (y, cells) in board.cells.iter().enumerate() {
        let y = y as u8;
        for (x, cell) in cells.iter().enumerate() {
            let x = x as u8;
            let mut cell = DisplayCell {
                right: DisplayWallState::from_wall_state(cell.right),
                bottom: DisplayWallState::from_wall_state(cell.bottom),
                joint: DisplayWallState::from_wall_state(cell.joint),
            };

            match player_and_move {
                Some((
                    _,
                    Move::AddWall {
                        orientation: Orientation::Vertical,
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
                        orientation: Orientation::Horizontal,
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
                    orientation: _,
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
            crossterm::cursor::MoveTo((3 * loc.0) as u16, (2 * loc.1) as u16),
            match player {
                Player::Player1 => Print("v"),
                Player::Player2 => Print("^"),
            }
        )?;
    }

    if let Some((player, candidate_move)) = player_and_move {
        if board.is_legal(*player, candidate_move) {
            if let Move::MoveTo(nx, ny) = candidate_move {
                queue!(
                    stdout(),
                    SetForegroundColor(DisplayWallState::Candidate.to_color()),
                    crossterm::cursor::MoveTo((3 * nx) as u16, (2 * ny) as u16),
                    Print("#")
                )?;
            }
        }
    }

    // let packed_board: crate::bitpacked::BoardV2 = board.clone().into();

    queue!(
        stdout(),
        SetForegroundColor(crossterm::style::Color::White),
        crossterm::cursor::MoveTo(0, 18),
        Print(format!(
            "1 (v) has {} walls and {} steps to go",
            board.available_walls(crate::Player::Player1),
            board
                .distance_to_goal(crate::Player::Player1)
                .unwrap_or(u8::MAX)
        )),
        crossterm::cursor::MoveTo(0, 19),
        Print(format!(
            "2 (^) has {} walls and {} steps to go",
            board.available_walls(crate::Player::Player2),
            board
                .distance_to_goal(crate::Player::Player2)
                .unwrap_or(u8::MAX)
        )),
    )?;

    stdout().flush()?;

    Ok(())
}
