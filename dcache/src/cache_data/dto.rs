use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct GetCacheByListKeyRequest {
    pub keys : Vec<i32>,
}

#[derive(Serialize, Debug)]
pub struct CachedValue {
    pub key : i32,
    pub value : String,
}