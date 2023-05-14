mod cache_data;

use std::sync::Arc;
use cache_data::data_cache::GlobalCache;
use actix_web::{get, post, web, Responder, HttpResponse};
use log::info;
use crate::cache_data::dto::{GetCacheByListKeyRequest, KeyValue};
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
    let cache_data = cache_manager.global_cache.find_by_key(key).await;
    if cache_data.is_none() {
        return None;
    }
    let result = cache_data.unwrap().clone().await;
    Some(result.unwrap().clone())
}

#[post("/get_caches")]
pub async fn get_cache_by_list_key(
    cache_manager: Data<Arc<CacheManager>>,
    body: web::Json<GetCacheByListKeyRequest>,
) -> impl Responder {
    let keys = body.into_inner().keys;
    let output = cache_manager.global_cache.find_by_keys(&keys).await;

    let mut result = Vec::with_capacity(output.len());
    for op in output.iter() {
        if op.is_some() {
            let data = op.clone().unwrap().await.unwrap().clone();
            result.push(data);
        }
    }

    let json_response = serde_json::json!({
        "result" : result,
    });
    HttpResponse::Ok().json(json_response)
}

#[get("/invalid_cache/{key}")]
pub async fn invalid_cache(
    mut cache_manager: web::Data<Arc<CacheManager>>,
    path: web::Path<(i32, )>,
) -> Option<String> {
    let key = path.into_inner().0;
    let cache_data = cache_manager.global_cache.invalid(key).await;
    if cache_data.is_none() {
        return None;
    }
    Some("success".to_string())
}

