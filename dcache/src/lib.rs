mod cache_data;

use std::cell::RefCell;
use std::sync::Arc;
use cache_data::data_cache::GlobalCache;
use actix_web::{get, post, web, Responder, HttpResponse};
use lazy_static::lazy_static;
use log::{error, info, warn};
use crate::cache_data::dto::GetCacheByListKeyRequest;
use actix_web::web::Data;

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

#[get("/get_cache/{key}")]
pub async fn get_cache(
    cache_manager: web::Data<Arc<CacheManager>>,
    path: web::Path<(i32, )>,
) -> Option<String> {
    let key: i32 = path.into_inner().0;
    get_value_by_key(cache_manager.clone(), key).await
}

async fn get_value_by_key(cache_manager: Data<Arc<CacheManager>>, key: i32) -> Option<String> {
    let x = cache_manager.global_cache.get(key).await;
    if x.is_none() {
        return None;
    }
    let x = x.unwrap().clone();
    let y = x.as_ref().clone();
    println!("{:?}", y);
    Some(y)
}

#[post("/get_caches/")]
pub async fn get_cache_by_list_key(
    mut cache_manager: web::Data<Arc<CacheManager>>,
    body: web::Json<GetCacheByListKeyRequest>,
) -> impl Responder {
    let keys = body.into_inner().keys;

    let mut values: Vec<String> = Vec::new();
    for key in keys {
        let value = self::get_value_by_key(cache_manager.clone(), key).await;
        if value.is_some() {
            values.push(value.unwrap());
        }
    }

    let json_response = serde_json::json!({
        "values" : values,
    });
    HttpResponse::Ok().json(json_response)
}

#[get("/invalid_cache/{key}")]
pub async fn invalid_cache(
    mut cache_manager: web::Data<Arc<CacheManager>>,
    path: web::Path<(i32, )>,
) -> Option<String> {
    let key = path.into_inner().0;
    let x = cache_manager.global_cache.invalid(key).await;
    if x.is_none() {
        return None;
    }
    let x = x.unwrap().clone();
    let y = x.as_ref().clone();
    println!("{:?}", y);
    Some(y)
}

