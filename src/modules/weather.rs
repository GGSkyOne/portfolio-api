use actix_web::{web, HttpResponse, Responder};
use awc::Client;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::config;

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

async fn get_cached_weather(lang: String) -> String {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut connection = client.get_multiplexed_async_connection().await.unwrap();

    let weather: String = connection.get(format!("weather_{lang}")).await.unwrap();
    if !weather.is_empty() {
        return weather
    } else {
        return String::from("")
    }
}

async fn get_weather(query: web::Query<Information>) -> impl Responder {
    let client = Client::default();
    let config = config();

    let cached_weather: String = get_cached_weather(query.lang.clone()).await;

    if !cached_weather.is_empty() {
        let sr = cached_weather.to_string();
        println!("{}", format!("{}", sr));
    } else {
        println!("no")
    }

    let url = format!(
        "{url}/current.json?key={key}&q={q}&aqi=no&lang={lang}",
        url = config.base_url.weather_api.clone(),
        key = config.weather.key.as_str(),
        q = config.weather.city.as_str(),
        lang = query.lang.as_str()
    );

    let request = client.get(url)
        .send()
        .await
        .unwrap()
        .body()
        .await
        .unwrap();

    let utf = std::str::from_utf8(&request);

    match utf {
        Ok(valid_str) => {
            let response: WeatherApiResponse = serde_json::from_str(valid_str).unwrap();

            return HttpResponse::Ok().json(WeatherResponse {
                temp_c: response.current.temp_c,
                temp_f: response.current.temp_f,
                condition: response.current.condition.text
            });
        },
        Err(error) => {
            return HttpResponse::InternalServerError().body(format!("Invalid UTF-8 sequence: {}", error))
        },
    }
}