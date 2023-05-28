use dcache::init;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init().await
}
