use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySql, Pool};

#[derive(Deserialize, Debug)]
pub struct GetCacheByListKeyRequest {
    pub keys : Vec<i32>,
}

#[derive(Serialize, Debug)]
pub struct KeyValue {
    pub key : i32,
    pub value : Option<UserData>,
}

pub struct DataRepo {
    pub sql_pool: Pool<MySql>,
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct UserData {
    pub id : i32,
    pub name : String,
}