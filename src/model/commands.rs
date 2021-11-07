pub mod balance;
pub mod bill;
pub mod owe;

use super::utils::format_money;

use serenity::model::prelude::User;

use std::{
    error::Error,
    fmt::{Display, Formatter, Result, Write},
};

#[derive(Debug, Clone)]
pub struct HandleCommandError;

impl Display for HandleCommandError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "could not get content")
    }
}

impl Error for HandleCommandError {}

pub struct CommandResult {
    pub response: String,
    pub transaction: TransactionType,
}

pub struct OweTransaction {
    initiator: User,
    amount: i64,
    recipient: User,
    description: String,
}

impl Display for OweTransaction {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{} owes {} to {} for \"{}\"",
            self.initiator.tag(),
            format_money(self.amount),
            self.recipient.tag(),
            self.description
        )
    }
}

pub struct BillTransaction {
    initiator: User,
    amount: i64,
    recipients: Vec<User>,
    description: String,
}

impl Display for BillTransaction {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut recipient_list = String::new();

        for recipient in &self.recipients {
            write!(recipient_list, " {}", recipient.tag()).unwrap();
        }

        write!(
            f,
            "{} billed {} to{} for \"{}\"",
            self.initiator.tag(),
            format_money(self.amount),
            recipient_list,
            self.description
        )
    }
}

pub enum TransactionType {
    Owe(OweTransaction),
    Bill(BillTransaction),
    None,
}

impl Display for TransactionType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            TransactionType::Owe(x) => write!(f, "{}", x),
            TransactionType::Bill(x) => write!(f, "{}", x),
            TransactionType::None => unreachable!(),
        }
    }
}
