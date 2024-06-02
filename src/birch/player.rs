use std::io;
use std::process::{Command, ExitStatus};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use chess::Color;
use interactive_process::InteractiveProcess;
use vampirc_uci::{parse_with_unknown, Serializable, UciMessage};

pub struct Player {
    color: Color,
    proc: InteractiveProcess,
    sent_quit: bool,
}

fn label(color: Color) -> &'static str {
    match color {
        Color::White => "WHITE",
        Color::Black => "black",
    }
}

impl Player {
    pub fn new(
        color: Color,
        cmd_str: &str,
        sender: Arc<Mutex<Sender<(Color, UciMessage)>>>,
    ) -> Player {
        let mut cmd = Command::new(cmd_str);
        let proc = InteractiveProcess::new(&mut cmd, move |r| match r {
            Ok(line) => {
                for msg in parse_with_unknown(&line) {
                    // println!("[{}] < \t{msg}", label(color));
                    sender.lock().unwrap().send((color, msg)).unwrap();
                }
            }
            Err(e) => {
                println!("[{}] ! {e}", label(color));
            }
        })
        .unwrap();
        Player {
            color,
            proc,
            sent_quit: false,
        }
    }

    pub fn send(&mut self, message: UciMessage) {
        if let UciMessage::Quit = message {
            self.sent_quit = true
        }
        let msg = message.serialize();
        let msg = msg.trim();
        // println!("[{}] > {msg}", label(self.color));
        if let Err(e) = self.proc.send(msg) {
            println!("[{}] ! {e}", label(self.color));
        }
    }

    pub fn close(mut self) -> io::Result<Option<ExitStatus>> {
        // todo: make this Drop
        if !self.sent_quit {
            self.send(UciMessage::Quit);
            sleep(Duration::from_millis(50));
        }
        let mut child = self.proc.close();
        if let Ok(None) = child.try_wait() {
            child.kill().expect("Failed to kill child");
            println!("[{}] ! killed still-running engine", label(self.color));
        }
        child.try_wait()
    }
}
