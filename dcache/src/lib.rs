use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use actix_web::web::Data;
use dotenv::dotenv;
use log::info;
use rdkafka::ClientConfig;
use rdkafka::consumer::StreamConsumer;
use sqlx::{MySql, Pool};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::types::Uuid;
use crate::cache_data::data_cache::GlobalCache;

use crate::cache_data::dto::{ ERROR_DATA, GetCacheByListKeyRequest, KeyValue, UserData};
use crate::cache_data::mq_consumer::{MqConsumer, MqConsumerFunc};
use crate::cache_data::repo::MySqlDataRepo;

mod cache_data;

#[derive(Clone)]
pub struct CacheManager {
    pub global_cache: Arc<GlobalCache>,
}

impl CacheManager {
    pub fn new(global_cache: Arc<GlobalCache>) -> CacheManager {

        CacheManager { global_cache }
    }
    pub fn get_cache(&self) -> Arc<GlobalCache> {
        self.global_cache.clone()
    }
}

pub async fn init() -> std::io::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("booting up");
    dotenv().ok();

    let mysql_url: String = env::var("mysql.url").unwrap().parse().unwrap();
    let sql_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(1))
        .connect(&mysql_url)
        .await.unwrap();

    let num_shard: usize = env::var("cache.shard.num").unwrap().parse().unwrap();
    let shard_size: usize = env::var("cache.shard.max_capacity").unwrap().parse().unwrap();

    let repo = MySqlDataRepo { sql_pool };
    let global_cache:Arc<GlobalCache> = Arc::new(
        GlobalCache::new(num_shard, shard_size, repo)
    );

    let cache_manager = Arc::new(CacheManager::new(global_cache.clone()));

    let mut group_id = "d_cache_group_id".to_string();
    group_id.push_str(Uuid::new_v4().to_string().as_str());
    info!("group_id {}",group_id);
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("auto.offset.reset", "beginning")
        .set("group.id", group_id)
        .create()
        .expect("invalid consumer config");

    let mq_consumer = MqConsumer::new(consumer, global_cache.clone());

    mq_consumer.invalid_by_signal("invalid_topic");

    HttpServer::new(move || {
        App::new()
            .service(ping)
            .service(get_cache)
            .service(get_cache_by_list_key)
            .app_data(web::Data::new(cache_manager.clone()))
    })
        .bind(("0.0.0.0", 9111))?
        .run()
        .await?;

    Ok(())
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

    let mut key_value: KeyValue = match value {
        Some(val) => {
            KeyValue {
                key,
                value: Some(val),
            }
        }
        None => {
            KeyValue {
                key,
                value: None,
            }
        }
    };

    let json_response = serde_json::json!({
        "result" : key_value,
    });
    HttpResponse::Ok().json(json_response)
}

async fn get_value_by_key(cache_manager: Data<Arc<CacheManager>>, key: i32) -> Option<UserData> {
    let cache_data = cache_manager.global_cache.find_by_key(key).await;
    cache_data
}

#[post("/get_caches")]
pub async fn get_cache_by_list_key(
    cache_manager: Data<Arc<CacheManager>>,
    body: web::Json<GetCacheByListKeyRequest>,
) -> impl Responder {
    let keys = body.into_inner().keys;
    let output = cache_manager.global_cache.find_by_keys(&keys).await;

    let mut result = Vec::from_iter(output);

    let json_response = serde_json::json!({
        "result" : result,
    });

    HttpResponse::Ok().json(json_response)
}

