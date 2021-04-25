use clap::{AppSettings, Clap};
use display::DisplayError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WallState {
    Wall,
    Open,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell<WS> {
    right: WS,
    bottom: WS,
    joint: WS,
}

pub type RegularCell = Cell<WallState>;

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    cells: [[RegularCell; 9]; 9],
    player1_loc: (usize, usize),
    player2_loc: (usize, usize),
    player1_walls: usize,
    player2_walls: usize,
}

impl Board {
    fn empty() -> Self {
        let open_cell = RegularCell {
            right: WallState::Open,
            bottom: WallState::Open,
            joint: WallState::Open,
        };

        let open_cells = [
            open_cell, open_cell, open_cell, open_cell, open_cell, open_cell, open_cell, open_cell,
            open_cell,
        ];

        Board {
            cells: [
                open_cells, open_cells, open_cells, open_cells, open_cells, open_cells, open_cells,
                open_cells, open_cells,
            ],
            player1_loc: (4, 0),
            player2_loc: (4, 8),
            player1_walls: 10,
            player2_walls: 10,
        }
    }

    fn add_wall(&mut self, location: &(usize, usize), horizontal: bool) {
        if horizontal {
            self.cell_mut(location).bottom = WallState::Wall;
            self.cell_mut(location).joint = WallState::Wall;
            self.cell_mut(&(location.0 + 1, location.1)).bottom = WallState::Wall;
        } else {
            self.cell_mut(location).right = WallState::Wall;
            self.cell_mut(location).joint = WallState::Wall;
            self.cell_mut(&(location.0, location.1 + 1)).right = WallState::Wall;
        }
    }

    pub fn cell(&self, (x, y): &(usize, usize)) -> &RegularCell {
        &self.cells[*y][*x]
    }

    pub fn cell_mut(&mut self, (x, y): &(usize, usize)) -> &mut RegularCell {
        &mut self.cells[*y][*x]
    }

    pub fn location(&self, player: &Player) -> &(usize, usize) {
        match player {
            Player::Player1 => &self.player1_loc,
            Player::Player2 => &self.player2_loc,
        }
    }

    pub fn location_mut(&mut self, player: &Player) -> &mut (usize, usize) {
        match player {
            Player::Player1 => &mut self.player1_loc,
            Player::Player2 => &mut self.player2_loc,
        }
    }

    fn is_legal(&self, player: &Player, candidate_move: &Move) -> bool {
        match candidate_move {
            Move::AddWall {
                location,
                horizontal,
            } => {
                let (x, y) = *location;
                self.cell(&location).joint == WallState::Open
                    && if *horizontal {
                        self.cell(&location).bottom == WallState::Open
                            && self.cell(&(x + 1, y)).bottom == WallState::Open
                    } else {
                        self.cell(&location).right == WallState::Open
                            && self.cell(&(x, y + 1)).right == WallState::Open
                    }
            }
            Move::MoveToken(direction) => {
                let loc = self.location(player);
                direction.shift(loc).is_some()
                    && match direction {
                        Direction::Down => self.cell(&(loc.0, loc.1)).bottom == WallState::Open,
                        Direction::Up => self.cell(&(loc.0, loc.1 - 1)).bottom == WallState::Open,
                        Direction::Left => self.cell(&(loc.0 - 1, loc.1)).right == WallState::Open,
                        Direction::Right => self.cell(&(loc.0, loc.1)).right == WallState::Open,
                    }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Player {
    Player1,
    Player2,
}
impl Player {
    pub fn other(&self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
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

#[derive(Serialize, Deserialize, Debug)]
pub enum Move {
    AddWall {
        horizontal: bool,
        location: (usize, usize),
    },
    MoveToken(Direction),
}

#[derive(Debug)]
pub enum Error {
    DisplayError(DisplayError),
    TcpError(tcp::GameError),
}

impl From<DisplayError> for Error {
    fn from(e: DisplayError) -> Error {
        Error::DisplayError(e)
    }
}
impl From<tcp::GameError> for Error {
    fn from(e: tcp::GameError) -> Error {
        Error::TcpError(e)
    }
}

mod bitpacked;
mod display;
mod tcp;
#[derive(Clap)]
#[clap(version = "1.0", author = "Sheyne Anderson")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    action: LaunchAction,

    #[clap(long)]
    player: usize,
}

#[derive(Clap)]
enum LaunchAction {
    Serve {
        #[clap(long, default_value = "3333")]
        port: u16,
    },
    Connect {
        connect: String,
    },
}

fn apply_move(board: &mut Board, candidate: &Move, player: &Player) {
    match candidate {
        Move::AddWall {
            location,
            horizontal,
        } => {
            board.add_wall(&location, *horizontal);
        }
        Move::MoveToken(d) => {
            if let Some(new_loc) = d.shift(&board.location(&player)) {
                *board.location_mut(&player) = new_loc;
            }
        }
    }
}

fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();

    let mut tcp = match opts.action {
        LaunchAction::Serve { port } => tcp::Game::serve(format!("0.0.0.0:{}", port)),
        LaunchAction::Connect { connect } => tcp::Game::connect(connect),
    }?;

    let mut board = Board::empty();
    let mut display = display::Display::new()?;
    let iam = if opts.player == 1 {
        Player::Player1
    } else {
        Player::Player2
    };

    let mut current_player = Player::Player1;

    let mut candidate = Move::MoveToken(Direction::Down);
    loop {
        display.show(&board)?;
        if current_player == iam {
            display.get_move(&board, &iam, &mut candidate)?;
            if !board.is_legal(&iam, &candidate) {
                todo!();
            }

            apply_move(&mut board, &candidate, &iam);

            tcp.send(&candidate)?;
        } else {
            let candidate = tcp.receive()?;
            if !board.is_legal(&current_player, &candidate) {
                todo!();
            }

            apply_move(&mut board, &candidate, &current_player);
        }
        current_player = current_player.other();
    }
}
