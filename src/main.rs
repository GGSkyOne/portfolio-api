use actix_web::{middleware, web, App, HttpServer};
use client::create_client;
use config::load_config;
use connectors::redis_connector::connect_to_redis;
use modules::{projects::projects_config, spotify::spotify_config, weather::weather_config};

pub mod client;
pub mod config;
pub mod modules;
pub mod connectors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    load_config();
    create_client();
    connect_to_redis().await;

    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api/v2")
                    .wrap(middleware::Compress::default())
                    .configure(weather_config)
                    .configure(projects_config)
                    .configure(spotify_config)
            )

    })
    .bind(("127.0.0.1", 3005))?
    .run()
    .await
}