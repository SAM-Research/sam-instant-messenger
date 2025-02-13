use crate::message_cache::MessageCache;
#[cfg(test)]
use crate::test_utils::random_string;
use deadpool_redis::redis::cmd;
use deadpool_redis::{Config, Connection, Runtime};
use libsignal_protocol::ProtocolAddress;
use prost::bytes::Bytes;
use prost::Message;
use sam_common::sam_message::ServerEnvelope;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RedisCache {
    connection: deadpool_redis::Pool,
    #[cfg(test)]
    pub test_key: String,
}

impl RedisCache {
    pub fn new(redis_url: String) -> Self {
        let redis_config = Config::from_url(redis_url);
        let redis_pool: deadpool_redis::Pool = redis_config
            .create_pool(Some(Runtime::Tokio1))
            .expect("Failed to create connection pool");
        #[cfg(not(test))]
        return Self {
            connection: redis_pool,
        };
        #[cfg(test)]
        Self {
            connection: redis_pool,
            test_key: random_string(8).to_string(),
        }
    }

    async fn get_connection(&self) -> Connection {
        self.connection
            .get()
            .await
            .expect("Needs a redis connection")
    }
}

#[async_trait::async_trait]
impl MessageCache for RedisCache {
    async fn insert(
        &self,
        address: &ProtocolAddress,
        envelope: &mut ServerEnvelope,
        message_guid: &str,
    ) -> String {
        let mut connection = self.get_connection().await;

        let queue_key: String = self.get_message_queue_key(address);
        let queue_metadata_key: String = self.get_message_queue_metadata_key(address);
        let queue_total_index_key: String = self.get_queue_index_key();

        envelope.id = message_guid.to_string();
        let data = envelope.encode_to_vec();

        let message_id_exists = cmd("HEXISTS")
            .arg(&queue_metadata_key)
            .arg(message_guid)
            .query_async::<u8>(&mut connection)
            .await
            .expect("HEXISTS command returned an error");

        if message_id_exists == 1 {
            let message_id = cmd("HGET")
                .arg(&queue_metadata_key)
                .arg(message_guid)
                .query_async::<String>(&mut connection)
                .await
                .expect("redis HGET: could not query message guid that should be present");

            return message_id;
        }

        let message_number = cmd("HINCRBY")
            .arg(&queue_metadata_key)
            .arg("counter")
            .arg(1)
            .query_async::<u64>(&mut connection)
            .await
            .expect("redis HINCRBY: could not increase and query message number");

        cmd("ZADD")
            .arg(&queue_key)
            .arg("NX")
            .arg(message_number)
            .arg(&data)
            .exec_async(&mut connection)
            .await
            .expect("redis ZADD: Could not insert message redis");

        cmd("HSET")
            .arg(&queue_metadata_key)
            .arg(message_guid)
            .arg(message_number)
            .exec_async(&mut connection)
            .await
            .expect("redis ZADD: Could not add message guid");

        cmd("EXPIRE")
            .arg(&queue_key)
            .arg(2678400)
            .exec_async(&mut connection)
            .await
            .expect("redis EXPIRE: Could not set expiration on queue key");

        cmd("EXPIRE")
            .arg(&queue_metadata_key)
            .arg(2678400)
            .exec_async(&mut connection)
            .await
            .expect("redis EXPIRE: Could not set expiration on metadata key");

        let time = SystemTime::now();
        let time_in_millis: u64 = time
            .duration_since(UNIX_EPOCH)
            .expect("Could not get time since UNIX")
            .as_secs();

        cmd("ZADD")
            .arg(&queue_total_index_key)
            .arg("NX")
            .arg(time_in_millis)
            .arg(&queue_key)
            .exec_async(&mut connection)
            .await
            .expect("redis ZADD: Could not add time when message was added");

        // TODO: notify the message availability manager

        message_number.to_string()
    }

    // Removes and returns the messages
    async fn remove(
        &self,
        address: &ProtocolAddress,
        message_guids: Vec<String>,
    ) -> Vec<ServerEnvelope> {
        let mut connection = self.get_connection().await;

        let queue_key: String = self.get_message_queue_key(address);
        let queue_metadata_key: String = self.get_message_queue_metadata_key(address);
        let queue_total_index_key: String = self.get_queue_index_key();

        let mut removed_messages: Vec<ServerEnvelope> = Vec::new();

        for guid in message_guids {
            let message_id: Option<String> = cmd("HGET")
                .arg(&queue_metadata_key)
                .arg(&guid)
                .query_async(&mut connection)
                .await
                .expect("redis HGET: could not query message guid");

            if let Some(message_number) = message_id.clone() {
                // retrieving the message
                let messages = cmd("ZRANGE")
                    .arg(&queue_key)
                    .arg(&message_number)
                    .arg(&message_number)
                    .arg("BYSCORE")
                    .arg("LIMIT")
                    .arg(0)
                    .arg(1)
                    .query_async::<Option<Vec<Vec<u8>>>>(&mut connection)
                    .await
                    .expect("redis ZRANGE: could not query message based on id");

                // delete the message
                cmd("ZREMRANGEBYSCORE")
                    .arg(&queue_key)
                    .arg(&message_number)
                    .arg(&message_number)
                    .exec_async(&mut connection)
                    .await
                    .expect(
                        "redis ZREMRANGEBYSCORE: could not remove message that was just queried",
                    );

                // delete the guid from the cache
                cmd("HDEL")
                    .arg(&queue_metadata_key)
                    .arg(&guid)
                    .exec_async(&mut connection)
                    .await
                    .expect("redis HDEL: could not remove message id that was just queried");

                if let Some(envelope) = messages {
                    removed_messages.push(
                        Message::decode(Bytes::from(envelope[0].clone()))
                            .ok()
                            .expect("It is a message"),
                    );
                }
            }
        }

        if cmd("ZCARD")
            .arg(&queue_key)
            .query_async::<u64>(&mut connection)
            .await
            .expect("redis ZCARD: could not query message guid")
            == 0
        {
            cmd("DEL")
                .arg(&queue_key)
                .query_async::<()>(&mut connection)
                .await
                .expect("redis DEL: should be able to delete message queue");

            cmd("DEL")
                .arg(&queue_metadata_key)
                .query_async::<()>(&mut connection)
                .await
                .expect("redis DEL: should be able to delete message id from the cache");

            cmd("ZREM")
                .arg(&queue_total_index_key)
                .arg(&queue_key)
                .query_async::<()>(&mut connection)
                .await
                .expect("redis ZREM: should be able to remove message id from the cache");
        }

        removed_messages
    }

    async fn has_messages(&self, address: &ProtocolAddress) -> bool {
        let mut connection = self.get_connection().await;

        let msg_count = cmd("ZCARD")
            .arg(self.get_message_queue_key(address))
            .query_async::<u32>(&mut connection)
            .await
            .expect("redis ZCARD: Should be able to make a command");

        let queue_name = format!("{}::{}", address.name(), address.device_id());

        // TODO: Notify the message availability handler

        msg_count > 0
    }

    async fn get_all_messages(&self, address: &ProtocolAddress) -> Vec<ServerEnvelope> {
        let mut connection = self.get_connection().await;
        let queue_key = self.get_message_queue_key(address);
        let queue_lock_key = self.get_persist_in_progress_key(address);

        let locked = cmd("GET")
            .arg(&queue_lock_key)
            .query_async::<Option<String>>(&mut connection)
            .await
            .expect("redis GET should be able to query lock key.");

        // if there is a queue lock key on, due to persist of message.
        if locked.is_some() {
            return Vec::new();
        }

        let messages = cmd("ZRANGE")
            .arg(queue_key.clone())
            .arg("(-1") // gets all messages
            .arg("+inf")
            .arg("BYSCORE")
            .arg("LIMIT")
            .arg(0)
            .arg(100) // page size
            .arg("WITHSCORES")
            .query_async::<Vec<Vec<u8>>>(&mut connection)
            .await
            .expect("redis ZRANGE should be able to query messages");

        let mut envelopes: Vec<ServerEnvelope> = Vec::new();
        // messages is a [envelope1, msg_id1, envelope2, msg_id2, ...]
        for i in (0..messages.len()).step_by(2) {
            if let Ok(envelope) = Message::decode(Bytes::from(messages[i].clone())) {
                envelopes.push(envelope);
            }
        }
        envelopes
    }

    async fn lock_queue_for_persistence(&self, address: &ProtocolAddress) -> () {
        let mut connection = self.get_connection().await;

        cmd("SETEX")
            .arg(self.get_persist_in_progress_key(address))
            .arg(30)
            .arg("1")
            .exec_async(&mut connection)
            .await
            .expect("redis SETEX should be able to set persistence lock key");

        ()
    }

    async fn unlock_queue_for_persistence(&self, address: &ProtocolAddress) -> () {
        let mut connection = self.get_connection().await;

        cmd("DEL")
            .arg(self.get_persist_in_progress_key(address))
            .exec_async(&mut connection)
            .await
            .expect("DEL command returned an error");

        let queue_name = format!("{}::{}", address.name(), address.device_id());

        // TODO: Notify the message availability handler that messages have persisted
    }

    async fn get_persisted_messages(
        &self,
        address: &ProtocolAddress,
        limit: i32,
    ) -> Vec<ServerEnvelope> {
        let mut connection = self.get_connection().await;

        let messages = cmd("ZRANGE")
            .arg(self.get_message_queue_key(address))
            .arg(0)
            .arg(limit)
            .query_async::<Vec<Vec<u8>>>(&mut connection)
            .await
            .expect("redis ZRANGE should get messages when connection is active");

        let valid_envelopes: Vec<ServerEnvelope> = messages
            .into_iter()
            .filter_map(|m| Message::decode(Bytes::from(m)).ok())
            .collect();

        valid_envelopes
    }

    fn add_message_availability_listener(&mut self) {
        // TODO: implement when websockets are implemented
    }

    fn remove_message_availability_listener(&mut self) {
        // TODO: implement when websockets are implemented
    }

    async fn get_too_old_message_queues(&self, max_time: u64, limit: u8) -> Vec<String> {
        let mut connection = self.get_connection().await;
        let queue_index_key = self.get_queue_index_key();

        let results = cmd("ZRANGE")
            .arg(&queue_index_key)
            .arg(0)
            .arg(max_time)
            .arg("BYSCORE")
            .arg("LIMIT")
            .arg(0)
            .arg(limit)
            .query_async::<Vec<String>>(&mut connection)
            .await
            .expect("redis ZRANGE: could not query query time of insertion for message queue");

        if !results.is_empty() {
            cmd("ZREM")
                .arg(&queue_index_key)
                .arg(&results)
                .exec_async(&mut connection)
                .await
                .expect("redis ZREM: should be able to remove message queue");
        }
        results
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

#[cfg(test)]
pub mod redis_cache_tests {
    use super::*;
    use crate::test_utils::{
        cache::{generate_envelope, generate_uuid, new_redis_cache, teardown},
        user::new_protocol_address,
    };
    /*
        TODO: Test when websockets are implemented
        #[tokio::test]
        async fn test_message_availability_listener_new_messages() {
            let mut redis_cache: RedisCache = RedisCache::new();
            let uuid = generate_uuid();
            let address = new_protocol_address();

            let mut envelope = generate_envelope(&uuid);

            redis
                .add_message_availability_listener(&address, websocket.clone())
                .await;

            message_cache
                .insert(&address, &mut envelope, &uuid)
                .await
                .unwrap();

            assert!(websocket.lock().await.evoked_handle_new_messages);
        }
         */

    #[tokio::test]
    async fn test_insert() {
        let redis_cache = new_redis_cache();
        let mut connection = redis_cache.get_connection().await;
        let address = new_protocol_address();
        let message_guid = generate_uuid();
        let to = generate_uuid();
        let from = generate_uuid();

        let mut envelope = generate_envelope(&to, &from, &message_guid);

        let message_id = redis_cache
            .insert(&address, &mut envelope, &message_guid)
            .await;

        let result = cmd("ZRANGEBYSCORE")
            .arg(redis_cache.get_message_queue_key(&address))
            .arg(&message_id)
            .arg(&message_id)
            .query_async::<Vec<Vec<u8>>>(&mut connection)
            .await
            .unwrap();

        teardown(&redis_cache.test_key, connection).await;

        assert_eq!(
            envelope,
            Message::decode(Bytes::from(result[0].clone()))
                .ok()
                .expect("It is a message")
        );
    }

    #[tokio::test]
    async fn test_insert_same_id() {
        let redis_cache: RedisCache = new_redis_cache();

        let mut connection = redis_cache.get_connection().await;

        let address = new_protocol_address();

        let message_guid = generate_uuid();
        let to = generate_uuid();
        let from = generate_uuid();

        let mut envelope1 = generate_envelope(&to, &from, &message_guid);
        let mut envelope2 = generate_envelope(&to, &from, &message_guid);

        let message_id = redis_cache
            .insert(&address, &mut envelope1, &message_guid)
            .await;

        // should return the same message id
        let message_id_2 = redis_cache
            .insert(&address, &mut envelope2, &message_guid)
            .await;

        let result = cmd("ZRANGEBYSCORE")
            .arg(redis_cache.get_message_queue_key(&address))
            .arg(&message_id_2)
            .arg(&message_id_2)
            .query_async::<Vec<Vec<u8>>>(&mut connection)
            .await
            .unwrap();

        teardown(&redis_cache.test_key, connection).await;

        assert_eq!(
            envelope1,
            Message::decode(Bytes::from(result[0].clone()))
                .ok()
                .expect("It is a message")
        );

        assert_eq!(message_id, message_id_2);
    }

    #[tokio::test]
    async fn test_insert_different_ids() {
        let redis_cache: RedisCache = new_redis_cache();

        let mut connection = redis_cache.get_connection().await;

        let address = new_protocol_address();

        let message_guid1 = generate_uuid();
        let message_guid2 = generate_uuid();
        let to = generate_uuid();
        let from = generate_uuid();
        let mut envelope1 = generate_envelope(&to, &from, &message_guid1);
        let mut envelope2 = generate_envelope(&to, &from, &message_guid2);

        // inserting messages
        let message_id = redis_cache
            .insert(&address, &mut envelope1, &message_guid1)
            .await;

        let message_id_2 = redis_cache
            .insert(&address, &mut envelope2, &message_guid2)
            .await;

        // querying the envelopes
        let result_1 = cmd("ZRANGEBYSCORE")
            .arg(redis_cache.get_message_queue_key(&address))
            .arg(&message_id)
            .arg(&message_id)
            .query_async::<Vec<Vec<u8>>>(&mut connection)
            .await
            .unwrap();

        let result_2 = cmd("ZRANGEBYSCORE")
            .arg(redis_cache.get_message_queue_key(&address))
            .arg(&message_id_2)
            .arg(&message_id_2)
            .query_async::<Vec<Vec<u8>>>(&mut connection)
            .await
            .unwrap();

        teardown(&redis_cache.test_key, connection).await;

        // they are inserted as two different messages
        assert_ne!(message_id, message_id_2);

        assert_ne!(
            ServerEnvelope::decode(Bytes::from(result_1[0].clone())).unwrap(),
            ServerEnvelope::decode(Bytes::from(result_2[0].clone())).unwrap()
        );
    }

    #[tokio::test]
    async fn test_remove() {
        let redis_cache: RedisCache = new_redis_cache();
        let connection = redis_cache.get_connection().await;
        let address = new_protocol_address();
        let message_guid = generate_uuid();
        let to = generate_uuid();
        let from = generate_uuid();
        let mut envelope = generate_envelope(&to, &from, &message_guid);

        redis_cache
            .insert(&address, &mut envelope, &message_guid)
            .await;

        let removed_messages = redis_cache.remove(&address, vec![message_guid]).await;

        teardown(&redis_cache.test_key, connection).await;

        assert_eq!(removed_messages.len(), 1);
        assert_eq!(removed_messages[0], envelope);
    }

    #[tokio::test]
    async fn test_get_all_messages() {
        let redis_cache: RedisCache = new_redis_cache();
        let connection = redis_cache.get_connection().await;
        let address = new_protocol_address();
        let mut envelopes = Vec::new();

        for _ in 0..10 {
            let message_guid = generate_uuid();
            let to = generate_uuid();
            let from = generate_uuid();
            let mut envelope = generate_envelope(&to, &from, &message_guid);

            redis_cache
                .insert(&address, &mut envelope, &message_guid)
                .await;

            envelopes.push(envelope);
        }

        //getting those messages
        let messages = redis_cache.get_all_messages(&address).await;

        teardown(&redis_cache.test_key, connection).await;

        assert_eq!(messages.len(), 10);

        for (message, envelope) in messages.into_iter().zip(envelopes.into_iter()) {
            assert_eq!(message, envelope);
        }
    }

    #[tokio::test]
    async fn test_has_messages() {
        let redis_cache: RedisCache = new_redis_cache();
        let connection = redis_cache.get_connection().await;
        let address = new_protocol_address();
        let message_guid = generate_uuid();
        let to = generate_uuid();
        let from = generate_uuid();

        let mut envelope = generate_envelope(&to, &from, &message_guid);

        let does_not_has_messages = redis_cache.has_messages(&address).await;

        redis_cache
            .insert(&address, &mut envelope, &message_guid)
            .await;

        let has_messages = redis_cache.has_messages(&address).await;

        teardown(&redis_cache.test_key, connection).await;

        assert!(!does_not_has_messages);
        assert!(has_messages);
    }

    /*
    TODO: Implement when WebSockets are implemented
    #[tokio::test]
    async fn test_get_messages_to_persist() {
        let redis_cache: RedisCache = new_redis_cache();
        let connection = redis_cache.get_connection().await;
        let address = new_protocol_address();
        let message_guid = generate_uuid();
        let to = generate_uuid();
        let from = generate_uuid();

        let mut envelope = generate_envelope(&to, &from, &message_guid);

        redis_cache
            .insert(&address, &mut envelope, &message_guid)
            .await
            .unwrap();

        let envelopes = redis_cache
            .get_messages_to_persist(&address, -1)
            .await
            .unwrap();

        teardown(&message_cache.test_key, connection).await;

        assert_eq!(envelopes.len(), 1);
    }
     */
}
