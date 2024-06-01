use std::io;
use std::pin::Pin;

use async_std::future::Future;
use async_std::task::block_on;
use futures::{join, SinkExt, Stream, StreamExt};
use vampirc_uci::UciMessage;

use bebchess::uci::*;

/// RACHEL: Really Awful CHess Engine for Learning
fn main() {
    let (itx, irx) = new_try_channel();
    let (mut otx, orx) = new_channel();
    block_on(async {
        join!(
            run_std_loops(itx, orx),
            process_messages(Box::pin(irx), &mut otx)
        );
    });
}

async fn process_messages(
    // engine: Arc<Engine>,
    mut msg_stream: Pin<Box<impl Stream<Item = io::Result<UciMessage>>>>,
    // msg_handler: &dyn MsgHandler,
    msg_sender: &mut UciSender,
) {
    while let Some(msg_r) = msg_stream.next().await {
        if let Ok(msg) = msg_r {
            eprintln!("[RACHEL] < {msg}");
            match msg {
                UciMessage::Quit => break,
                UciMessage::Uci => {
                    msg_sender
                        .send(UciMessage::id_name("Rachel"))
                        .await
                        .expect("Failed to send name");
                    msg_sender
                        .send(UciMessage::id_author("Barney Boisvert"))
                        .await
                        .expect("Failed to send author");
                    msg_sender
                        .send(UciMessage::UciOk)
                        .await
                        .expect("Failed to send OK");
                }
                _ => {}
            };
            // msg_handler.handle_msg(engine.as_ref(), &msg, msg_sender);
        } else {
            eprintln!("[RACHEL] ! {}", msg_r.err().unwrap());
        }
    }
}
