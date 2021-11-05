use super::HandleCommandError;

use crate::accounts::AccountsType;
use crate::utils::*;

use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

use std::fmt::Write;

pub async fn balance_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<String, HandleCommandError> {
    let accounts: AccountsType = match get_accounts_lock(&ctx).await {
        Ok(accounts_lock) => accounts_lock,
        Err(_e) => return Err(HandleCommandError),
    };

    let mut response = format!("{}'s balance:\n", command.user.tag());

    let accounts_read = accounts.read().await;
    if let Some(account) = accounts_read.get(&command.user.id) {
        for (id, &balance) in account {
            if let Ok(user) = id.to_user(ctx).await {
                if let Err(_e) = write!(
                    response,
                    "`{:<32}{:>16}`\n",
                    user.tag(),
                    format_money(balance)
                ) {
                    return Err(HandleCommandError);
                }
            }
        }
    }

    Ok(response)
}
