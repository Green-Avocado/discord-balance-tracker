use super::{CommandResult, HandleCommandError, TransactionType};

use super::super::accounts::AccountsType;
use super::super::utils::*;

use serenity::{
    builder::CreateApplicationCommand, client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
};

use std::fmt::Write;

pub fn balance_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("balance").description("Get balance")
}

pub async fn balance_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<CommandResult, HandleCommandError> {
    let accounts: AccountsType = match get_accounts_lock(ctx).await {
        Ok(accounts_lock) => accounts_lock,
        Err(_e) => return Err(HandleCommandError),
    };

    let mut response = format!("{}'s balance:\n", command.user.tag());

    let accounts_read = accounts.read().await;
    if let Some(account) = accounts_read.get(&command.user.id) {
        for (id, &balance) in account {
            if let Ok(user) = id.to_user(ctx).await {
                if let Err(_e) = writeln!(
                    response,
                    "`{:<32}{:>16}`",
                    user.tag(),
                    format_money(balance)
                ) {
                    return Err(HandleCommandError);
                }
            }
        }
    }

    Ok(CommandResult {
        response,
        transaction: TransactionType::None,
    })
}
