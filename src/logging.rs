use crate::model::commands::TransactionType;

use serenity::prelude::TypeMap;
use tokio::sync::RwLock;
use typemap_rev::TypeMapKey;

use std::io::Write;
use std::{fs::File, sync::Arc};

pub async fn log(data: Arc<RwLock<TypeMap>>, transaction: TransactionType) {
    if let TransactionType::None = transaction {
        return;
    }

    let log_lock = {
        let data_read = data.read().await;
        match data_read.get::<Log>() {
            Some(data) => data.clone(),
            None => return,
        }
    };

    let mut log_file = log_lock.write().await;

    writeln!(log_file, "{}", transaction).unwrap();
}

pub struct Log;

impl TypeMapKey for Log {
    type Value = Arc<RwLock<File>>;
}
