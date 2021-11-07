use super::{CommandResult, HandleCommandError, OweTransaction, TransactionType};

use super::super::utils::*;

use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
        ApplicationCommandOptionType,
    },
};

use std::collections::HashMap;

pub fn owe_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("owe")
        .description("Owe a user")
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
        })
        .create_option(|option| {
            option
                .name("user")
                .description("User to owe")
                .kind(ApplicationCommandOptionType::User)
                .required(true)
        })
}

pub async fn owe_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<CommandResult, HandleCommandError> {
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

                let response = format!(
                    "{} owes {} to {} for {}",
                    command.user.tag(),
                    format_money(amount),
                    receiver.tag(),
                    description
                );

                return Ok(CommandResult {
                    response,
                    transaction: TransactionType::Owe(OweTransaction {
                        initiator: command.user.clone(),
                        amount,
                        recipient: receiver.clone(),
                        description: description.to_string(),
                    }),
                });
            }
        }
    }

    Err(HandleCommandError)
}
