use bitpacked::BoardV2;
use clap::{AppSettings, Clap};
use display::DisplayError;
use game::*;

mod game;

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

fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();

    let mut tcp = match opts.action {
        LaunchAction::Serve { port } => tcp::Game::serve(format!("0.0.0.0:{}", port)),
        LaunchAction::Connect { connect } => tcp::Game::connect(connect),
    }?;

    let mut board = BoardV2::empty();
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
