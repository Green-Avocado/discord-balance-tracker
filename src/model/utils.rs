use super::accounts::{Accounts, AccountsType};

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

    if input.starts_with('-') {
        negative = true;
        input = &((*input)[1..]);
    }

    if input.starts_with('$') {
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
        if next.len() != 2 {
            return Err(ParseMoneyError);
        }

        match next.parse::<u32>() {
            Ok(cents) => money += cents,
            Err(_e) => return Err(ParseMoneyError),
        };
    }

    if let Some(_next) = split.next() {
        return Err(ParseMoneyError);
    }

    if negative {
        Ok(-(i64::from(money)))
    } else {
        Ok(money.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_money_zero() {
        assert_eq!("$0.00", format_money(0));
    }

    #[test]
    fn test_format_money_positive() {
        assert_eq!("$0.01", format_money(1));
        assert_eq!("$0.12", format_money(12));
        assert_eq!("$1.23", format_money(123));
        assert_eq!("$12.34", format_money(1234));
    }

    #[test]
    fn test_format_money_negative() {
        assert_eq!("-$0.01", format_money(-1));
        assert_eq!("-$0.12", format_money(-12));
        assert_eq!("-$1.23", format_money(-123));
        assert_eq!("-$12.34", format_money(-1234));
    }

    #[test]
    fn test_parse_money_zero() -> Result<(), String> {
        let expected = 0;
        match parse_money("0") {
            Ok(actual) => {
                if expected != actual {
                    return Err(format!("Expected {}, got {}", expected, actual));
                }
            }
            Err(e) => return Err(e.to_string()),
        }

        Ok(())
    }

    #[test]
    fn test_parse_money_positive() -> Result<(), String> {
        let expected = 1;
        match parse_money("0.01") {
            Ok(actual) => {
                if expected != actual {
                    return Err(format!("Expected {}, got {}", expected, actual));
                }
            }
            Err(e) => return Err(e.to_string()),
        }

        let expected = 1200;
        match parse_money("12") {
            Ok(actual) => {
                if expected != actual {
                    return Err(format!("Expected {}, got {}", expected, actual));
                }
            }
            Err(e) => return Err(e.to_string()),
        }

        let expected = 1234;
        match parse_money("$12.34") {
            Ok(actual) => {
                if expected != actual {
                    return Err(format!("Expected {}, got {}", expected, actual));
                }
            }
            Err(e) => return Err(e.to_string()),
        }

        Ok(())
    }

    #[test]
    fn test_parse_money_negative() -> Result<(), String> {
        let expected = -1;
        match parse_money("-0.01") {
            Ok(actual) => {
                if expected != actual {
                    return Err(format!("Expected {}, got {}", expected, actual));
                }
            }
            Err(e) => return Err(e.to_string()),
        }

        let expected = -1200;
        match parse_money("-12") {
            Ok(actual) => {
                if expected != actual {
                    return Err(format!("Expected {}, got {}", expected, actual));
                }
            }
            Err(e) => return Err(e.to_string()),
        }

        let expected = -1234;
        match parse_money("-$12.34") {
            Ok(actual) => {
                if expected != actual {
                    return Err(format!("Expected {}, got {}", expected, actual));
                }
            }
            Err(e) => return Err(e.to_string()),
        }

        Ok(())
    }

    #[test]
    fn test_parse_money_error() -> Result<(), String> {
        match parse_money("a") {
            Ok(actual) => return Err(format!("Expected error, got {}", actual)),
            Err(_e) => {}
        }

        match parse_money("-0.0.1") {
            Ok(actual) => return Err(format!("Expected error, got {}", actual)),
            Err(_e) => {}
        }

        match parse_money("-0.-1") {
            Ok(actual) => return Err(format!("Expected error, got {}", actual)),
            Err(_e) => {}
        }

        Ok(())
    }
}
