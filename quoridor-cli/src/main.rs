use clap::{AppSettings, Clap};
use display::DisplayError;
use parse_display::FromStr;
use quoridor_game::ai::greedy::GreedyAiPlayer;
use quoridor_game::ai::mcts::MctsAiPlayer;
use quoridor_game::bitpacked::BoardV2;
use quoridor_game::*;
use std::{
    convert::{TryFrom, TryInto},
    hash::Hash,
};
use tcp::GameError;

mod replay;

#[derive(Debug)]
pub enum Error {
    InvalidMoveAttempted,
    CantFindMoveError,
    DisplayError(DisplayError),
    TcpError(tcp::GameError),
}

impl From<DisplayError> for Error {
    fn from(e: DisplayError) -> Error {
        Error::DisplayError(e)
    }
}
impl From<GameError> for Error {
    fn from(e: GameError) -> Error {
        Error::TcpError(e)
    }
}

pub trait RemotePlayer {
    fn send(&mut self, m: &Move) -> Result<(), Error>;
    fn receive(&mut self) -> Result<Move, Error>;
}

impl RemotePlayer for replay::Replay {
    fn send(&mut self, _m: &Move) -> Result<(), Error> {
        Ok(())
    }
    fn receive(&mut self) -> Result<Move, Error> {
        Ok(self.next())
    }
}

impl RemotePlayer for tcp::Game {
    fn send(&mut self, m: &Move) -> Result<(), Error> {
        tcp::Game::send(self, m).map_err(Error::TcpError)
    }
    fn receive(&mut self) -> Result<Move, Error> {
        tcp::Game::receive(self).map_err(Error::TcpError)
    }
}

impl<B: Board + Clone + Hash + Eq> RemotePlayer for quoridor_game::ai::greedy::GreedyAiPlayer<B> {
    fn send(&mut self, m: &Move) -> Result<(), Error> {
        quoridor_game::ai::greedy::GreedyAiPlayer::send(self, m)
            .map_err(|_| Error::InvalidMoveAttempted)
    }
    fn receive(&mut self) -> Result<Move, Error> {
        quoridor_game::ai::greedy::GreedyAiPlayer::receive(self)
            .map_err(|_| Error::InvalidMoveAttempted)
    }
}

impl RemotePlayer for quoridor_game::ai::mcts::MctsAiPlayer {
    fn send(&mut self, m: &Move) -> Result<(), Error> {
        quoridor_game::ai::mcts::MctsAiPlayer::send(self, m)
            .map_err(|_| Error::InvalidMoveAttempted)
    }
    fn receive(&mut self) -> Result<Move, Error> {
        quoridor_game::ai::mcts::MctsAiPlayer::receive(self).map_err(|_| Error::CantFindMoveError)
    }
}

mod display;
mod tcp;
#[derive(Clap)]
#[clap(version = "1.0", author = "Sheyne Anderson")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    player1: PlayerKind,
    player2: PlayerKind,
}

#[derive(Clap, FromStr)]
#[display(style = "kebab-case")]
enum PlayerKind {
    Keyboard,
    GreedyAi,
    MctsAi,
    Replay1,
    #[display("serve-{port}")]
    Serve {
        port: u16,
    },
    #[display("connect-{connect}")]
    Connect {
        connect: String,
    },
}

enum PlayerDriver {
    RemotePlayer(Box<dyn RemotePlayer>),
    Keyboard,
}

impl TryFrom<PlayerKind> for PlayerDriver {
    type Error = Error;

    fn try_from(kind: PlayerKind) -> Result<Self, Error> {
        let board = BoardV2::empty();

        Ok(match kind {
            PlayerKind::Serve { port } => {
                PlayerDriver::RemotePlayer(Box::new(tcp::Game::serve(format!("0.0.0.0:{}", port))?))
            }
            PlayerKind::Connect { connect } => {
                PlayerDriver::RemotePlayer(Box::new(tcp::Game::connect(connect)?))
            }
            PlayerKind::GreedyAi => {
                PlayerDriver::RemotePlayer(Box::new(GreedyAiPlayer::new(board)))
            }
            PlayerKind::MctsAi => PlayerDriver::RemotePlayer(Box::new(MctsAiPlayer::new(board))),
            PlayerKind::Keyboard => PlayerDriver::Keyboard,
            _ => todo!(),
        })
    }
}

struct Main {
    player1: PlayerDriver,
    player2: PlayerDriver,
    display: display::Display,
    board: BoardV2,
    candidate: Move,
}

impl Main {
    fn driver(&mut self, p: Player) -> &mut PlayerDriver {
        match p {
            Player::Player1 => &mut self.player1,
            Player::Player2 => &mut self.player2,
        }
    }

    fn get_move(&mut self, p: Player) -> Result<Move, Error> {
        Ok(match self.driver(p) {
            PlayerDriver::Keyboard => {
                self.display
                    .get_move(&self.board.clone().into(), &p, &mut self.candidate)?;
                self.candidate.clone()
            }
            PlayerDriver::RemotePlayer(p) => p.receive()?,
        })
    }

    fn send_move(&mut self, p: Player, mov: &Move) -> Result<(), Error> {
        match self.driver(p) {
            PlayerDriver::Keyboard => (),
            PlayerDriver::RemotePlayer(p) => p.send(mov)?,
        }
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();

    let mut main = Main {
        player1: opts.player1.try_into()?,
        player2: opts.player2.try_into()?,
        display: display::Display::new()?,
        board: BoardV2::empty(),
        candidate: Move::MoveToken(Direction::Down),
    };

    let mut current_player = Player::Player1;

    loop {
        main.display.show(&main.board.clone().into())?;

        let candidate = main.get_move(current_player)?;

        if !main.board.is_legal(current_player, &candidate) {
            todo!();
        }

        main.board
            .apply_move(&candidate, current_player)
            .map_err(|_| Error::InvalidMoveAttempted)?;

        main.send_move(current_player.other(), &candidate)?;

        current_player = current_player.other();
    }
}
