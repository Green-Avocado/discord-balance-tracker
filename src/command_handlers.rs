use serenity::{
    client::Context,
    model::{
        id::UserId,
        interactions::application_command::{
            ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
        },
    },
};

use tokio::sync::RwLock;
use typemap_rev::TypeMapKey;

use std::{collections::HashMap, fmt::Write, sync::Arc};

use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct HandleCommandError;

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

#[derive(Debug, Clone)]
struct ParseMoneyError;

impl fmt::Display for ParseMoneyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not parse money")
    }
}

impl Error for ParseMoneyError {}

pub struct Accounts;

impl TypeMapKey for Accounts {
    type Value = Arc<RwLock<HashMap<UserId, HashMap<UserId, i64>>>>;
}

pub async fn balance_handler(
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

pub async fn owe_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<String, HandleCommandError> {
    let mut amount = None;
    let mut description = None;
    let mut user_opt = None;

    for option in &command.data.options {
        match option.name.as_ref() {
            "amount" => match &option.resolved {
                Some(ApplicationCommandInteractionDataOptionValue::String(value)) => {
                    amount = Some(parse_money(value));
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

    if let Some(Ok(amount)) = amount {
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

pub async fn bill_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<String, HandleCommandError> {
    let mut amount = None;
    let mut description = None;
    let mut users = Vec::new();

    for option in &command.data.options {
        match option.name.as_ref() {
            "amount" => match &option.resolved {
                Some(ApplicationCommandInteractionDataOptionValue::String(value)) => {
                    amount = Some(parse_money(value));
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

    if let Some(Ok(amount)) = amount {
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
