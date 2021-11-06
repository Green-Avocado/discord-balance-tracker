use crate::model::accounts::{Accounts, AccountsType};

use serde::{Serialize, Serializer};
use serde_json;
use serenity::client::Context;

use std::fs::File;

pub async fn write_accounts_file(ctx: &Context) {
    let accounts_lock = {
        let data_read = ctx.data.read().await;
        match data_read.get::<Accounts>() {
            Some(data) => data.clone(),
            None => panic!("Could not get lock"),
        }
    };

    let accounts = accounts_lock.read().await;
    let file = File::create("foo.txt").unwrap();
    serde_json::to_writer_pretty(file, &*accounts).unwrap();
}

pub async fn read_accounts_file() {}
