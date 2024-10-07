use std::fs;

use actix_web::{web::{self, Json}, HttpResponse, Responder};
use base64_light::base64_encode;
use serde::{Deserialize, Serialize};
use struson::writer::simple::{SimpleJsonWriter, ValueWriter};

use crate::{client::get_client, config::get_config};

pub fn spotify_config(config: &mut web::ServiceConfig) {
    config.service(
        web::resource("/spotify")
            .route(web::get().to(get_currently_playing_track))
    );
}

#[derive(Serialize, Deserialize)]
struct TokenStorage {
    access_token: String,
    refresh_token: String
}

#[derive(Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>
}

fn get_token_storage() -> TokenStorage {
    let file = fs::File::open("storage/spotify_token_storage.json").expect("Cannot open spotify token storage file");
    let token_storage: TokenStorage = serde_json::from_reader(file).expect("Spotify token storage file should be proper JSON");
    return token_storage
}

async fn get_currently_playing_track() -> impl Responder {
    let track_data: Result<Json<TrackData>, u16> = get_track_data().await;

    match track_data {
        Ok(track) => {
            return HttpResponse::Ok().json(track)
        },
        Err(code) => {
            if code == 401 {
                refresh_spotify_token().await;
                let refreshed_track_data: Result<Json<TrackData>, u16> = get_track_data().await;
                
                match refreshed_track_data {
                    Ok(track) => {
                        return HttpResponse::Ok().json(track)
                    },
                    Err(_code) => {
                        return HttpResponse::Unauthorized().body("Check your credentials!")
                    }
                }
            } else if code == 204 {
                return HttpResponse::Ok().json(Json(TrackData {
                    is_active: false,
                    track: None
                }))
            } else {
                return HttpResponse::InternalServerError().body("Cannot get currently playing Spotify track.")
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TrackData {
    is_active: bool,
    track: Option<TrackItem>,
}

#[derive(Serialize, Deserialize)]
struct TrackItem {
    title: String,
    release_date: String,
    artist: String,
    image: String,
    is_playing: bool,
    explicit: bool,
    duration: i64,
    progress: i64
}

#[derive(Serialize, Deserialize)]
struct ApiTrackData {
    progress_ms: i64,
    is_playing: bool,
    item: ApiTrackItem,
}

#[derive(Serialize, Deserialize)]
struct ApiTrackItem {
    name: String,
    artists: Vec<ApiTrackArtist>,
    album: ApiTrackAlbum,
    explicit: bool,
    duration_ms: i64
}

#[derive(Serialize, Deserialize)]
struct ApiTrackArtist {
    name: String
}

#[derive(Serialize, Deserialize)]
struct ApiTrackAlbum {
    release_date: String,
    images: Vec<ApiTrackImage>,
}

#[derive(Serialize, Deserialize)]
struct ApiTrackImage {
    url: String
}

async fn get_track_data() -> Result<Json<TrackData>, u16> {
    let storage: TokenStorage = get_token_storage();
    let client: reqwest::Client = get_client();
    let config = get_config();

    let url = format!(
        "{url}/me/player/currently-playing",
        url = config.base_url.spotify_api.clone()
    );

    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT_ENCODING, "gzip, deflate, br")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", storage.access_token.to_string()))
        .send()
        .await
        .unwrap();

    let status_code = response.status().as_u16();

    let string_response = response
        .text()
        .await
        .unwrap();

    if status_code == 200 {
        let response_object: Result<ApiTrackData, serde_json::Error> = serde_json::from_str(&string_response);

        match response_object {
            Ok(response) => {
                return Ok(Json(TrackData {
                    is_active: true,
                    track: Some(TrackItem {
                        title: response.item.name,
                        release_date: response.item.album.release_date,
                        artist: response.item.artists[0].name.clone(),
                        image: response.item.album.images[1].url.clone(),
                        is_playing: response.is_playing,
                        explicit: response.item.explicit,
                        duration: response.item.duration_ms,
                        progress: response.progress_ms
                        
                    })
                }));
            },
            Err(_error) => {
                return Err(status_code)
            }
        }
    } else {
        return Err(status_code)
    }
}

async fn refresh_spotify_token() {
    let storage: TokenStorage = get_token_storage();
    let client = get_client();
    let config = get_config();

    let url = format!(
        "{url}/token?refresh_token={refresh_token}&grant_type=refresh_token",
        url = config.base_url.spotify_accounts_api.clone(),
        refresh_token = storage.refresh_token
    );

    let credentials = format!("{client_id}:{secret}",
        client_id = config.spotify.client_id,
        secret = config.spotify.secret
    );
    let encoded_credentials = base64_encode(&credentials);

    let response = client
        .post(url)
        .header(reqwest::header::ACCEPT_ENCODING, "gzip, deflate, br")
        .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(reqwest::header::CONTENT_LENGTH, 0)
        .header(reqwest::header::AUTHORIZATION, format!("Basic {}", encoded_credentials))
        .send()
        .await
        .unwrap();

    let string_response = response
        .text()
        .await
        .unwrap();

    let response_object: Result<TokenResponse, serde_json::Error> = serde_json::from_str(&string_response);

    match response_object {
        Ok(response) => {
            let _ = fs::remove_file("storage/spotify_token_storage.json");
            let file = fs::File::create("storage/spotify_token_storage.json").expect("Cannot open spotify token storage file");

            let json_writer = SimpleJsonWriter::new(file);

            let final_refresh_token: String;

            if response.refresh_token.is_none() {
                final_refresh_token = storage.refresh_token
            } else {
                final_refresh_token = response.refresh_token.unwrap()
            }

            json_writer.write_object(|object_writer| {
                object_writer.write_string_member("access_token", &response.access_token)?;
                object_writer.write_string_member("refresh_token", &final_refresh_token)?;
                Ok(())
            }).unwrap();
        },
        Err(error) => {
            println!("{}", error);
            return
        }
    }
}