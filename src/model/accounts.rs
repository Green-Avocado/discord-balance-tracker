use serenity::model::id::UserId;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use typemap_rev::TypeMapKey;

use std::{collections::HashMap, sync::Arc};

pub type AccountsType = Arc<RwLock<HashMap<UserId, HashMap<UserId, i64>>>>;

pub struct Accounts;

impl TypeMapKey for Accounts {
    type Value = AccountsType;
}
