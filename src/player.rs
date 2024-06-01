use std::fmt::Display;
use std::io;
use std::process::{Command, ExitStatus};
use std::thread::sleep;
use std::time::Duration;

use interactive_process::InteractiveProcess;
use vampirc_uci::{parse_with_unknown, Serializable, UciMessage};

pub struct Player {
    label: &'static str,
    cmd: Command,
    proc: InteractiveProcess,
    sent_quit: bool,
}

impl Player {
    pub fn new<T>(label: &'static str, cmd_str: &str, msg_callback: T) -> Player
    where
        T: Fn(&UciMessage) + Send + 'static,
    {
        let mut cmd = Command::new(cmd_str);
        let proc = InteractiveProcess::new(&mut cmd, move |r| match r {
            Ok(line) => {
                for msg in parse_with_unknown(&line) {
                    println!("[{label}] < \t{msg}");
                    msg_callback(&msg);
                }
            }
            Err(e) => {
                println!("[{label}] ! {e}");
            }
        })
        .unwrap();
        Player {
            label,
            cmd,
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
        println!("[{}] > {msg}", self.label);
        if let Err(e) = self.proc.send(msg) {
            println!("[{}] ! {e}", self.label);
        }
    }

    pub fn close(mut self) -> io::Result<Option<ExitStatus>> {
        if !self.sent_quit {
            self.send(UciMessage::Quit);
            sleep(Duration::from_millis(500));
        }
        let mut child = self.proc.close();
        if let Ok(None) = child.try_wait() {
            child.kill();
            println!("[{}] ! killed still-running engine", self.label);
        }
        child.try_wait()
    }
}
