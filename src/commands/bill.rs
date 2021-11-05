use super::HandleCommandError;

use crate::utils::*;

use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
        ApplicationCommandOptionType,
    },
};

use std::collections::HashMap;

pub fn bill_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let mut command = command
        .name("bill")
        .description("Bill user(s) for transaction")
        .create_option(|option| {
            option
                .name("amount")
                .description("Amount in dollars")
                .kind(ApplicationCommandOptionType::String)
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
