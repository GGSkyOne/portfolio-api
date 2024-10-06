use std::sync::Arc;

use actix_web::{middleware, web, App, HttpServer};
use connectors::redis_connector::connect;
use modules::{projects::projects_config, spotify::spotify_config, weather::weather_config};
use config::Config;

pub mod config;
pub mod modules;
pub mod connectors;

use lazy_static::lazy_static;
use reqwest::Client;
use rustls::lock::Mutex;

lazy_static! {
    static ref REQWEST_CLIENT: Arc<Mutex<Option<Client>>> = Arc::new(Mutex::new(None));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    create_reqwest_client();
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

pub fn create_reqwest_client() {
    let new_client: Client = reqwest::Client::new();
    let mut client = REQWEST_CLIENT.lock().unwrap();
    *client = Some(new_client);
}

pub fn get_reqwest_client() -> Client {
    let client = REQWEST_CLIENT.lock().unwrap();
    return client.as_ref().expect("Reqwest client failed").clone();
}