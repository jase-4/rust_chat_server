use warp::Filter;

mod lib;
use lib::user_connected;

#[tokio::main]
async fn main(){

    let users = lib::Users::default();

    let users = warp::any().map(move || users.clone());
    
    let chat_server = warp::path("chat")
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws,users|{
            ws.on_upgrade(move|socket| user_connected(socket,users))
        });

        let index = warp::path::end().and(warp::fs::file("./html/index.html"));
        let routes = index.or(chat_server);

        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
