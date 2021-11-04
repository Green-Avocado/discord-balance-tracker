use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        Args, CommandResult, StandardFramework,
    },
    futures::StreamExt,
    model::{
        channel::Message,
        gateway::Ready,
        id::UserId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteraction,
                ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType,
            },
            Interaction, InteractionResponseType,
        },
    },
};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use tokio::sync::RwLock;
use typemap_rev::TypeMapKey;

use std::{collections::HashMap, error::Error, sync::Arc};

static PREFIX: &str = "!";

#[derive(Debug, Clone)]
struct HandleCommandError;

impl std::fmt::Display for HandleCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "could not get content")
    }
}

impl Error for HandleCommandError {}

#[derive(Debug, Clone)]
struct GetLockError;

impl std::fmt::Display for GetLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "could not get lock")
    }
}

impl Error for GetLockError {}

#[derive(Debug, Clone)]
struct ParseMoneyError;

impl std::fmt::Display for ParseMoneyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "could not parse money")
    }
}

impl Error for ParseMoneyError {}

#[derive(Debug, Clone)]
struct ParseMentionError;

impl std::fmt::Display for ParseMentionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "could not parse mention")
    }
}

impl Error for ParseMentionError {}

struct Accounts;

impl TypeMapKey for Accounts {
    type Value = Arc<RwLock<HashMap<UserId, i64>>>;
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
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "balance" => balance_handler(&ctx, &command).await,
                "owe" => owe_handler(&ctx, &command).await,
                "bill" => bill_handler(&ctx, &command).await,
                _ => Ok("not implemented".to_string()),
            };

            let content = match content {
                Ok(content) => content,
                Err(_e) => "Error getting content".to_string(),
            };

            if let Err(e) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("balance").description("Get balance")
                })
                .create_application_command(|command| {
                    command
                        .name("owe")
                        .description("Owe a user")
                        .create_option(|option| {
                            option
                                .name("user")
                                .description("User")
                                .kind(ApplicationCommandOptionType::User)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("amount")
                                .description("Amount")
                                .kind(ApplicationCommandOptionType::Integer)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("description")
                                .description("Transaction description")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    let mut command = command
                        .name("bill")
                        .description("Bill user(s) for transaction")
                        .create_option(|option| {
                            option
                                .name("amount")
                                .description("Amount")
                                .kind(ApplicationCommandOptionType::Integer)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("description")
                                .description("Transaction description")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        });

                    for i in 0..10 {
                        command = command.create_option(|option| {
                            option
                                .name(format!("user{}", i))
                                .description("User to bill")
                                .kind(ApplicationCommandOptionType::User)
                                .required(false)
                        })
                    }
                    command
                })
        })
        .await;

        println!(
            "I now have the following global slash commands: {:#?}",
            commands
        );
    }
}

async fn handle_signals(signals: Signals) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT => {
                std::process::exit(0);
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

    let application_id: u64 = std::env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Accounts>(Arc::new(RwLock::new(HashMap::new())));
        data.insert::<Members>(Arc::new(RwLock::new(Vec::new())));
    }

    if let Err(e) = client.start().await {
        println!("An error occurred while running the client: {:?}", e);
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
        match data_read.get::<Accounts>() {
            Some(data) => data.clone(),
            None => {
                msg.reply(ctx, "Error reading accounts data").await?;
                return Ok(());
            }
        }
    };

    let accounts = accounts_lock.read().await;

    if args.len() > 1 {
        msg.reply(ctx, format!("Usage: {}balance [@USER]", PREFIX))
            .await?;
        return Ok(());
    }

    match args.current() {
        Some(id) => {
            let balance = accounts
                .get(&match parse_mention(id) {
                    Ok(user_id) => user_id,
                    Err(_) => {
                        msg.reply(ctx, "Error parsing user id").await?;
                        return Ok(());
                    }
                })
                .map_or(0, |x| *x);
            msg.reply(ctx, format!("Their balance: {}", format_money(balance)))
                .await?;
        }
        None => {
            let balance = accounts.get(&msg.author.id).map_or(0, |x| *x);
            msg.reply(ctx, format!("Your balance: {}", format_money(balance)))
                .await?;
        }
    };

    Ok(())
}

#[command]
async fn owe(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() != 2 {
        msg.reply(ctx, format!("Usage: {}owe amount @USER", PREFIX))
            .await?;
        return Ok(());
    }

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
        Some(num) => match parse_money(num) {
            Ok(money) => {
                if money < 0 {
                    msg.reply(ctx, "Cannot bill negative amount").await?;
                    return Ok(());
                } else {
                    money
                }
            }
            Err(_) => {
                msg.reply(ctx, "Error parsing amount").await?;
                return Ok(());
            }
        },
        None => unreachable!(),
    };
    args.advance();

    let receiver = match args.current() {
        Some(id) => match parse_mention(id) {
            Ok(user_id) => user_id,
            Err(_) => {
                msg.reply(ctx, "Error parsing user id").await?;
                return Ok(());
            }
        },
        None => unreachable!(),
    };

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
        format!("Owed {} to <@{}>", format_money(amount), receiver),
    )
    .await?;

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
            Ok(money) => {
                if money < 0 {
                    msg.reply(ctx, "Cannot bill negative amount").await?;
                    return Ok(());
                } else {
                    money
                }
            }
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

async fn balance_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<String, HandleCommandError> {
    let accounts = match get_accounts_lock(&ctx).await {
        Ok(accounts_lock) => accounts_lock,
        Err(_) => return Err(HandleCommandError),
    };

    let balance = format_money(
        accounts
            .read()
            .await
            .get(&command.user.id)
            .map_or(0, |x| *x),
    );

    Ok(format!("Your balance: {}", balance))
}

async fn owe_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<String, HandleCommandError> {
    let mut amount = None;
    let mut user_opt = None;

    for option in &command.data.options {
        match option.name.as_ref() {
            "amount" => match &option.resolved {
                Some(value) => match value {
                    ApplicationCommandInteractionDataOptionValue::Integer(value) => {
                        amount = Some(*value);
                    }
                    _ => return Err(HandleCommandError),
                },
                None => return Err(HandleCommandError),
            },
            "user" => match &option.resolved {
                Some(value) => match value {
                    ApplicationCommandInteractionDataOptionValue::User(user, _member) => {
                        user_opt = Some(user);
                    }
                    _ => return Err(HandleCommandError),
                },
                None => return Err(HandleCommandError),
            },
            _ => return Err(HandleCommandError),
        }
    }

    if let Some(amount) = amount {
        if let Some(receiver) = user_opt {
            let accounts = match get_accounts_lock(&ctx).await {
                Ok(accounts_lock) => accounts_lock,
                Err(_) => return Err(HandleCommandError),
            };

            {
                let mut accounts = accounts.write().await;
                {
                    let receiver_entry = accounts.entry(receiver.id).or_insert(0);
                    *receiver_entry += amount;
                }
                {
                    let sender_entry = accounts.entry(command.user.id).or_insert(0);
                    *sender_entry -= amount;
                }
            }

            return Ok(format!("Owed {} to {}", amount, receiver.tag()));
        }
    }

    Err(HandleCommandError)
}

async fn bill_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<String, HandleCommandError> {
    let mut amount = None;
    let mut description = None;
    let mut users = Vec::new();

    for option in &command.data.options {
        match option.name.as_ref() {
            "amount" => match &option.resolved {
                Some(value) => match value {
                    ApplicationCommandInteractionDataOptionValue::Integer(value) => {
                        amount = Some(*value)
                    }
                    _ => return Err(HandleCommandError),
                },
                None => return Err(HandleCommandError),
            },
            "description" => match &option.resolved {
                Some(value) => match value {
                    ApplicationCommandInteractionDataOptionValue::String(value) => {
                        description = Some(value);
                    }
                    _ => return Err(HandleCommandError),
                },
                None => return Err(HandleCommandError),
            },
            _ => match &option.resolved {
                Some(value) => match value {
                    ApplicationCommandInteractionDataOptionValue::User(user, _member) => {
                        users.push(user);
                    }
                    _ => return Err(HandleCommandError),
                },
                None => {}
            },
        }
    }

    if let Some(amount) = amount {
        if let Some(description) = description {
            let accounts = match get_accounts_lock(&ctx).await {
                Ok(accounts_lock) => accounts_lock,
                Err(_) => return Err(HandleCommandError),
            };

            {
                let mut accounts = accounts.write().await;
                for user in &users {
                    let receiver_entry = accounts.entry(user.id).or_insert(0);
                    *receiver_entry -= amount;
                }
                {
                    let sender_entry = accounts.entry(command.user.id).or_insert(0);
                    *sender_entry += amount * users.len() as i64;
                }
            }

            return Ok(format!(
                "Billed {} to {} users for {}",
                amount,
                users.len(),
                description
            ));
        }
    }

    Err(HandleCommandError)
}

async fn get_accounts_lock(
    ctx: &Context,
) -> Result<Arc<RwLock<HashMap<UserId, i64>>>, GetLockError> {
    let accounts_lock = {
        let data_read = ctx.data.read().await;
        match data_read.get::<Accounts>() {
            Some(data) => data.clone(),
            None => return Err(GetLockError),
        }
    };

    Ok(accounts_lock)
}

fn format_money(money: i64) -> String {
    let mut string;
    if money >= 0 {
        string = format!("{:0>3}", money);
    } else {
        string = format!("-{:0>3}", -money);
    }
    string.insert(string.len() - 2, '.');
    string
}

fn parse_money(mut input: &str) -> Result<i64, ParseMoneyError> {
    let mut negative = false;

    if input.chars().nth(0).unwrap() == '-' {
        negative = true;
        input = &((*input)[1..]);
    }

    let mut split = (*input).split('.');

    let mut money = match split.next() {
        Some(dollars) => match dollars.parse::<u32>() {
            Ok(dollars) => dollars * 100,
            Err(_e) => return Err(ParseMoneyError),
        },
        None => return Err(ParseMoneyError),
    };

    if let Some(next) = split.next() {
        match next.parse::<u32>() {
            Ok(cents) => money += cents,
            Err(_e) => return Err(ParseMoneyError),
        };
    }

    if negative {
        Ok(-(i64::from(money)))
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
                Err(_e) => return Err(ParseMentionError),
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
                Err(_e) => {}
            };
        }
    }

    Err(ParseMentionError)
}
