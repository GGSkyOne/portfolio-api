use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml}};

#[derive(Deserialize)]
pub struct BaseUrlConfig {
    pub weather_api: String,
    pub spotify_api: String,
    pub spotify_accounts: String,
    pub spotify_accounts_api: String
}

#[derive(Deserialize)]
pub struct WeatherConfig {
    pub key: String,
    pub city: String
}

#[derive(Deserialize)]
pub struct RedisConfig {
    pub host: String
}

#[derive(Deserialize)]
pub struct Config {
    pub base_url: BaseUrlConfig,
    pub weather: WeatherConfig,
    pub redis: RedisConfig
}

pub fn get_config() -> Result<Config, figment::Error> {
    let config: Config = Figment::new()
    .merge(Toml::file("config.toml"))
    .extract()?;

    Ok(config)
}