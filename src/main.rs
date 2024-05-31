use std::io::Error;
use std::pin::Pin;
use std::thread;

use async_std::future::{ready, Future};
use async_std::io;
use async_std::prelude::*;
use async_std::task::block_on;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::future::abortable;
use futures::{
    join, AsyncRead, AsyncWriteExt, FutureExt, Sink, SinkExt, Stream, StreamExt, TryStreamExt,
};
use vampirc_uci::{parse_with_unknown, ByteVecUciMessage, UciMessage};

fn main() {
    // println!("Guess the number (1-100)!");
    // let secret_num = rand::thread_rng().gen_range(1..=100);
    // loop {
    //     println!("Enter your guess:");
    //     let mut guess = String::new();
    //     io::stdin()
    //         .read_line(&mut guess)
    //         .expect("Failed to read line");
    //     let guess: u32 = match guess.trim().parse() {
    //         Ok(num) => num,
    //         Err(e) => {
    //             println!("um... {e}");
    //             continue;
    //         }
    //     };
    //     println!("Guess: {guess}");
    //
    //     match guess.cmp(&secret_num) {
    //         Ordering::Less => println!("Too small!"),
    //         Ordering::Greater => println!("Too Big!"),
    //         Ordering::Equal => {
    //             println!("You win!");
    //             break;
    //         }
    //     }
    // }

    let (itx, irx) = new_try_channel();
    let (otx, orx) = new_channel();
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

/* BEGIN VENDOR vampirc-io */

pub type UciStream = dyn Stream<Item = Result<UciMessage, io::Error>> + Unpin + Send + Sync;
pub type UciSink = dyn Sink<UciMessage, Error = io::Error> + Unpin + Send;
pub type UciSender = UnboundedSender<UciMessage>;
pub type UciReceiver = UnboundedReceiver<UciMessage>;
pub type UciTrySender = UnboundedSender<io::Result<UciMessage>>;
pub type UciTryReceiver = UnboundedReceiver<io::Result<UciMessage>>;

pub fn from_reader<'a, R>(reader: io::BufReader<R>) -> Box<UciStream>
where
    R: AsyncRead + Unpin + Sync + Send + 'static,
{
    let stream = reader
        .lines()
        .map_ok(|line| parse_with_unknown(&(line + "\n")))
        .map_ok(|msg_list| msg_list[0].clone());

    Box::new(stream)
}

pub fn stdin_msg_stream() -> Box<UciStream> {
    from_reader(io::BufReader::new(io::stdin()))
}

pub fn stdout_msg_sink() -> Box<UciSink> {
    let sink = io::stdout()
        .into_sink()
        .buffer(256)
        .with(|msg: UciMessage| ready(Ok(ByteVecUciMessage::from(msg))).boxed());
    Box::new(sink)
}

pub async fn run_loops(
    mut inbound_source: Box<UciStream>,
    mut inbound_consumer: UciTrySender,
    mut outbound_source: UciReceiver,
    mut outbound_consumer: Box<UciSink>,
) {
    let outb = async {
        while let Some(msg) = StreamExt::next(&mut outbound_source).await {
            let sr = outbound_consumer.send(msg).await;
            if let Err(err) = sr {
                eprintln!(
                    "[{:?}] Error while sending message through the outbound channel: {}",
                    thread::current().id(),
                    err
                );
            }
        }
    };

    let (aoutb, aoutbh) = abortable(outb);

    let inb = async {
        loop {
            let msg_result = inbound_source.try_next().await;
            if let Ok(msg_opt) = msg_result {
                if msg_opt.is_none() {
                    break;
                } else {
                    let msg = msg_opt.unwrap();
                    let quit = match msg {
                        UciMessage::Quit => true,
                        _ => false,
                    };
                    inbound_consumer.send(Ok(msg)).await.unwrap();
                    if quit {
                        aoutbh.abort();
                        break;
                    }
                }
            } else {
                inbound_consumer
                    .send(Err(msg_result.err().unwrap()))
                    .await
                    .unwrap();
            }
        }
    };

    join!(inb, aoutb);
}

pub async fn run_std_loops(inbound_consumer: UciTrySender, outbound_source: UciReceiver) {
    run_loops(
        stdin_msg_stream(),
        inbound_consumer,
        outbound_source,
        stdout_msg_sink(),
    )
    .await;
}

pub fn new_channel() -> (UciSender, UciReceiver) {
    unbounded::<UciMessage>()
}

pub fn new_try_channel() -> (UciTrySender, UciTryReceiver) {
    unbounded::<io::Result<UciMessage>>()
}

/* END VENDOR vampirc-io */

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
