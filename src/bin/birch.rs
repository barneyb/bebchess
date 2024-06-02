use std::sync::mpsc;

use chess::Color;
use chess::GameResult;
use vampirc_uci::UciMessage;

use bebchess::birch::birch_game::BirchGame;
use bebchess::birch::players::Players;

const GERALD: &str = "/Users/barneyb/IdeaProjects/Senior-Project-Chess-AI/chess/engine/base_engine";
const RACHEL: &str = "/Users/barneyb/IdeaProjects/bebchess/target/debug/rachel";

/// BIRCH: Barney's Incredibly Ridiculous Chess Harness
fn main() {
    println!("Hello, from BIRCH!");
    let (tx, rx) = mpsc::channel();
    let mut players = Players::new(tx, GERALD, GERALD);
    let mut game = Box::new(BirchGame::new());
    // use std::str::FromStr;
    // let fen = "8/6n1/8/3k4/1K6/8/8/8 w - - 0 79";
    // let mut game = Box::new(BirchGame::from_str(fen).expect("Valid FEN"));
    // println!("[FEN \"{game}\"]");
    // println!("[SetUp \"1\"]");

    // todo: need to handle an engine crash
    'message_loop: for (c, msg) in rx.iter() {
        match &msg {
            UciMessage::Id {
                name: Some(name), ..
            } => {
                println!("[{c:?} \"{name}\"]")
            }
            UciMessage::Id { .. } | UciMessage::Option(_) | UciMessage::Info(_) => {}
            UciMessage::UciOk => {
                // set options
                players.send(c, UciMessage::IsReady);
            }
            UciMessage::ReadyOk => {
                players.send(c, UciMessage::UciNewGame);
                if c == game.side_to_move() {
                    players.next_turn(&game);
                }
            }
            UciMessage::BestMove { best_move: m, .. } => {
                if c == game.side_to_move() {
                    if game.make_move(*m) {
                        if c == Color::White {
                            print!("{}. {m} {{ {game} }}", game.get_full_move_counter());
                        } else {
                            println!(" {m} {{ {game} }}");
                        }
                    } else {
                        panic!("{:?} made illegal '{m}' at '{}'", c, game)
                    }
                    game.declare_draw_if_appropriate();
                    if let Some(_) = game.result() {
                        println!();
                        break 'message_loop;
                    } else {
                        players.next_turn(&game);
                    }
                } else {
                    eprintln!(
                        "{c:?} unexpectedly sent '{}' on {:?}'s turn",
                        msg,
                        game.side_to_move()
                    );
                }
            }
            UciMessage::Registration(_) | UciMessage::CopyProtection(_) => {
                eprintln!("Received unexpected {}", msg)
            }
            UciMessage::Unknown(_, Some(e)) => {
                eprintln!("Received unknown {msg}: {e}")
            }
            UciMessage::Unknown(_, _) => {
                eprintln!("Received unknown {msg}")
            }
            um => {
                eprintln!("Received unexpected {}", um)
            }
        }
    }
    players.close();
    if let Some(gr) = game.result() {
        println!("{gr:?} in {} moves: {game}", game.get_full_move_counter());
        if let Some(victor) = match gr {
            GameResult::WhiteCheckmates | GameResult::BlackResigns => Some(Color::White),
            GameResult::BlackCheckmates | GameResult::WhiteResigns => Some(Color::Black),
            GameResult::Stalemate | GameResult::DrawAccepted | GameResult::DrawDeclared => None,
        } {
            println!("{:?} wins!", victor)
        }
    } else {
        println!("Game over, but no result?!");
    }
}
