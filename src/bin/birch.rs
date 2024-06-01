use std::thread::sleep;
use std::time::Duration;

use vampirc_uci::UciMessage;

use bebchess::player::Player;

const GERALD: &str = "/Users/barneyb/IdeaProjects/Senior-Project-Chess-AI/chess/engine/base_engine";
const RACHEL: &str = "/Users/barneyb/IdeaProjects/bebchess/target/debug/rachel";

/// BIRCH: Barney's Incredibly Ridiculous Chess Harness
fn main() {
    println!("Hello, from BIRCH!");
    let mut white = Player::new("white", GERALD, |msg| match msg {
        UciMessage::UciOk => {}
        _ => {}
    });
    white.send(UciMessage::Uci);
    sleep(Duration::from_millis(1000));
    white.send(UciMessage::UciNewGame);
    sleep(Duration::from_millis(1000));
    white.send(UciMessage::Position {
        startpos: true,
        fen: None,
        moves: vec![],
    });
    sleep(Duration::from_millis(1000));
    white.send(UciMessage::go());
    sleep(Duration::from_millis(1000));
    white.send(UciMessage::Quit);
    sleep(Duration::from_millis(1000));

    if let Err(e) = white.close() {
        eprintln!("White failed to exit: {e}")
    }
}
