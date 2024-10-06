/* use std::{fs::File, io::BufReader}; */

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

    /* rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut cert_file = BufReader::new(File::open("cert.pem").unwrap());
    let mut key_file = BufReader::new(File::open("key.pem").unwrap());

    let tls_cert = rustls_pemfile::certs(&mut cert_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_cert, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap(); */

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