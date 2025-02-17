use ockam::{async_worker, Context, Result, Routed, Worker};
use serde::{Deserialize, Serialize};

struct Square;

#[derive(Serialize, Deserialize)]
struct Num(usize);

#[async_worker]
impl Worker for Square {
    type Message = Num;
    type Context = Context;

    async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<Num>) -> Result<()> {
        println!("Getting square request for number {}", msg.0);
        ctx.send_message(msg.sender(), Num(msg.0 * msg.0)).await
    }
}

fn main() {
    let (mut app, mut exe) = ockam::start_node();

    exe.execute(async move {
        app.start_worker("io.ockam.square", Square).await.unwrap();

        let num = 3;
        app.send_message("io.ockam.square", Num(num)).await.unwrap();

        // block until it receives a message
        let square = app.receive::<Num>().await.unwrap();
        println!("App: {} ^ 2 = {}", num, square.0);

        app.stop().await.unwrap();
    })
    .unwrap();
}
