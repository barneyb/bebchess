pub mod birch_game;
pub mod player;
pub mod players;

/// Sanity checks for FEN behavior of [Board]. Its published version has a
/// defect with the en passant square's rank.
#[cfg(test)]
mod board_test {
    use std::str::FromStr;

    use chess::{Board, ChessMove, Square};

    #[test]
    fn fen_en_passant() {
        let mut board = Board::default();
        assert_eq!(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            format!("{}", board)
        );
        board = board.make_move_new(ChessMove::new(Square::E2, Square::E4, None));
        assert_eq!(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            format!("{}", board)
        );
        board = board.make_move_new(ChessMove::new(Square::C7, Square::C5, None));
        assert_eq!(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 1",
            format!("{}", board)
        );
        board = board.make_move_new(ChessMove::new(Square::G1, Square::F3, None));
        let final_serialized_game = format!("{}", board);
        assert_eq!(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 0 1",
            final_serialized_game
        );
        assert_eq!(
            final_serialized_game,
            format!("{}", Board::from_str(&final_serialized_game).unwrap()),
        );
    }
}

/// Sanity checks for FEN behavior of [Game]. Its published version has a defect
/// with the halfmove/move counters. It also "inherits" [chess::Board]'s defect
/// with en passant rank.
#[cfg(test)]
mod game_test {
    use std::str::FromStr;

    use chess::{ChessMove, Color, Game};

    const INITIAL_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[test]
    fn fen_default() {
        assert_eq!(
            Game::new().to_string(),
            Game::from_str(INITIAL_POSITION).unwrap().to_string()
        );
    }

    #[test]
    fn fen_continued() {
        // from https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
        let mut game = Game::new();
        assert_eq!(INITIAL_POSITION, game.to_string());
        assert_eq!(Color::White, game.side_to_move());

        assert!(game.make_move(ChessMove::from_str("e2e4").unwrap()));
        assert_eq!(Color::Black, game.side_to_move());
        assert_eq!(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            game.to_string()
        );

        assert!(game.make_move(ChessMove::from_str("c7c5").unwrap()));
        assert_eq!(Color::White, game.side_to_move());
        assert_eq!(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
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
            "rnbqkbnr/pp1ppppp/8/8/1Pp1P3/5N2/P1PP1PPP/RNBQKB1R b KQkq b3 0 3",
            game.to_string()
        );
    }

    /// Vendored from non-`pub` [Game::fake_pgn_parser]
    fn fake_pgn_parser(moves: &str) -> Game {
        moves
            .split_whitespace()
            .filter(|s| !s.ends_with("."))
            .fold(Game::new(), |mut g, m| {
                g.make_move(ChessMove::from_san(&g.current_position(), m).expect("Valid SAN Move"));
                g
            })
    }

    #[test]
    fn fen_en_passant() {
        let game = fake_pgn_parser(
            "
             1. e4    e6
             2. d4    d6
             3. Nf3   Be7
             4. Nc3   a6
             5. Bc4   Nc6
             6. d5    exd5
             7. exd5  Ne5
             8. Nxe5  dxe5
             9. O-O   Bd6
            10. Re1   h6
            11. Ne4   Ne7
            12. Nxd6+ Qxd6
            13. b3    O-O
            14. Bb2   f6
            15. a4    Ng6
            16. Ba3   c5",
        );
        assert_eq!(
            "r1b2rk1/1p4p1/p2q1pnp/2pPp3/P1B5/BP6/2P2PPP/R2QR1K1 w - c6 0 17",
            game.to_string()
        );
    }
}
