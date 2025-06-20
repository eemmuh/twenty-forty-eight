use twenty_forty_eight::web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    web::start_server().await
} 