pub mod balance;
pub mod bill;
pub mod owe;

use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct HandleCommandError;

impl fmt::Display for HandleCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not get content")
    }
}

impl Error for HandleCommandError {}
