use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

use chess::{Board, ChessMove, Color, Error, Game, Piece};

pub struct BirchGame {
    game: Game,
    side_to_move: Color,
    halfmove_clock: u8,
    fullmove_number: u32,
}

impl BirchGame {
    pub fn new() -> BirchGame {
        Self::new_with_board(Board::default())
    }

    pub fn new_with_board(board: Board) -> BirchGame {
        let game = Game::new_with_board(board);
        BirchGame {
            side_to_move: game.side_to_move(),
            game,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn make_move(&mut self, m: ChessMove) -> bool {
        let mut board = self.game.current_position();
        let white_castle_rights = board.castle_rights(Color::White);
        let black_castle_rights = board.castle_rights(Color::Black);
        if board.piece_on(m.get_source()) == Some(Piece::Pawn) {
            self.halfmove_clock = 0;
        } else if board.piece_on(m.get_dest()).is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.game.make_move(m) {
            board = board.make_move_new(m);
            self.side_to_move = !self.side_to_move;
            if self.side_to_move == Color::White {
                self.fullmove_number += 1;
            }

            if board.castle_rights(Color::White) != white_castle_rights
                || board.castle_rights(Color::Black) != black_castle_rights
            {
                self.halfmove_clock = 0;
            }
            true
        } else {
            false
        }
    }

    pub fn can_declare_draw(&self) -> bool {
        if let Some(value) = self.can_declare_draw_partial() {
            return value;
        }
        self.game.can_declare_draw()
    }

    fn can_declare_draw_partial(&self) -> Option<bool> {
        if self.result().is_some() {
            return Some(false);
        }
        if self.halfmove_clock >= 100 {
            return Some(true);
        }
        None
    }

    pub fn draw_if_declarable(&mut self) -> bool {
        if let Some(value) = self.can_declare_draw_partial() {
            value
        } else {
            self.game.declare_draw()
        }
    }

    pub fn side_to_move(&self) -> Color {
        assert_eq!(self.game.side_to_move(), self.side_to_move);
        self.side_to_move
    }

    pub fn move_count(&self) -> u32 {
        self.fullmove_number
    }
}

impl FromStr for BirchGame {
    type Err = Error;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        Ok(BirchGame::new_with_board(Board::from_str(fen)?))
    }
}

impl Display for BirchGame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fen = self.game.current_position().to_string();
        if fen.ends_with(" 0 1") {
            fen.truncate(fen.len() - 3);
            fen += &format!("{} {}", self.halfmove_clock, self.fullmove_number)
        }
        f.write_str(&fen)
    }
}

impl Deref for BirchGame {
    type Target = Game;

    fn deref(&self) -> &Self::Target {
        &self.game
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INITIAL_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[test]
    fn fen_default() {
        assert_eq!(
            BirchGame::new().to_string(),
            BirchGame::from_str(INITIAL_POSITION).unwrap().to_string()
        );
    }

    #[test]
    fn fen_continued() {
        // from https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
        let mut game = BirchGame::new();
        assert_eq!(INITIAL_POSITION, game.to_string());
        assert_eq!(Color::White, game.side_to_move());

        assert!(game.make_move(ChessMove::from_str("e2e4").unwrap()));
        assert_eq!(Color::Black, game.side_to_move());
        assert_eq!(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
            game.to_string()
        );

        assert!(game.make_move(ChessMove::from_str("c7c5").unwrap()));
        assert_eq!(Color::White, game.side_to_move());
        assert_eq!(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
            game.to_string()
        );

        assert!(game.make_move(ChessMove::from_str("g1f3").unwrap()));
        assert_eq!(Color::Black, game.side_to_move());
        assert_eq!(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
            game.to_string()
        );

        assert!(game.make_move(ChessMove::from_str("c5c4").unwrap()));
        assert_eq!(Color::White, game.side_to_move());
        assert!(game.make_move(ChessMove::from_str("b2b4").unwrap()));
        assert_eq!(Color::Black, game.side_to_move());
        assert_eq!(
            "rnbqkbnr/pp1ppppp/8/8/1Pp1P3/5N2/P1PP1PPP/RNBQKB1R b KQkq b4 0 3", // ep target is wrong!
            game.to_string()
        );
    }
}
