use std::sync::Arc;

use lazy_static::lazy_static;
use rustls::lock::Mutex;
use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml}};

lazy_static! {
    static ref CONFIG: Arc<Mutex<Option<Config>>> = Arc::new(Mutex::new(None));
}

#[derive(Deserialize, Clone)]
pub struct BaseUrlConfig {
    pub weather_api: String,
    pub spotify_api: String,
    pub spotify_accounts: String,
    pub spotify_accounts_api: String
}

#[derive(Deserialize, Clone)]
pub struct WeatherConfig {
    pub key: String,
    pub city: String
}

#[derive(Deserialize, Clone)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub secret: String,
    pub redirect_uri: String
}

#[derive(Deserialize, Clone)]
pub struct RedisConfig {
    pub host: String
}

#[derive(Deserialize)]
#[derive(Clone)]
pub struct Config {
    pub base_url: BaseUrlConfig,
    pub weather: WeatherConfig,
    pub spotify: SpotifyConfig,
    pub redis: RedisConfig
}

pub fn load_config() {
    let loaded_config: Config = Figment::new()
    .merge(Toml::file("config.toml"))
    .extract()
    .expect("Cannot load config.toml, is it exists?");

    println!("Loaded config successfully");

    let mut config = CONFIG.lock().unwrap();
    *config = Some(loaded_config);
}

pub fn get_config() -> Config {
    let config = CONFIG.lock().unwrap();
    return config.as_ref().expect("Failed to get config, is it loaded?").clone();
}