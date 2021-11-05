use super::HandleCommandError;

use crate::utils::*;

use serenity::{
    client::Context,
    model::interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
    },
};

use std::collections::HashMap;

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
