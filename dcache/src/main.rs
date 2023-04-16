mod cache_data;

use std::{env};
use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use actix_web::dev::Server;
use dotenv::dotenv;
use log::{error, info, warn};
use log4rs;
use dcache::{CacheManager, get_cache, invalid_cache, get_cache_by_list_key, ping};
use cache_data::data_cache::GlobalCache;
use futures::future;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("booting up");
    dotenv().ok();
    let num_shard: usize = env::var("cache.shard.num").unwrap().parse().unwrap();
    let shard_size: usize = env::var("cache.shard.max_capacity").unwrap().parse().unwrap();
    let cache_manager = Arc::new(CacheManager::new(num_shard, shard_size));
    let manager1 = cache_manager.clone();
    let manager2 = cache_manager.clone();
    let s1 = HttpServer::new(move || {
        App::new()
            .service(ping)
            .service(get_cache)
            .service(invalid_cache)
            .service(get_cache_by_list_key)
            .app_data(web::Data::new(manager1.clone()))
    })
        .bind(("0.0.0.0", 9111))?
        .run();
    let s2 = HttpServer::new(move || {
        App::new()
            .service(ping)
            .service(get_cache)
            .service(invalid_cache)
            .service(get_cache_by_list_key)
            .app_data(web::Data::new(manager2.clone()))
    })
        .bind(("0.0.0.0", 9112))?
        .run();
    future::try_join(s1, s2).await?;

    Ok(())
}
