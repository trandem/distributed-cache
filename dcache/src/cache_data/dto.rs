use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct GetCacheByListKeyRequest {
    pub keys : Vec<i32>,
}