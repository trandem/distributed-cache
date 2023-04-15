use actix_web::{App, HttpServer};
use log::{error, info, warn};
use log4rs;
use dcache::{get_cache, ping};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    info!("booting up");
    HttpServer::new(|| {
        App::new()
            .service(ping)
            .service(get_cache)
    })
        .bind(("0.0.0.0", 9111))?
        .run()
        .await
}
