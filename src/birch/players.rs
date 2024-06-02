use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use chess::{Color, Game};
use futures::SinkExt;
use vampirc_uci::{UciFen, UciMessage};

use crate::birch::player::Player;

pub struct Players {
    white: Box<Player>,
    black: Box<Player>,
}

impl Players {
    pub fn new(msg_sink: Sender<(Color, UciMessage)>, white_cmd: &str, black_cmd: &str) -> Players {
        let sink = Arc::new(Mutex::new(msg_sink));
        let mut white = Player::new(Color::White, white_cmd, sink.clone());
        white.send(UciMessage::Uci);
        let mut black = Player::new(Color::Black, black_cmd, sink.clone());
        black.send(UciMessage::Uci);
        Players {
            white: Box::new(white),
            black: Box::new(black),
        }
    }

    pub fn send(&mut self, color: Color, msg: UciMessage) {
        (match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        })
        .send(msg);
    }

    pub fn next_turn(&mut self, game: &Box<Game>) {
        let c = game.side_to_move();
        self.send(
            c,
            UciMessage::Position {
                startpos: false,
                fen: Some(UciFen(game.current_position().to_string())),
                moves: vec![],
            },
        );
        self.send(c, UciMessage::go())
    }

    pub fn close(mut self) {
        self.white.close();
        self.black.close();
    }
}
