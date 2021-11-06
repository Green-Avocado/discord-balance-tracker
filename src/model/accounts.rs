use serenity::model::id::UserId;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use typemap_rev::TypeMapKey;

use std::{collections::HashMap, sync::Arc};

type AccountsHashMap = HashMap<UserId, HashMap<UserId, i64>>;

#[derive(Debug, Clone)]
pub struct AccountsType(Arc<RwLock<AccountsHashMap>>);

impl AccountsType {
    pub fn new() -> AccountsType {
        let x = Arc::new(RwLock::new(AccountsHashMap::new()));
        AccountsType(x)
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, AccountsHashMap> {
        self.0.write().await
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, AccountsHashMap> {
        self.0.read().await
    }
}

pub struct Accounts;

impl TypeMapKey for Accounts {
    type Value = AccountsType;
}
