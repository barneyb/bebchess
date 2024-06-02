use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

use chess::{Board, ChessMove, Color, Error, Game};

pub struct BirchGame {
    game: Game,
    /// Track move number explicitly, in contrast to [Game] deriving it on
    /// demand from the game's actions.
    full_move_counter: usize,
}

impl BirchGame {
    pub fn new() -> BirchGame {
        Self::new_with_board(Board::default())
    }

    fn new_with_board(board: Board) -> BirchGame {
        Self::new_with_game(Game::new_with_board(board))
    }

    fn new_with_game(game: Game) -> BirchGame {
        BirchGame {
            full_move_counter: game.get_full_move_counter(),
            game,
        }
    }

    pub fn make_move(&mut self, m: ChessMove) -> bool {
        if self.game.make_move(m) {
            if self.game.side_to_move() == Color::White {
                // black moved, so increment
                self.full_move_counter += 1;
            }
            true
        } else {
            false
        }
    }

    /// Alias of [Game::declare_draw] which better communicates that it both
    /// tests and acts (if appropriate).
    pub fn declare_draw_if_appropriate(&mut self) -> bool {
        self.game.declare_draw()
    }

    /// Override [Game::get_full_move_counter] to supply the pre-computed value.
    pub fn get_full_move_counter(&self) -> usize {
        assert_eq!(self.full_move_counter, self.game.get_full_move_counter());
        self.full_move_counter
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
        write!(f, "{}", self.game)
    }
}

impl Deref for BirchGame {
    type Target = Game;

    fn deref(&self) -> &Self::Target {
        &self.game
    }
}
