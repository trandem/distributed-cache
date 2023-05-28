use log::error;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use crate::cache_data::data_cache::GlobalCache;
use crate::cache_data::repo::MySqlDataRepo;

#[derive(Deserialize, Debug)]
pub struct GetCacheByListKeyRequest {
    pub keys: Vec<i32>,
}

#[derive(Serialize, Debug)]
pub struct KeyValue {
    pub key: i32,
    pub value: Option<UserData>,
}

pub const EMPTY_DATA: UserData = UserData {
    id: None,
    name: None,
};

pub const ERROR_DATA: UserData = UserData {
    id: Some(-1),
    name: None,
};

#[derive(Serialize, Debug, Clone, sqlx::FromRow,PartialEq)]
pub struct UserData {
    pub id: Option<i32>,
    pub name: Option<String>,
}

#[derive(Serialize,Deserialize, Debug, Clone,PartialEq)]
pub struct InvalidCache {
    pub id: Option<i32>
}