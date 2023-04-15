mod cache_data;

use std::sync::Arc;
use cache_data::data_cache::GlobalCache;
use actix_web::{get};
use lazy_static::lazy_static;
use log::{error, info, warn};

lazy_static! {
    static ref GLOBAL_CACHE : Arc<GlobalCache> = {
        let mut global = GlobalCache::new(10,10);
        Arc::new(global)
    };
}

#[get("/ping")]
pub async fn ping() -> String {
    info!("ping");
    "pong".to_string()
}

#[get("/get_cache")]
pub async fn get_cache<'a>() -> Option<&'a str> {
    let mut global = GlobalCache::new(10,10);
    let x = global.get(Arc::new(1)).await;
    if x.is_none() {
        return None;
    }
    println!("{:?}",x);
    Some("lol")
}

