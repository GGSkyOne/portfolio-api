use std::sync::Arc;

use lazy_static::lazy_static;
use reqwest::Client;
use rustls::lock::Mutex;

lazy_static! {
    static ref REQWEST_CLIENT: Arc<Mutex<Option<Client>>> = Arc::new(Mutex::new(None));
}

pub fn create_client() {
    let new_client: Client = reqwest::Client::new();
    println!("Created client successfully");
    
    let mut client = REQWEST_CLIENT.lock().unwrap();
    *client = Some(new_client);
}

pub fn get_client() -> Client {
    let client = REQWEST_CLIENT.lock().unwrap();
    return client.as_ref().expect("Failed to get client, is it created?").clone();
}