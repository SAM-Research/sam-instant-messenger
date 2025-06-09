use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use log::{debug, error};
use prost::Message;
use sam_common::{address::DeviceAddress, sam_message::ServerEnvelope, AccountId, DeviceId};
use sqlx::{Pool, Postgres};
use tokio::sync::{
    mpsc::{self},
    Mutex,
};

use crate::managers::{
    error::MessageManagerError,
    traits::message_manager::{EnvelopeId, MessageManager},
};

#[derive(Debug, Clone)]
pub struct PostgresMessageManager {
    pool: Pool<Postgres>,
    subscribers: Arc<Mutex<HashMap<DeviceAddress, mpsc::Sender<EnvelopeId>>>>,
    channel_buffer: usize,
}

impl PostgresMessageManager {
    pub fn new(pool: Pool<Postgres>, channel_buffer: usize) -> Self {
        Self {
            pool,
            subscribers: Arc::default(),
            channel_buffer,
        }
    }
}

#[async_trait]
impl MessageManager for PostgresMessageManager {
    async fn channel_buffer(&self) -> usize {
        self.channel_buffer
    }
    async fn insert_envelope(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
        envelope: ServerEnvelope,
    ) -> Result<(), MessageManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        let env = envelope_id.uuid();
        let bytes = envelope.encode_to_vec();
        let result = sqlx::query!(
            r#"
            INSERT INTO msg_queue (receiver, msg, envelope_id)
            SELECT id, 
                   $1,
                   $2
            FROM devices
            WHERE owner = 
                    (SELECT id
                    FROM accounts
                    WHERE account_id = $3)
            AND device_id = $4
            "#,
            bytes,
            env,
            aci,
            dev
        )
        .execute(&self.pool)
        .await
        .map_err(|err| match err {
            _ => {
                error!("An error occurred while inserting envelope into database: {err}");
                MessageManagerError::ServiceUnavailable
            }
        })?;

        if result.rows_affected() != 1 {
            error!("Failed to store a message in the database")
        }
        if let Some(sender) = self
            .subscribers
            .lock()
            .await
            .get(&DeviceAddress::new(account_id, device_id))
        {
            sender
                .send(envelope_id)
                .await
                .inspect_err(|e| debug!("{e}"))
                .map_err(|_| MessageManagerError::MessageSubscriberSendError)?;
        }

        Ok(())
    }

    async fn get_envelope(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<ServerEnvelope, MessageManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        let env = envelope_id.uuid();
        let decode_result = sqlx::query!(
            r#"
            SELECT msg_queue.msg
            FROM msg_queue
            INNER JOIN devices on devices.id = msg_queue.receiver
            WHERE devices.owner = 
                (SELECT id
                 FROM accounts
                 WHERE account_id = $1)
              AND devices.device_id = $2
              AND msg_queue.envelope_id = $3
            "#,
            aci,
            dev,
            env
        )
        .fetch_one(&self.pool)
        .await
        .map(|rec| ServerEnvelope::decode(rec.msg.as_slice()))
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => {
                debug!("Attempted to fetch an envelope that does not exist.");
                MessageManagerError::EnvelopeDoesNotExists
            }
            _ => {
                error!("An error occurred while searching for a message in the database: {err}");
                MessageManagerError::ServiceUnavailable
            }
        })?;
        decode_result.map_err(|_| {
            error!("Could not decode envelope {} for {}.{}.", env, aci, dev);
            MessageManagerError::ServiceUnavailable
        })
    }

    async fn remove_envelope(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), MessageManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        let env = envelope_id.uuid();
        let result = sqlx::query!(
            r#"
            DELETE FROM msg_queue
            WHERE msg_queue.receiver =
            (SELECT id
             FROM devices
             WHERE device_id = $1
                   AND owner = 
                    (SELECT id
                     FROM accounts
                     WHERE account_id = $2)
            ) AND msg_queue.envelope_id = $3
            "#,
            dev,
            aci,
            env
        )
        .execute(&self.pool)
        .await
        .map_err(|err| match err {
            _ => {
                error!("An error occurred while deleting an envelope from the database: {err}");
                MessageManagerError::ServiceUnavailable
            }
        })?;

        if result.rows_affected() != 1 {
            debug!(
                "Wrong number of rows affected ({}) while trying to remove envelope.",
                result.rows_affected()
            );
        }

        Ok(())
    }

    async fn get_envelope_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<EnvelopeId>, MessageManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        sqlx::query!(
            r#"
            SELECT msg_queue.envelope_id
            FROM msg_queue
            INNER JOIN devices on devices.id = msg_queue.receiver
            WHERE devices.owner = 
                (SELECT id
                 FROM accounts
                 WHERE account_id = $1)
              AND devices.device_id = $2
            "#,
            aci,
            dev,
        )
        .fetch_all(&self.pool)
        .await
        .map(|recs| recs.iter().map(|rec| rec.envelope_id.into()).collect())
        .map_err(|err| match err {
            _ => {
                error!("An error occurred while deleting an envelope from the database: {err}");
                MessageManagerError::ServiceUnavailable
            }
        })
    }

    async fn subscribe(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<mpsc::Receiver<EnvelopeId>, MessageManagerError> {
        let key = DeviceAddress::new(account_id, device_id);
        let (sender, receiver) = mpsc::channel(self.channel_buffer);

        if self.subscribers.lock().await.contains_key(&key) {
            return Err(MessageManagerError::MessageSubscriberAlreadyExists)?;
        }

        let _ = self.subscribers.lock().await.insert(key, sender);
        Ok(receiver)
    }

    async fn unsubscribe(&mut self, account_id: AccountId, device_id: DeviceId) {
        let key = DeviceAddress::new(account_id, device_id);

        if !self.subscribers.lock().await.contains_key(&key) {
            return;
        }

        self.subscribers.lock().await.remove(&key);
    }

    async fn dispatch_envelopes(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), MessageManagerError> {
        let ids = self.get_envelope_ids(account_id, device_id).await?;
        if ids.len() > 0 {
            let key = DeviceAddress::new(account_id, device_id);

            match self.subscribers.lock().await.get(&key) {
                Some(sender) => {
                    for id in ids {
                        sender
                            .send(id)
                            .await
                            .inspect_err(|e| debug!("{e}"))
                            .map_err(|_| MessageManagerError::MessageSubscriberSendError)?;
                    }
                    Ok(())
                }
                None => Err(MessageManagerError::MessageSubscriberDoesNotExists)?,
            }
        } else {
            Ok(())
        }
    }

    async fn add_pending_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), MessageManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        let env = envelope_id.uuid();
        let result = sqlx::query!(
            r#"
            UPDATE msg_queue
            SET acknowledged = FALSE
            WHERE msg_queue.receiver =
            (SELECT id
             FROM devices
             WHERE device_id = $1
                   AND owner = 
                    (SELECT id
                     FROM accounts
                     WHERE account_id = $2)
            ) AND msg_queue.envelope_id = $3
            "#,
            dev,
            aci,
            env
        )
        .execute(&self.pool)
        .await
        .map_err(|err| match err {
            _ => {
                error!("An error occurred while deleting an envelope from the database: {err}");
                MessageManagerError::ServiceUnavailable
            }
        })?;

        if result.rows_affected() != 1 {
            debug!(
                "Wrong number of rows affected ({}) while trying to mark envelope as pending.",
                result.rows_affected()
            );
        }

        Ok(())
    }

    async fn remove_pending_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), MessageManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        let env = envelope_id.uuid();
        let result = sqlx::query!(
            r#"
            UPDATE msg_queue
            SET acknowledged = TRUE 
            WHERE msg_queue.receiver =
            (SELECT id
                FROM devices
                WHERE device_id = $1
                    AND owner = 
                    (SELECT id
                        FROM accounts
                        WHERE account_id = $2)
            ) AND msg_queue.envelope_id = $3
            "#,
            dev,
            aci,
            env
        )
        .execute(&self.pool)
        .await
        .map_err(|err| match err {
            _ => {
                error!("An error occurred while deleting an envelope from the database: {err}");
                MessageManagerError::ServiceUnavailable
            }
        })?;

        if result.rows_affected() != 1 {
            debug!(
                "Wrong number of rows affected ({}) while trying to mark envelope as acknowledged.",
                result.rows_affected()
            );
        }

        Ok(())
    }
}
