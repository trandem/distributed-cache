mod cache_data;

use std::cell::RefCell;
use std::sync::Arc;
use cache_data::data_cache::GlobalCache;
use actix_web::{get, web};
use lazy_static::lazy_static;
use log::{error, info, warn};

#[derive(Clone)]
pub struct CacheManager {
    global_cache: Arc<GlobalCache>,
}

impl CacheManager {
    pub fn new(num_shard: usize, shard_max_capacity: usize) -> CacheManager {
        let global_cache = Arc::new(
            GlobalCache::new(num_shard, shard_max_capacity)
        );
        CacheManager { global_cache }
    }
}

#[get("/ping")]
pub async fn ping() -> String {
    info!("ping");
    "pong".to_string()
}

#[get("/get_cache")]
pub async fn get_cache<>(mut cache_manager: web::Data<Arc<CacheManager>>) -> Option<String> {
    let x = cache_manager.global_cache.get(1).await;
    if x.is_none() {
        return None;
    }
    let x = x.unwrap().clone();
    let y = x.as_ref().clone();
    println!("{:?}", y);
    Some(y)
}
#[get("/invalid_cache")]
pub async fn invalid_cache<>(mut cache_manager: web::Data<Arc<CacheManager>>) -> Option<String> {
    let x = cache_manager.global_cache.invalid(1).await;
    if x.is_none() {
        return None;
    }
    let x = x.unwrap().clone();
    let y = x.as_ref().clone();
    println!("{:?}", y);
    Some(y)
}

