use std::collections::HashMap;
use std::sync::{atomic::{AtomicUsize,Ordering},
    Arc
};

use warp:: ws::{WebSocket, Message};
use futures_util::{StreamExt, SinkExt, TryFutureExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

static USER_ID: AtomicUsize = AtomicUsize::new(1);

pub type Users = Arc<RwLock<HashMap<usize, UnboundedSender<Message>>>>;

pub async fn user_connected(ws: WebSocket,users: Users){
    let id = USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("New User Has Joined");
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    let (tx,rx) = unbounded_channel();

    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move{
        while let Some(message) = rx.next().await{
            user_ws_tx.send(message)
            .unwrap_or_else(|e|{
                eprintln!("Error with websocket send: {}",e);
            })
            .await;
        }
    });

    users.write().await.insert(id, tx);

    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg)=>msg,
            Err(e) => {
                eprintln!("websocket error: {}", e);
                break;
            }
        };
        send_message(id,msg,&users).await;
    }

    user_disconnected(id, &users).await;

}

async fn send_message(id:usize,msg:Message,users: &Users){
    let msg = if let Ok(s) = msg.to_str(){
        s
    }else{
        return;
    };

    let new_msg = format!("User<{}>: {}",id,msg);

    for (&uid, tx) in users.read().await.iter() {
        if id != uid {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    
    users.write().await.remove(&my_id);
}