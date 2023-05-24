use std::sync::Arc;

use actix_web::{get, HttpResponse, post, Responder, web};
use actix_web::web::Data;
use log::info;
use sqlx::{MySql, Pool};

use cache_data::data_cache::GlobalCache;

use crate::cache_data::dto::{ERROR_DATA, GetCacheByListKeyRequest, KeyValue, MySqlDataRepo, UserData};

mod cache_data;

#[derive(Clone)]
pub struct CacheManager {
    global_cache: Arc<GlobalCache>,
}

impl CacheManager {
    pub fn new(num_shard: usize, shard_max_capacity: usize, sqlx: Pool<MySql>) -> CacheManager {
        let repo = MySqlDataRepo { sql_pool: sqlx };
        let global_cache = Arc::new(
            GlobalCache::new(num_shard, shard_max_capacity, repo)
        );
        CacheManager { global_cache }
    }
    pub fn get_cache(&self) -> Arc<GlobalCache>{
        self.global_cache.clone()
    }
}

#[get("/ping")]
pub async fn ping() -> String {
    info!("ping");
    "pong".to_string()
}

#[get("/get_cache/{key}")]
pub async fn get_cache(
    cache_manager: Data<Arc<CacheManager>>,
    path: web::Path<(i32, )>,
) -> impl Responder {
    let key: i32 = path.into_inner().0;
    let value = get_value_by_key(cache_manager.clone(), key).await;

    let mut key_value: KeyValue;
    match value {
        Some(val) => {
            key_value = KeyValue {
                key,
                value: Some(val),
            }
        }
        None => {
            key_value = KeyValue {
                key,
                value: None,
            }
        }
    }

    let json_response = serde_json::json!({
        "result" : key_value,
    });
    HttpResponse::Ok().json(json_response)
}

async fn get_value_by_key(cache_manager: Data<Arc<CacheManager>>, key: i32) -> Option<UserData> {
    let cache_data = cache_manager.global_cache.find_by_key(key).await;
    if cache_data.is_none() {
        return None;
    }
    let result = cache_data.unwrap().clone().await;
    let data = result.unwrap().clone();
    if data ==ERROR_DATA {
        info!("error when query do in valid {}",key);
        cache_manager.global_cache.invalid(key).await;
    }
    Some(data)
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

