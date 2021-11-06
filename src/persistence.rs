use crate::model::accounts::{Accounts, AccountsType};

use serde_json;
use serenity::prelude::TypeMap;
use tokio::sync::RwLock;

use std::{fs::create_dir, fs::File, sync::Arc};

const DATA_FILE: &str = "data/balances.json";

pub async fn write_accounts_file(data: Arc<RwLock<TypeMap>>) {
    let lock = get_lock(data).await;
    let accounts = lock.read().await;

    if let Err(_e) = create_dir("data") {
        return;
    }

    if let Ok(file) = File::create(DATA_FILE) {
        serde_json::to_writer_pretty(file, &*accounts).unwrap();
    }
}

pub async fn read_accounts_file(data: Arc<RwLock<TypeMap>>) {
    let lock = get_lock(data).await;
    let mut accounts = lock.write().await;

    if let Ok(file) = File::open(DATA_FILE) {
        *accounts = serde_json::from_reader(file).unwrap();
    }
}

async fn get_lock(data: Arc<RwLock<TypeMap>>) -> AccountsType {
    let accounts_lock = {
        let data_read = data.read().await;
        match data_read.get::<Accounts>() {
            Some(data) => data.clone(),
            None => panic!("Could not get lock"),
        }
    };

    accounts_lock
}
