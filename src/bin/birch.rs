use std::sync::mpsc;

use chess::Action;
use chess::GameResult;
use chess::{Color, Game};
use vampirc_uci::UciMessage;

use bebchess::birch::players::Players;

const GERALD: &str = "/Users/barneyb/IdeaProjects/Senior-Project-Chess-AI/chess/engine/base_engine";
const RACHEL: &str = "/Users/barneyb/IdeaProjects/bebchess/target/debug/rachel";

/// BIRCH: Barney's Incredibly Ridiculous Chess Harness
fn main() {
    println!("Hello, from BIRCH!");
    let (tx, rx) = mpsc::channel();
    let mut players = Players::new(tx, GERALD, GERALD);
    let mut game = Box::new(Game::new());
    let mut move_count: u32 = 1;

    'message_loop: for (c, msg) in rx.iter() {
        match &msg {
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
                    game.make_move(*m);
                    if let Some(_) = game.result() {
                        break 'message_loop;
                    } else if game.can_declare_draw() {
                        game.declare_draw();
                        break 'message_loop;
                    } else {
                        if c == Color::Black {
                            println!("after {:>3}: {}", move_count, game.current_position());
                            move_count += 1;
                        }
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
        let move_count = game
            .actions()
            .iter()
            .filter(|a| {
                if let Action::MakeMove(_) = a {
                    true
                } else {
                    false
                }
            })
            .count()
            / 2;
        println!(
            "{:?} in {} moves: {}",
            gr,
            move_count,
            game.current_position()
        );
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
