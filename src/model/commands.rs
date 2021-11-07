pub mod balance;
pub mod bill;
pub mod owe;

use serenity::model::prelude::User;

use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct HandleCommandError;

impl fmt::Display for HandleCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
}

pub struct BillTransaction {
    initiator: User,
    amount: i64,
    recipients: Vec<User>,
}

pub enum TransactionType {
    Owe(OweTransaction),
    Bill(BillTransaction),
    None,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
