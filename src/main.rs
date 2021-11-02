use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
    model::{channel::Message, prelude::Ready},
};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use typemap_rev::TypeMapKey;

use std::{env, sync::Arc};

struct WriteQueueSender;

impl TypeMapKey for WriteQueueSender {
    type Value = Arc<Sender<String>>;
}

#[group]
#[commands(balance, pay, charge)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx): (Sender<String>, Receiver<String>) = channel(16);

    tokio::spawn(async move {
        let mut rx = rx;
        loop {
            let received = rx.recv().await.unwrap();
            println!("Got: {}", received);
        }
    });

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<WriteQueueSender>(Arc::new(tx));
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn balance(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Balance: ").await?;

    Ok(())
}

#[command]
async fn pay(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pay").await?;

    let tx = ctx
        .data
        .read()
        .await
        .get::<WriteQueueSender>()
        .unwrap()
        .clone();
    tx.send("Payment".to_string()).await.unwrap();

    Ok(())
}

#[command]
async fn charge(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Charge").await?;

    let tx = ctx
        .data
        .read()
        .await
        .get::<WriteQueueSender>()
        .unwrap()
        .clone();
    tx.send("Charge".to_string()).await.unwrap();

    Ok(())
}
