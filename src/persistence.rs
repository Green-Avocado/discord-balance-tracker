use crate::model::accounts::{Accounts, AccountsType};

use serde::{Serialize, Serializer};
use serde_json;
use serenity::{client::Context, prelude::TypeMap};

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use typemap_rev::TypeMapKey;

use std::{collections::HashMap, fs::File, sync::Arc};

pub async fn write_accounts_file(data: Arc<RwLock<TypeMap>>) {
    let accounts_lock = {
        let data_read = data.read().await;
        match data_read.get::<Accounts>() {
            Some(data) => data.clone(),
            None => panic!("Could not get lock"),
        }
    };

    let accounts = accounts_lock.read().await;
    let file = File::create("balances.json").unwrap();
    serde_json::to_writer_pretty(file, &*accounts).unwrap();
}

pub async fn read_accounts_file(data: &Arc<RwLock<TypeMap>>) {
    let accounts_lock = {
        let data_read = data.read().await;
        match data_read.get::<Accounts>() {
            Some(data) => data.clone(),
            None => panic!("Could not get lock"),
        }
    };

    let accounts = accounts_lock.write().await;
}
