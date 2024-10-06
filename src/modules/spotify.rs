use std::fs;

use actix_web::{http::header::{self, ContentType}, web::{self, Json}, HttpResponse, Responder};
use awc::Client;
use base64_light::base64_encode;
use serde::{Deserialize, Serialize};
use struson::writer::simple::{SimpleJsonWriter, ValueWriter};

use crate::config;

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
    let track_data: Json<TrackData> = get_track_data().await;

    if track_data.status_code.is_none() {
        let track: &TrackItem = track_data.track.as_ref().unwrap();

        return HttpResponse::Ok().json(Json(ShortTrackData {
            is_active: track_data.is_active,
            track: Some(TrackItem {
                title: track.title.clone(),
                release_date: track.release_date.clone(),
                artist: track.artist.clone(),
                image: track.image.clone(),
                is_playing: track.is_playing.clone(),
                explicit: track.explicit.clone(),
                duration: track.duration.clone(),
                progress: track.progress.clone()
                
            })
        }))
    } else if track_data.status_code == Some(401) {
        refresh_spotify_token().await;
        let refreshed_track_data: Json<TrackData> = get_track_data().await;
        
        if refreshed_track_data.status_code.is_none() {
            return HttpResponse::Ok().json(refreshed_track_data)
        } else {
            return HttpResponse::Unauthorized().body("Check your credentials!")
        }
    } else if track_data.status_code == Some(204) {
        return HttpResponse::Ok().json(Json(ShortTrackData {
            is_active: track_data.is_active,
            track: None
        }))
    } else {
        return HttpResponse::InternalServerError().body("Cannot get currently playing Spotify track.")
    }
}

/* ## My track structs ## */

#[derive(Serialize, Deserialize)]
struct ShortTrackData {
    is_active: bool,
    track: Option<TrackItem>,
}

#[derive(Serialize, Deserialize)]
struct TrackData {
    status_code: Option<u16>,
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

/* ## API track structs ## */

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

async fn get_track_data() -> Json<TrackData> {
    let storage: TokenStorage = get_token_storage();
    
    let client = Client::default();
    let config = config();

    let url = format!(
        "{url}/me/player/currently-playing",
        url = config.base_url.spotify_api.clone()
    );

    let mut request = client.get(url)
        .append_header((header::ACCEPT_ENCODING, "gzip, deflate, br"))
        .append_header((header::AUTHORIZATION, format!("Bearer {}", storage.access_token.to_string())))
        .send()
        .await
        .unwrap();

    let body = request
        .body()
        .await
        .unwrap();

    let status_code = request.status();

    let utf = std::str::from_utf8(&body).unwrap();

    if status_code.as_u16() == 200 {
        let response_object: Result<ApiTrackData, serde_json::Error> = serde_json::from_str(utf);

        match response_object {
            Ok(response) => {
                return Json(TrackData {
                    status_code: None,
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
                });
            },
            Err(_error) => {
                return Json(TrackData {
                    status_code: Some(status_code.as_u16()),
                    is_active: false,
                    track: None
                });
            }
        }
    } else {
        return Json(TrackData {
            status_code: Some(status_code.as_u16()),
            is_active: false,
            track: None
        });
    }
}

async fn refresh_spotify_token() {
    let storage: TokenStorage = get_token_storage();
    let client = Client::default();
    let config = config();

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

    let mut request = client.post(url)
        .append_header(ContentType::form_url_encoded())
        .append_header((header::AUTHORIZATION, format!("Basic {}", encoded_credentials)))
        .send()
        .await
        .unwrap();

    let body = request
        .body()
        .await
        .unwrap();

    let utf = std::str::from_utf8(&body);

    match utf {
        Ok(valid_str) => {
            let response_object: Result<TokenResponse, serde_json::Error> = serde_json::from_str(valid_str);

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
                Err(_error) => {
                    return
                }
            }
        },
        Err(_error) => {
            return
        },
    }
}