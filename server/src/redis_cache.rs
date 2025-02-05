use crate::message_cache::MessageCache;
use crate::ServerError;
use crate::ServerError::{CacheCommunicationError, CacheInsertionError, CacheRemoveError};
use deadpool_redis::redis::cmd;
use deadpool_redis::{Config, Connection, Runtime};
use libsignal_protocol::ProtocolAddress;
use sam_common::sam_message::ServerEnvelope;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
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
            test_key: "hello".to_string(),
        }
    }

    async fn get_connection(&self) -> Connection {
        self.connection
            .get()
            .await
            .expect("Needs a redis connection")
    }
}

impl<T: MessageAvailabilityListener> MessageCache<T> for RedisCache<T> {
    async fn insert(
        &self,
        address: &ProtocolAddress,
        envelope: ServerEnvelope,
        message_guid: &str,
    ) {
        let mut connection = self.get_connection().await;

        let queue_key: String = self.get_message_queue_key(address);
        let queue_metadata_key: String = self.get_message_queue_metadata_key(address);
        let queue_total_index_key: String = self.get_queue_index_key();

        envelope.server_guid = Some(message_guid.to_string());
        let data = envelope.encode_to_vec();

        let message_guid_exists = cmd("HEXISTS")
            .arg(&queue_metadata_key)
            .arg(message_guid)
            .query_async::<u8>(&mut connection)
            .await?;

        if message_guid_exists == 1 {
            let num = cmd("HGET")
                .arg(&queue_metadata_key)
                .arg(message_guid)
                .query_async::<String>(&mut connection)
                .await?;

            return Ok(num.parse().expect("Could not parse redis id"));
        }

        let message_id = cmd("HINCRBY")
            .arg(&queue_metadata_key)
            .arg("counter")
            .arg(1)
            .query_async::<u64>(&mut connection)
            .await?;

        cmd("ZADD")
            .arg(&queue_key)
            .arg("NX")
            .arg(message_id)
            .arg(&data)
            .query_async::<()>(&mut connection)
            .await?;

        cmd("HSET")
            .arg(&queue_metadata_key)
            .arg(message_guid)
            .arg(message_id)
            .query_async::<()>(&mut connection)
            .await?;

        cmd("EXPIRE")
            .arg(&queue_key)
            .arg(2678400)
            .query_async::<()>(&mut connection)
            .await?;

        cmd("EXPIRE")
            .arg(&queue_metadata_key)
            .arg(2678400)
            .query_async::<()>(&mut connection)
            .await?;

        let time = SystemTime::now();
        let time_in_millis: u64 = time.duration_since(UNIX_EPOCH)?.as_secs();

        cmd("ZADD")
            .arg(&queue_total_index_key)
            .arg("NX")
            .arg(time_in_millis)
            .arg(&queue_key)
            .query_async::<()>(&mut connection)
            .await?;

        // notifies the message availability manager
        let queue_name = format!("{}::{}", address.name(), address.device_id());
        if let Some(listener) = self.listeners.lock().await.get(&queue_name) {
            listener.lock().await.handle_new_messages_available().await;
        }

        Ok(message_id)
    }

    fn remove(&self, address: &ProtocolAddress, message_guids: Vec<String>) {
        todo!()
    }

    async fn has_messages(&self, address: &ProtocolAddress) -> Result<bool, ServerError> {
        let mut connection = self.get_connection().await;

        let msg_count = cmd("ZCARD")
            .arg(self.get_message_queue_key(address))
            .query_async::<u32>(&mut connection)
            .await
            .map_err(|redis_error| CacheCommunicationError(redis_error))?;

        let queue_name = format!("{}::{}", address.name(), address.device_id());
        if let Some(listener) = self.listeners.lock().await.get(&queue_name) {
            listener.lock().await.handle_new_messages_available().await;
        }

        Ok(msg_count > 0)
    }

    fn get_all_messages(&self, address: &ProtocolAddress) {
        todo!()
    }

    async fn lock_queue_for_persistence(
        &self,
        address: &ProtocolAddress,
    ) -> Result<(), ServerError> {
        let mut connection = self.get_connection().await;

        cmd("SETEX")
            .arg(self.get_persist_in_progress_key(address))
            .arg(30)
            .arg("1")
            .query_async::<()>(&mut connection)
            .await
            .map_err(|redis_error| {
                CacheInsertionError("Persistence Key".to_string(), redis_error)
            })?;

        Ok(())
    }

    async fn unlock_queue_for_persistence(
        &self,
        address: &ProtocolAddress,
    ) -> Result<(), ServerError> {
        let mut connection = self.get_connection();

        let result = cmd("DEL")
            .arg(self.get_persist_in_progress_key(address))
            .query_async::<()>(&mut connection)
            .await
            .map_err(|redis_error| CacheRemoveError("Persistence key".to_string(), redis_error));

        let queue_name = format!("{}::{}", address.name(), address.device_id());
        if let Some(listener) = self.listeners.lock().await.get(&queue_name) {
            listener.lock().await.handle_messages_persisted().await;
        }

        result
    }

    fn get_persisted_messages(&self, address: &ProtocolAddress, limit: i32) {
        todo!()
    }

    fn add_message_availability_listener(
        &mut self,
        address: &ProtocolAddress,
        listener: Arc<std::sync::Mutex<T>>,
    ) {
        todo!()
    }

    fn remove_message_availability_listener(&mut self, address: &ProtocolAddress) {
        todo!()
    }

    fn get_too_old_message_queues(&self, max_time: u64, limit: u8) {
        todo!()
    }

    fn get_message_queue_key(&self, address: &ProtocolAddress) -> String {
        #[cfg(not(test))]
        return format!(
            "user_queue::{{{}::{}}}",
            address.name(),
            address.device_id()
        );
        #[cfg(test)]
        format!(
            "{}user_queue::{{{}::{}}}",
            self.test_key,
            address.name(),
            address.device_id()
        )
    }

    fn get_persist_in_progress_key(&self, address: &ProtocolAddress) -> String {
        #[cfg(not(test))]
        return format!(
            "user_queue_persisting::{{{}::{}}}",
            address.name(),
            address.device_id()
        );
        #[cfg(test)]
        format!(
            "{}user_queue_persisting::{{{}::{}}}",
            self.test_key,
            address.name(),
            address.device_id()
        )
    }

    fn get_message_queue_metadata_key(&self, address: &ProtocolAddress) -> String {
        #[cfg(not(test))]
        return format!(
            "user_queue_metadata::{{{}::{}}}",
            address.name(),
            address.device_id()
        );
        #[cfg(test)]
        format!(
            "{}user_queue_metadata::{{{}::{}}}",
            self.test_key,
            address.name(),
            address.device_id()
        )
    }

    fn get_queue_index_key(&self) -> String {
        #[cfg(not(test))]
        return "user_queue_index_key".to_string();
        #[cfg(test)]
        format!("{}user_queue_index_key", self.test_key)
    }
}
