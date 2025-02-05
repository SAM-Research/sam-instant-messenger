use crate::message_cache::MessageCache;
use deadpool_redis::{Config, Runtime};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait::async_trait]
pub trait MessageAvailabilityListener {
    async fn handle_new_messages_available(&mut self) -> bool;

    async fn handle_messages_persisted(&mut self) -> bool;
}

type ListenerMap<T> = Arc<Mutex<HashMap<String, Arc<Mutex<T>>>>>;

struct RedisCache<T: MessageAvailabilityListener> {
    connection: deadpool_redis::Pool,
    listeners: ListenerMap<T>,
    #[cfg(test)]
    pub test_key: String,
}

impl<T: MessageAvailabilityListener> RedisCache<T> {
    pub fn new() -> Self {
        let _ = dotenv::dotenv();
        let redis_url = std::env::var("REDIS_URL").expect("Unable to read REDIS_URL .env var");
        let redis_config = Config::from_url(redis_url);
        let redis_pool: deadpool_redis::Pool = redis_config
            .create_pool(Some(Runtime::Tokio1))
            .expect("Failed to create connection pool");
        #[cfg(not(test))]
        return Self {
            connection: redis_pool,
            listeners: Arc::new(Mutex::new(HashMap::new())),
        };
        #[cfg(test)]
        Self {
            connection: redis_pool,
            listeners: Arc::new(Mutex::new(HashMap::new())),
            test_key: random_string(8),
        }
    }

    fn get_connection(&self) -> deadpool_redis::Pool {}
}

impl<T: MessageAvailabilityListener> MessageCache for RedisCache<T> {
    fn insert() {}

    fn remove() {
        todo!()
    }

    fn has_messages() -> bool {
        todo!()
    }

    fn get_all_messages() {
        todo!()
    }

    fn lock_queue_for_persistence() {
        todo!()
    }

    fn unlock_queue_for_persistence() {
        todo!()
    }

    fn get_persisted_messages() {
        todo!()
    }

    fn add_message_availability_listener() {
        todo!()
    }

    fn remove_message_availability_listener() {
        todo!()
    }

    fn get_too_old_message_queues() {
        todo!()
    }
}
