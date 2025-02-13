use crate::redis_cache::RedisCache;
use deadpool_redis::redis::cmd;
use sam_common::sam_message::{EnvelopeType, ServerEnvelope};
use uuid::Uuid;

pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub async fn teardown(key: &str, mut con: deadpool_redis::Connection) {
    let pattern = format!("{}*", key);

    let mut cursor = 0;
    loop {
        let (new_cursor, keys): (u64, Vec<String>) = cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern.clone())
            .query_async(&mut con)
            .await
            .expect("Teardown scan failed");

        if !keys.is_empty() {
            cmd("DEL")
                .arg(&keys)
                .query_async::<u8>(&mut con)
                .await
                .expect("Teardown delete failed");
        }

        cursor = new_cursor;
        if cursor == 0 {
            break;
        }
    }
}

pub fn generate_envelope(to: &str, from: &str, guid: &str) -> ServerEnvelope {
    ServerEnvelope {
        r#type: EnvelopeType::SignalMessage.into(),
        destination: to.into(),
        source: from.into(),
        id: guid.into(),
        content: vec![10, 20, 30].into(),
    }
}

pub fn new_redis_cache() -> RedisCache {
    let _ = dotenv::dotenv();
    RedisCache::new(std::env::var("REDIS_URL").expect("Unable to read REDIS_URL .env var"))
}

// TODO: Implement MockSocket for tests when WebSockets are implemented
