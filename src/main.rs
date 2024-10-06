use actix_web::{middleware, web, App, HttpServer};
use connectors::redis_connector::connect;
use modules::{projects::projects_config, spotify::spotify_config, weather::weather_config};
use config::Config;

pub mod config;
pub mod modules;
pub mod connectors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    connect().await;
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api/v1")
                    .wrap(middleware::Compress::default())
                    .configure(weather_config)
                    .configure(projects_config)
                    .configure(spotify_config)
            )

    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

pub fn config() -> Config {
    return config::get_config().expect("Failed to load configuration");
}