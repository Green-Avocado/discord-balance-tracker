use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    futures::StreamExt,
    model::{
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

use std::{collections::HashMap, error::Error, fmt, fmt::Write, sync::Arc};

#[derive(Debug, Clone)]
struct HandleCommandError;

impl fmt::Display for HandleCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not get content")
    }
}

impl Error for HandleCommandError {}

#[derive(Debug, Clone)]
struct GetLockError;

impl fmt::Display for GetLockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not get lock")
    }
}

impl Error for GetLockError {}

struct Accounts;

impl TypeMapKey for Accounts {
    type Value = Arc<RwLock<HashMap<UserId, HashMap<UserId, i64>>>>;
}

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
                Err(_e) => "Error handling command".to_string(),
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
                                .name("amount")
                                .description("Amount in cents")
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
                        .create_option(|option| {
                            option
                                .name("user")
                                .description("User")
                                .kind(ApplicationCommandOptionType::User)
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
                                .description("Amount in cents")
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

        println!("Loaded {} commands.", commands.unwrap_or(Vec::new()).len());
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
    let signals = match Signals::new(&[SIGTERM, SIGINT]) {
        Ok(signals) => signals,
        Err(_e) => std::process::exit(1),
    };
    tokio::spawn(handle_signals(signals));

    let token = std::env::var("DISCORD_TOKEN").expect("token");

    let application_id: u64 = std::env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Accounts>(Arc::new(RwLock::new(HashMap::new())));
    }

    if let Err(e) = client.start().await {
        println!("An error occurred while running the client: {:?}", e);
    }
}

async fn balance_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<String, HandleCommandError> {
    let accounts = match get_accounts_lock(&ctx).await {
        Ok(accounts_lock) => accounts_lock,
        Err(_e) => return Err(HandleCommandError),
    };

    let mut response = format!("{}'s balance:\n", command.user.tag());

    let accounts_read = accounts.read().await;
    if let Some(account) = accounts_read.get(&command.user.id) {
        for (id, &balance) in account {
            if let Ok(user) = id.to_user(ctx).await {
                if let Err(_e) = write!(response, "{:<32}{}\n", user.tag(), format_money(balance)) {
                    return Err(HandleCommandError);
                }
            }
        }
    }

    Ok(response)
}

async fn owe_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<String, HandleCommandError> {
    let mut amount = None;
    let mut description = None;
    let mut user_opt = None;

    for option in &command.data.options {
        match option.name.as_ref() {
            "amount" => match &option.resolved {
                Some(ApplicationCommandInteractionDataOptionValue::Integer(value)) => {
                    amount = Some(value);
                }
                _ => return Err(HandleCommandError),
            },
            "description" => match &option.resolved {
                Some(ApplicationCommandInteractionDataOptionValue::String(value)) => {
                    description = Some(value);
                }
                _ => return Err(HandleCommandError),
            },
            "user" => match &option.resolved {
                Some(ApplicationCommandInteractionDataOptionValue::User(user, _member)) => {
                    user_opt = Some(user);
                }
                _ => return Err(HandleCommandError),
            },
            _ => return Err(HandleCommandError),
        }
    }

    if let Some(&amount) = amount {
        if let Some(description) = description {
            if let Some(receiver) = user_opt {
                let accounts = match get_accounts_lock(&ctx).await {
                    Ok(accounts_lock) => accounts_lock,
                    Err(_e) => return Err(HandleCommandError),
                };

                {
                    let mut accounts = accounts.write().await;
                    {
                        let receiver_entry = accounts.entry(receiver.id).or_insert(HashMap::new());
                        *receiver_entry.entry(command.user.id).or_insert(0) += amount;
                    }
                    {
                        let sender_entry =
                            accounts.entry(command.user.id).or_insert(HashMap::new());
                        *sender_entry.entry(receiver.id).or_insert(0) -= amount;
                    }
                }

                return Ok(format!(
                    "{} owes {} to {} for {}",
                    command.user.tag(),
                    format_money(amount),
                    receiver.tag(),
                    description
                ));
            }
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
                Some(ApplicationCommandInteractionDataOptionValue::Integer(value)) => {
                    amount = Some(value);
                }
                _ => return Err(HandleCommandError),
            },
            "description" => match &option.resolved {
                Some(ApplicationCommandInteractionDataOptionValue::String(value)) => {
                    description = Some(value);
                }
                _ => return Err(HandleCommandError),
            },
            _ => match &option.resolved {
                Some(ApplicationCommandInteractionDataOptionValue::User(user, _member)) => {
                    users.push(user);
                }
                _ => return Err(HandleCommandError),
            },
        }
    }

    if let Some(&amount) = amount {
        if let Some(description) = description {
            let accounts = match get_accounts_lock(&ctx).await {
                Ok(accounts_lock) => accounts_lock,
                Err(_e) => return Err(HandleCommandError),
            };

            {
                let mut accounts = accounts.write().await;
                for receiver in &users {
                    let receiver_entry = accounts.entry(receiver.id).or_insert(HashMap::new());
                    *receiver_entry.entry(command.user.id).or_insert(0) -= amount;

                    let sender_entry = accounts.entry(command.user.id).or_insert(HashMap::new());
                    *sender_entry.entry(receiver.id).or_insert(0) += amount;
                }
            }

            return Ok(format!(
                "{} billed {} to {} users for {}",
                command.user.tag(),
                format_money(amount),
                users.len(),
                description
            ));
        }
    }

    Err(HandleCommandError)
}

async fn get_accounts_lock(
    ctx: &Context,
) -> Result<Arc<RwLock<HashMap<UserId, HashMap<UserId, i64>>>>, GetLockError> {
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
        string = format!("${:0>3}", money);
    } else {
        string = format!("-${:0>3}", -money);
    }
    string.insert(string.len() - 2, '.');
    string
}
