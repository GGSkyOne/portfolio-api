use std::sync::Arc;

use lazy_static::lazy_static;
use redis::aio::MultiplexedConnection;
use rustls::lock::Mutex;

use crate::config::get_config;

lazy_static! {
    static ref REDIS_CONNECTION: Arc<Mutex<Option<MultiplexedConnection>>> = Arc::new(Mutex::new(None));
}

pub async fn connect_to_redis() {
    let config = get_config();

    let client = redis::Client::open(config.redis.host).unwrap();
    let conn = client.get_multiplexed_async_connection().await.unwrap();
    println!("Connected to Redis successfully");

    let mut connection = REDIS_CONNECTION.lock().unwrap();
    *connection = Some(conn)
}

pub async fn get_connection() -> MultiplexedConnection {
   let connection = REDIS_CONNECTION.lock().unwrap();
   return connection.as_ref().expect("Connection to Redis is not established").clone()
}