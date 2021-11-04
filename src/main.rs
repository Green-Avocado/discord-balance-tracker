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

use std::{collections::HashMap, error::Error, fmt, process::exit, sync::Arc};

static PREFIX: &str = "!";

#[derive(Debug, Clone)]
struct ParseMoneyError;

impl fmt::Display for ParseMoneyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not parse money")
    }
}

impl Error for ParseMoneyError {}

#[derive(Debug, Clone)]
struct ParseMentionError;

impl fmt::Display for ParseMentionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not parse mention")
    }
}

impl Error for ParseMentionError {}

struct Accounts;

impl TypeMapKey for Accounts {
    type Value = Arc<RwLock<HashMap<UserId, i32>>>;
}

struct Members;

impl TypeMapKey for Members {
    type Value = Arc<RwLock<Vec<UserId>>>;
}

#[group]
#[commands(group, balance, owe, bill)]
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

    let token = std::env::var("DISCORD_TOKEN").expect("token");
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
                    let group_lock = {
                        let data_read = ctx.data.read().await;
                        data_read.get::<Members>().unwrap().clone()
                    };

                    while !args.is_empty() {
                        let arg = parse_mention(args.current().unwrap()).unwrap();
                        args.advance();

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
                    let group_lock = {
                        let data_read = ctx.data.read().await;
                        data_read.get::<Members>().unwrap().clone()
                    };

                    while !args.is_empty() {
                        let arg = parse_mention(args.current().unwrap()).unwrap();
                        args.advance();

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

                    let accounts_lock = {
                        let data_read = ctx.data.read().await;
                        data_read.get::<Accounts>().unwrap().clone()
                    };
                    let accounts = accounts_lock.read().await;

                    if group.is_empty() {
                        msg.reply(ctx, "No users in group").await?;
                    } else {
                        let user_list: String = group
                            .iter()
                            .map(|x| {
                                format!(
                                    "<@{}>: {}\n",
                                    *x,
                                    format_money(accounts.get(x).map_or(0, |x| *x))
                                )
                            })
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
            msg.reply(ctx, format!("Your balance: {}", format_money(balance)))
                .await?
        }
        1 => {
            let balance = accounts
                .get(&parse_mention(args.current().unwrap()).unwrap())
                .map_or(0, |x| *x);
            msg.reply(ctx, format!("Their balance: {}", format_money(balance)))
                .await?
        }
        _ => {
            msg.reply(ctx, format!("Usage: {}balance [@USER]", PREFIX))
                .await?
        }
    };

    Ok(())
}

#[command]
async fn owe(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() == 2 {
        let accounts_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<Accounts>().unwrap().clone()
        };

        let amount = parse_money(args.current().unwrap()).unwrap();
        args.advance();
        let receiver = parse_mention(args.current().unwrap()).unwrap();

        {
            let mut accounts = accounts_lock.write().await;
            {
                let receiver_entry = accounts.entry(receiver).or_insert(0);
                *receiver_entry += amount;
            }
            {
                let sender_entry = accounts.entry(msg.author.id).or_insert(0);
                *sender_entry -= amount;
            }
        }
        msg.reply(
            ctx,
            format!("Owe {} to <@{}>", format_money(amount), receiver),
        )
        .await?;
    } else {
        msg.reply(ctx, format!("Usage: {}owe amount @USER", PREFIX))
            .await?;
    }

    Ok(())
}

#[command]
async fn bill(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let accounts_lock = {
        let data_read = ctx.data.read().await;
        match data_read.get::<Accounts>() {
            Some(data) => data.clone(),
            None => {
                msg.reply(ctx, "Error reading accounts data").await?;
                return Ok(());
            }
        }
    };

    let amount = match args.current() {
        Some(str) => match parse_money(str) {
            Ok(money) => money,
            Err(_) => {
                msg.reply(ctx, "Error parsing amount").await?;
                return Ok(());
            }
        },
        None => {
            msg.reply(ctx, format!("Usage: {}bill amount [@USERS]", PREFIX))
                .await?;
            return Ok(());
        }
    };
    args.advance();

    let user_list = match args.remaining() {
        0 => {
            let group_lock = {
                let data_read = ctx.data.read().await;
                match data_read.get::<Members>() {
                    Some(data) => data.clone(),
                    None => {
                        msg.reply(ctx, "Error reading group data").await?;
                        return Ok(());
                    }
                }
            };
            let vec = group_lock.read().await.clone();
            vec
        }
        _ => match parse_mention_list(&mut args) {
            Ok(list) => list,
            Err(_) => {
                msg.reply(ctx, "Error parsing user ids").await?;
                return Ok(());
            }
        },
    };

    let mut accounts = accounts_lock.write().await;
    let mut i = 0;
    for user in user_list {
        let receiver_entry = accounts.entry(user).or_insert(0);
        *receiver_entry -= amount;
        i += 1;
    }
    {
        let sender_entry = accounts.entry(msg.author.id).or_insert(0);
        *sender_entry += amount * i;
    }
    msg.reply(ctx, "Billed").await?;

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

fn parse_money(mut input: &str) -> Result<i32, ParseMoneyError> {
    let mut negative = false;

    if input.chars().nth(0).unwrap() == '-' {
        negative = true;
        input = &((*input)[1..]);
    }

    let mut split = (*input).split('.');

    let mut money = match split.next() {
        Some(dollars) => match dollars.parse::<u16>() {
            Ok(dollars) => dollars * 100,
            Err(_) => return Err(ParseMoneyError),
        },
        None => return Err(ParseMoneyError),
    };

    if let Some(next) = split.next() {
        match next.parse::<u16>() {
            Ok(cents) => money += cents,
            Err(_) => return Err(ParseMoneyError),
        };
    }

    if negative {
        Ok(-(i32::from(money)))
    } else {
        Ok(money.into())
    }
}

fn parse_mention_list(inputs: &mut Args) -> Result<Vec<UserId>, ParseMentionError> {
    let mut vec = Vec::new();

    loop {
        let user_id = match inputs.current() {
            Some(id) => match parse_mention(id) {
                Ok(user_id) => user_id,
                Err(_) => return Err(ParseMentionError),
            },
            None => break,
        };

        vec.push(user_id);

        inputs.advance();
    }

    Ok(vec)
}

fn parse_mention(input: &str) -> Result<UserId, ParseMentionError> {
    if let Some(s) = input.strip_prefix("<@!") {
        if let Some(s) = s.strip_suffix(">") {
            match s.parse::<u64>() {
                Ok(id) => return Ok(UserId(id)),
                Err(_) => {}
            };
        }
    }

    Err(ParseMentionError)
}
