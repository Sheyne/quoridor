use std::io::{BufRead, BufReader, BufWriter};
use std::{
    io::Write,
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

pub struct Game {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

#[derive(Debug)]
pub enum GameError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
}

impl Game {
    fn from_tcp(stream: TcpStream) -> std::io::Result<Self> {
        Ok(Self {
            reader: BufReader::new(stream.try_clone()?),
            writer: BufWriter::new(stream),
        })
    }

    pub fn serve<A: ToSocketAddrs>(addr: A) -> Result<Self, GameError> {
        let listener = TcpListener::bind(addr).unwrap();
        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => return Ok(Game::from_tcp(stream).map_err(GameError::IoError)?),
                _ => todo!(),
            }
        }
        todo!()
    }

    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, GameError> {
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(Game::from_tcp(stream).map_err(GameError::IoError)?),
            _ => todo!(),
        }
    }

    pub fn send(&mut self, action: &super::Move) -> Result<(), GameError> {
        write!(
            self.writer,
            "{}\n",
            serde_json::to_string(&action).map_err(GameError::JsonError)?
        )
        .map_err(GameError::IoError)?;
        self.writer.flush().map_err(GameError::IoError)?;
        Ok(())
    }

    pub fn receive(&mut self) -> Result<super::Move, GameError> {
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(GameError::IoError)?;
        Ok(serde_json::from_str(&line).map_err(GameError::JsonError)?)
    }
}
