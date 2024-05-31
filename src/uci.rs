use std::thread;

use async_std::future::{ready, Future};
use async_std::io;
use async_std::prelude::*;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::future::abortable;
use futures::{
    join, AsyncRead, AsyncWriteExt, FutureExt, Sink, SinkExt, Stream, StreamExt, TryStreamExt,
};
use vampirc_uci::{parse_with_unknown, ByteVecUciMessage, UciMessage};

/*
This module started as a vendoring of vampirc-oi, which seems to be abandonware,
but served as an ok-enough starting point for async UCI OI. The initial weakness
which prompted the fork was the inability to abort on a "quit" message.
 */

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
        .map_ok(|line| parse_with_unknown(&line))
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
