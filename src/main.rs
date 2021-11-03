use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        Args, CommandResult, StandardFramework,
    },
    futures::StreamExt,
    model::{channel::Message, id::UserId, prelude::Ready},
};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use tokio::sync::RwLock;
use typemap_rev::TypeMapKey;

use std::{collections::HashMap, env, num::ParseIntError, process::exit, sync::Arc};

static PREFIX: &str = "!";

struct Accounts;

impl TypeMapKey for Accounts {
    type Value = Arc<RwLock<HashMap<UserId, i32>>>;
}

struct Members;

impl TypeMapKey for Members {
    type Value = Arc<RwLock<Vec<u64>>>;
}

#[group]
#[commands(group, balance, pay, bill)]
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
            }
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() {
    let signals = Signals::new(&[SIGTERM, SIGINT]).unwrap();
    tokio::spawn(handle_signals(signals));

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(PREFIX))
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Accounts>(Arc::new(RwLock::new(HashMap::new())));
        data.insert::<Members>(Arc::new(RwLock::new(Vec::new())));
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn group(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() == 0 {
        msg.reply(ctx, "group").await?;
    } else {
        match args.current().unwrap() {
            "add" => {
                if args.remaining() > 1 {
                    args.advance();
                    for arg in args.iter::<u64>() {
                        let arg = arg.unwrap_or(0);
                        let group_lock = {
                            let data_read = ctx.data.read().await;
                            data_read.get::<Members>().unwrap().clone()
                        };

                        {
                            let mut group = group_lock.write().await;
                            if !group.contains(&arg) {
                                group.push(arg);
                            }
                        }
                    }
                    msg.reply(ctx, "Added users from group").await?;
                } else {
                    msg.reply(ctx, format!("Usage: {}group add USERS", PREFIX))
                        .await?;
                }
            }
            "remove" => {
                if args.remaining() > 1 {
                    args.advance();
                    for arg in args.iter::<u64>() {
                        let arg = arg.unwrap_or(0);
                        let group_lock = {
                            let data_read = ctx.data.read().await;
                            data_read.get::<Members>().unwrap().clone()
                        };

                        {
                            let mut i = 0;
                            let mut group = group_lock.write().await;
                            for user in group.iter() {
                                if *user == arg {
                                    break;
                                }
                                i += 1;
                            }
                            group.remove(i);
                        }
                    }
                    msg.reply(ctx, "Removed users from group").await?;
                } else {
                    msg.reply(ctx, format!("Usage: {}group remove USERS", PREFIX))
                        .await?;
                }
            }
            "list" => {
                if args.remaining() == 1 {
                    let group_lock = {
                        let data_read = ctx.data.read().await;
                        data_read.get::<Members>().unwrap().clone()
                    };
                    let group = group_lock.read().await;

                    if group.is_empty() {
                        msg.reply(ctx, "No users in group").await?;
                    } else {
                        let user_list: String = group
                            .iter()
                            .map(|x| format!("<@{}> {}\n", *x, *x))
                            .collect();
                        msg.reply(ctx, user_list).await?;
                    }
                } else {
                    msg.reply(ctx, format!("Usage: {}group list", PREFIX))
                        .await?;
                }
            }
            _ => {
                msg.reply(ctx, format!("Usage: {}group COMMAND", PREFIX))
                    .await?;
            }
        };
    }

    Ok(())
}

#[command]
async fn balance(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let accounts_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<Accounts>().unwrap().clone()
    };

    let accounts = accounts_lock.read().await;

    match args.len() {
        0 => {
            let balance = accounts.get(&msg.author.id).map_or(0, |x| *x);
            msg.reply(ctx, format_money(balance)).await?
        }
        1 => {
            let balance = accounts
                .get(&UserId(args.parse::<u64>().unwrap_or(0)))
                .map_or(0, |x| *x);
            msg.reply(ctx, format_money(balance)).await?
        }
        _ => {
            msg.reply(ctx, format!("Usage: {}balance [USER]", PREFIX))
                .await?
        }
    };

    Ok(())
}

#[command]
async fn pay(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() == 2 {
        let accounts_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<Accounts>().unwrap().clone()
        };

        let amount = parse_money(args.current().unwrap()).unwrap_or(0);
        let receiver = args.single::<u64>().unwrap_or(0);

        {
            let mut accounts = accounts_lock.write().await;
            {
                let receiver_entry = accounts.entry(UserId(receiver)).or_insert(0);
                *receiver_entry += amount;
            }
            {
                let sender_entry = accounts.entry(msg.author.id).or_insert(0);
                *sender_entry -= amount;
            }
        }
    } else {
        msg.reply(ctx, format!("Usage: {}pay amount USER", PREFIX))
            .await?;
    }

    Ok(())
}

#[command]
async fn bill(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let accounts_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<Accounts>().unwrap().clone()
    };

    match args.len() {
        0 => {
            msg.reply(ctx, format!("Usage: {}bill amount [USERS]", PREFIX))
                .await?;
        }
        1 => {
            let amount = parse_money(args.current().unwrap()).unwrap_or(0);
            let group_lock = {
                let data_read = ctx.data.read().await;
                data_read.get::<Members>().unwrap().clone()
            };
            let group = group_lock.read().await;

            let mut accounts = accounts_lock.write().await;
            let mut i = 0;
            for user in group.iter() {
                let receiver_entry = accounts.entry(UserId(*user)).or_insert(0);
                *receiver_entry -= amount;
                i += 1;
            }
            {
                let sender_entry = accounts.entry(msg.author.id).or_insert(0);
                *sender_entry += amount * i;
            }
        }
        _ => {
            let amount = parse_money(args.current().unwrap()).unwrap_or(0);
            args.advance();

            let mut accounts = accounts_lock.write().await;
            let mut i = 0;
            for arg in args.iter::<u64>() {
                let arg = arg.unwrap_or(0);
                let receiver_entry = accounts.entry(UserId(arg)).or_insert(0);
                *receiver_entry -= amount;
                i += 1;
            }
            {
                let sender_entry = accounts.entry(msg.author.id).or_insert(0);
                *sender_entry += amount * i;
            }
        }
    }

    Ok(())
}

fn format_money(money: i32) -> String {
    let mut string;
    if money >= 0 {
        string = format!("{:0>3}", money);
    } else {
        string = format!("-{:0>3}", -money);
    }
    string.insert(string.len() - 2, '.');
    string
}

fn parse_money(input: &str) -> Result<i32, ParseIntError> {
    let mut negative = false;

    if input.chars().nth(0).unwrap() == '-' {
        negative = true;
    }

    let mut split = input.split('.');
    let mut money = split.next().unwrap().parse::<i32>().unwrap_or(0) * 100;
    let next = split.next();
    if next.is_some() {
        let cents = next.unwrap().parse::<i32>().unwrap_or(0);
        if negative {
            money -= cents;
        } else {
            money += cents;
        }
    }

    Ok(money)
}
