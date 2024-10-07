use actix_web::{web::{self, Json}, HttpResponse, Responder};
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};

use crate::{client::get_client, config::get_config, connectors::redis_connector::get_connection};

pub fn weather_config(config: &mut web::ServiceConfig) {
    config.service(
        web::resource("/weather")
            .route(web::get().to(get_weather))
    );
}

#[derive(Deserialize)]
struct Information {
    lang: String
}

#[derive(Serialize, Deserialize)]
struct WeatherApiResponse {
    current: WeatherApiCurrent,
}

#[derive(Serialize, Deserialize)]
struct WeatherApiCurrent {
    temp_c: f32,
    temp_f: f32,
    condition: WeatherApiCondition
}

#[derive(Serialize, Deserialize)]
struct WeatherApiCondition {
    text: String
}

#[derive(Serialize, Deserialize)]
struct WeatherResponse {
    temp_c: f32,
    temp_f: f32,
    condition: String
}

async fn get_cached_localized_weather(lang: String) -> String {
    let mut connection = get_connection().await;
    let weather_result: Result<String, RedisError> = connection.get(format!("weather_{lang}")).await;

    match weather_result {
        Ok(weather) => {
            return weather
        },
        Err(_error) => {
            return String::new()
        }
    }
}

async fn set_weather_cache(lang: String, value: String) {
    let mut connection = get_connection().await;
    let _: Result<String, RedisError> = connection.set_ex(format!("weather_{lang}"), value, 3600).await;
}

async fn get_weather(query: web::Query<Information>) -> impl Responder {
    let client = get_client();
    let config = get_config();

    let cached_weather: String = get_cached_localized_weather(query.lang.clone()).await;

    if !cached_weather.is_empty() {
        let json_cached_weather: WeatherResponse = serde_json::from_str(&cached_weather).expect("Cannot get JSON from cached weather");
        return HttpResponse::Ok().json(json_cached_weather)
    }

    let url = format!(
        "{url}/current.json?key={key}&q={q}&aqi=no&lang={lang}",
        url = config.base_url.weather_api.clone(),
        key = config.weather.key.as_str(),
        q = config.weather.city.as_str(),
        lang = query.lang.as_str()
    );

    let api_response = client
        .get(url)
        .send()
        .await
        .unwrap();

    let string_response = api_response
        .text()
        .await
        .unwrap();

    let response: WeatherApiResponse = serde_json::from_str(&string_response).unwrap();

    let json_response = Json(WeatherResponse {
        temp_c: response.current.temp_c,
        temp_f: response.current.temp_f,
        condition: response.current.condition.text
    });

    let stringified_response = serde_json::to_string(&json_response).unwrap();
    set_weather_cache(query.lang.clone(), stringified_response).await;
    return HttpResponse::Ok().json(json_response);
}