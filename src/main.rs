use std::cmp::Ordering;
use std::io;
use std::io::Error;
use std::pin::Pin;

use async_std::future::Future;
use async_std::task::block_on;
use futures::{
    join, AsyncRead, AsyncWriteExt, FutureExt, Sink, SinkExt, Stream, StreamExt, TryStreamExt,
};
use rand::Rng;
use vampirc_uci::UciMessage;

use crate::uci::*;

mod uci;

fn main() {
    println!("Guess the number (1-100)!");
    let secret_num = rand::thread_rng().gen_range(1..=100);
    loop {
        println!("Enter your guess:");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");
        let guess = guess.trim();
        if "quit".eq(guess) {
            break;
        }
        let guess: u32 = match guess.parse() {
            Ok(num) => num,
            Err(e) => {
                println!("um... {e}");
                continue;
            }
        };
        println!("Guess: {guess}");

        match guess.cmp(&secret_num) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too Big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }

    let (itx, irx) = new_try_channel();
    let (otx, orx) = new_channel();
    println!("Connected to UCI...");
    let q = block_on(async {
        let (_, r) = join!(
            run_std_loops(itx, orx),
            process_message(Box::pin(irx), &otx)
        );

        match r {
            Status::QUIT => return,
            _ => {}
        }
    });
}

enum Status {
    QUIT,
    NEXT,
}

async fn process_message(
    // engine: Arc<Engine>,
    mut msg_stream: Pin<Box<impl Stream<Item = io::Result<UciMessage>>>>,
    // msg_handler: &dyn MsgHandler,
    msg_sender: &UciSender,
) -> Status {
    while let Some(msg_r) = msg_stream.next().await {
        if let Ok(msg) = msg_r {
            match log_message(&msg) {
                Status::QUIT => break,
                _ => {}
            }
            // msg_handler.handle_msg(engine.as_ref(), &msg, msg_sender);
        } else {
            log_error(msg_r.err().unwrap());
        }
    }
    return Status::QUIT;
}

fn log_error(err: Error) {
    todo!()
}

fn log_message(msg: &UciMessage) -> Status {
    if msg.is_unknown() {
        println!("{msg}");
    }
    return match msg {
        UciMessage::Quit => Status::QUIT,
        _ => Status::NEXT,
    };
}
