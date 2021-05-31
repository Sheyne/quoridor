use clap::{AppSettings, Clap};
use display::DisplayError;
use parse_display::{Display, FromStr};
use quoridor_ai::{greedy, mcts::MctsAiPlayer};
use quoridor_ai::{greedy::GreedyAiPlayer, mcts::MctsError};
use quoridor_game::bitpacked::BoardV2;
use quoridor_game::*;
use std::{
    convert::{TryFrom, TryInto},
    hash::Hash,
};
use tcp::GameError;

#[derive(Debug)]
pub enum Error {
    InvalidMoveAttempted,
    CantFindMoveError,
    MctsError(MctsError),
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

impl<B: Board + Clone + Eq + Hash> RemotePlayer for quoridor_ai::rubot::QuoridorGame<B> {
    fn send(&mut self, m: &Move) -> Result<(), Error> {
        self.apply_move(m).map_err(|_| Error::InvalidMoveAttempted)
    }

    fn receive(&mut self) -> Result<Move, Error> {
        let mov = if let Some(mov) =
            rubot::Bot::new(self.current_player()).select(self, std::time::Duration::from_secs(1))
        {
            mov
        } else {
            greedy::best_move(self.board().clone(), self.current_player())
                .map_err(|_| Error::CantFindMoveError)?
        };

        self.apply_move(&mov)
            .map_err(|_| Error::InvalidMoveAttempted)?;

        Ok(mov)
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

impl<B: Board + Clone + Hash + Eq> RemotePlayer for quoridor_ai::greedy::GreedyAiPlayer<B> {
    fn send(&mut self, m: &Move) -> Result<(), Error> {
        quoridor_ai::greedy::GreedyAiPlayer::send(self, m).map_err(|_| Error::InvalidMoveAttempted)
    }
    fn receive(&mut self) -> Result<Move, Error> {
        quoridor_ai::greedy::GreedyAiPlayer::receive(self).map_err(|_| Error::InvalidMoveAttempted)
    }
}

impl RemotePlayer for quoridor_ai::mcts::MctsAiPlayer {
    fn send(&mut self, m: &Move) -> Result<(), Error> {
        quoridor_ai::mcts::MctsAiPlayer::send(self, m).map_err(Error::MctsError)
    }
    fn receive(&mut self) -> Result<Move, Error> {
        quoridor_ai::mcts::MctsAiPlayer::receive(self).map_err(Error::MctsError)
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

#[derive(FromStr, Display, Clone)]
#[display(style = "kebab-case")]
enum PlayerKind {
    Keyboard,
    GreedyAi,
    Rubot,
    #[display("mcts-ai-{0}")]
    MctsAi(u32),
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
                PlayerDriver::RemotePlayer(Box::new(GreedyAiPlayer::new(board, Player::Player1)))
            }
            PlayerKind::Rubot => PlayerDriver::RemotePlayer(Box::new(
                quoridor_ai::rubot::QuoridorGame::<BoardV2>::new(),
            )),
            PlayerKind::MctsAi(t) => {
                PlayerDriver::RemotePlayer(Box::new(MctsAiPlayer::new(board, t)))
            }
            PlayerKind::Keyboard => PlayerDriver::Keyboard,
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
        player1: opts.player1.clone().try_into()?,
        player2: opts.player2.clone().try_into()?,
        display: display::Display::new()?,
        board: BoardV2::empty(),
        candidate: Move::MoveTo(0, 0),
    };

    let mut current_player = Player::Player1;

    loop {
        main.display.show(&main.board.clone().into())?;

        if main.display.check_exit() {
            drop(main);
            println!("User requested exit.");
            return Ok(());
        }
        let candidate = main.get_move(current_player)?;

        if !main.board.is_legal(current_player, &candidate) {
            panic!(
                "{:?} tried to play an illegal move {:?}",
                current_player, candidate
            );
        }

        main.board
            .apply_move(&candidate, current_player)
            .map_err(|_| Error::InvalidMoveAttempted)?;

        main.send_move(current_player.other(), &candidate)?;

        if main.board.player_location(Player::Player1).1 == 8
            || main.board.player_location(Player::Player2).1 == 0
        {
            let winner = if main.board.player_location(Player::Player1).1 == 8 {
                Player::Player1
            } else {
                Player::Player2
            };
            drop(main);
            println!(
                "{:?} ({}) wins!",
                winner,
                match winner {
                    Player::Player1 => opts.player1,
                    Player::Player2 => opts.player2,
                }
            );
            return Ok(());
        }

        current_player = current_player.other();
    }
}
