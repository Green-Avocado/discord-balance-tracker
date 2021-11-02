use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework, Args,
    },
    model::{channel::Message, prelude::Ready},
    futures::StreamExt,
};
use typemap_rev::TypeMapKey;
use signal_hook::{consts::signal::*};
use signal_hook_tokio::Signals;

use std::{env, sync::Arc, process::exit, collections::HashMap};

struct Debt {
    user: u64,
    amount: i32,
}

struct Accounts;

    impl TypeMapKey for Accounts {
        type Value = Arc<HashMap<u64, Vec<Debt>>>;
    }

#[group]
#[commands(balance, pay, bill)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn handle_signals(signals: Signals) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT => {
                exit(0);
            },
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() {
    let signals = Signals::new(&[
        SIGTERM,
        SIGINT,
    ]).unwrap();
    tokio::spawn(handle_signals(signals));

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
        data.insert::<Accounts>(Arc::new(HashMap::new()));
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn balance(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.len() {
        0 => msg.reply(ctx, "Your balance: ").await?,
        1 => msg.reply(ctx, "Their balance: ").await?,
        _ => msg.reply(ctx, "Usage: ").await?,
    };

    Ok(())
}

#[command]
async fn pay(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(ctx, "Pay").await?;

    Ok(())
}

#[command]
async fn bill(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(ctx, "Charge").await?;

    Ok(())
}
