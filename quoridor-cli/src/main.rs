use std::hash::Hash;

use clap::{AppSettings, Clap};
use display::DisplayError;
use quoridor_game::ai::AiPlayer;
use quoridor_game::bitpacked::BoardV2;
use quoridor_game::*;
use tcp::GameError;

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
impl From<GameError> for Error {
    fn from(e: GameError) -> Error {
        Error::TcpError(e)
    }
}

pub trait RemotePlayer {
    fn send(&mut self, m: &Move) -> Result<(), Error>;
    fn receive(&mut self) -> Result<Move, Error>;
}

impl RemotePlayer for tcp::Game {
    fn send(&mut self, m: &Move) -> Result<(), Error> {
        tcp::Game::send(self, m).map_err(Error::TcpError)
    }
    fn receive(&mut self) -> Result<Move, Error> {
        tcp::Game::receive(self).map_err(Error::TcpError)
    }
}

impl<B: Board + Clone + Hash + Eq> RemotePlayer for quoridor_game::ai::AiPlayer<B> {
    fn send(&mut self, m: &Move) -> Result<(), Error> {
        quoridor_game::ai::AiPlayer::send(self, m);
        Ok(())
    }
    fn receive(&mut self) -> Result<Move, Error> {
        Ok(quoridor_game::ai::AiPlayer::receive(self))
    }
}

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
    Ai,
    Serve {
        #[clap(long, default_value = "3333")]
        port: u16,
    },
    Connect {
        connect: String,
    },
}

fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();

    let mut board = BoardV2::empty();
    let mut tcp: Box<dyn RemotePlayer> = match opts.action {
        LaunchAction::Serve { port } => Box::new(tcp::Game::serve(format!("0.0.0.0:{}", port))?),
        LaunchAction::Connect { connect } => Box::new(tcp::Game::connect(connect)?),
        LaunchAction::Ai => Box::new(AiPlayer::new(board.clone())),
    };

    let mut display = display::Display::new()?;
    let iam = if opts.player == 1 {
        Player::Player1
    } else {
        Player::Player2
    };

    let mut current_player = Player::Player1;

    let mut candidate = Move::MoveToken(Direction::Down);
    loop {
        let v2 = board.clone().into();
        display.show(&v2)?;
        if current_player == iam {
            display.get_move(&v2, &iam, &mut candidate)?;
            if !board.is_legal(iam, &candidate) {
                todo!();
            }

            board.apply_move(&candidate, iam);

            tcp.send(&candidate)?;
        } else {
            let candidate = tcp.receive()?;
            if !board.is_legal(current_player, &candidate) {
                todo!();
            }

            board.apply_move(&candidate, current_player);
        }
        current_player = current_player.other();
    }
}
