mod cache_data;

use std::{env};
use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use log::info;
use log4rs;
use dcache::{CacheManager, get_cache, invalid_cache, get_cache_by_list_key, ping};
use futures::future;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("booting up");
    dotenv().ok();

    let host: String = env::var("sql.jdbc.url").unwrap().parse().unwrap();
    let db: String = env::var("sql.db").unwrap().parse().unwrap();
    let user_name: String = env::var("sql.user").unwrap().parse().unwrap();
    let pass_word: String = env::var("sql.pass").unwrap().parse().unwrap();
    let opts = MySqlConnectOptions::new()
        .host(host.as_str())
        .username(user_name.as_str())
        .password(pass_word.as_str());

    let sql_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect_with(opts).await.unwrap();

    let num_shard: usize = env::var("cache.shard.num").unwrap().parse().unwrap();
    let shard_size: usize = env::var("cache.shard.max_capacity").unwrap().parse().unwrap();
    let cache_manager = Arc::new(CacheManager::new(num_shard, shard_size, sql_pool));
    let manager1 = cache_manager.clone();


    HttpServer::new(move || {
        App::new()
            .service(ping)
            .service(get_cache)
            .service(invalid_cache)
            .service(get_cache_by_list_key)
            .app_data(web::Data::new(manager1.clone()))
    })
        .bind(("0.0.0.0", 9111))?
        .run()
        .await?;

    Ok(())
}
