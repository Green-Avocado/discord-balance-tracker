use crate::accounts::*;

use serenity::client::Context;

use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct GetLockError;

impl fmt::Display for GetLockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not get lock")
    }
}

impl Error for GetLockError {}

#[derive(Debug, Clone)]
pub struct ParseMoneyError;

impl fmt::Display for ParseMoneyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not parse money")
    }
}

impl Error for ParseMoneyError {}

pub async fn get_accounts_lock(ctx: &Context) -> Result<AccountsType, GetLockError> {
    let accounts_lock = {
        let data_read = ctx.data.read().await;
        match data_read.get::<Accounts>() {
            Some(data) => data.clone(),
            None => return Err(GetLockError),
        }
    };

    Ok(accounts_lock)
}

pub fn format_money(money: i64) -> String {
    let mut string;
    if money >= 0 {
        string = format!("${:0>3}", money);
    } else {
        string = format!("-${:0>3}", -money);
    }
    string.insert(string.len() - 2, '.');
    string
}

pub fn parse_money(mut input: &str) -> Result<i64, ParseMoneyError> {
    let mut negative = false;

    if input.chars().nth(0).unwrap() == '-' {
        negative = true;
        input = &((*input)[1..]);
    }

    if input.chars().nth(0).unwrap() == '$' {
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
