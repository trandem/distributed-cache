use std::{env, thread};
use std::sync::Arc;
use std::time::Duration;

use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use futures::{future, StreamExt};
use log4rs;
use log::info;
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{BaseConsumer, Consumer, StreamConsumer};
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::types::Uuid;
use sqlx::types::uuid::uuid;

use dcache::{CacheManager, get_cache, get_cache_by_list_key, ping};
use crate::cache_data::dto::InvalidCache;

mod cache_data;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("booting up");
    dotenv().ok();

    let mysql_url: String = env::var("mysql.url").unwrap().parse().unwrap();
    let sql_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&mysql_url).await.unwrap();

    let num_shard: usize = env::var("cache.shard.num").unwrap().parse().unwrap();
    let shard_size: usize = env::var("cache.shard.max_capacity").unwrap().parse().unwrap();
    let cache_manager = Arc::new(CacheManager::new(num_shard, shard_size, sql_pool));
    let manager1 = cache_manager.clone();


    let mut group_id = "d_cache_group_id".to_string();
    group_id.push_str(Uuid::new_v4().to_string().as_str());
    info!("group_id {}",group_id);
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("auto.offset.reset", "beginning")
        .set("group.id", group_id)
        .create()
        .expect("invalid consumer config");
    consumer.subscribe(&["invalid_topic"]).expect("TODO: panic message");

    let manager2 = cache_manager.clone();
    tokio::spawn(async move {
            while let Some(message) = consumer.stream().next().await {
            let rs = message.unwrap();
            let value = rs.payload().unwrap();

            let invalid: InvalidCache = serde_json::from_slice(value).expect("failed to deser JSON to User");
            info!("value from kafka {:?}", invalid);
            if invalid.id.is_some() {
                let id = invalid.id.unwrap();
                manager2.get_cache().invalid(id).await;
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .service(ping)
            .service(get_cache)
            .service(get_cache_by_list_key)
            .app_data(web::Data::new(manager1.clone()))
    })
        .bind(("0.0.0.0", 9111))?
        .run()
        .await?;

    Ok(())
}
