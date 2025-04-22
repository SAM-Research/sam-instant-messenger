use log::{debug, error};
use rand::{CryptoRng, Rng};
use sam_common::{
    sam_message::{ClientEnvelope, ServerEnvelope},
    AccountId,
};
use tokio::sync::mpsc::Receiver;

use crate::{
    encryption::{decrypt, encrypt},
    net::{protocol::MessageStatus, ApiClient},
    storage::{AccountStore, ContactStore, MessageStore, Store, StoreType},
};

use super::{key::fetch_prekeys, LogicError};

pub async fn process_messages<T: StoreType, R: Rng + CryptoRng + Default>(
    store: &mut Store<T>,
    envelope_queue: &mut Receiver<ServerEnvelope>,
    block: bool,
    rng: &mut R,
) -> Result<(), LogicError> {
    if !block && envelope_queue.is_empty() {
        return Ok(());
    }
    while let Some(envelope) = envelope_queue.recv().await {
        // TODO: How should we handle failure to decrypt and/or store message?
        process_message(envelope, store, rng)
            .await
            .inspect_err(|e| error!("Failed to store message {e}"))?;

        if envelope_queue.is_empty() {
            break;
        }
    }
    Ok(())
}

pub async fn process_message<T: Rng + CryptoRng + Default>(
    envelope: ServerEnvelope,
    store: &mut Store<impl StoreType>,
    rng: &mut T,
) -> Result<(), LogicError> {
    let envelope = decrypt(envelope, store, rng).await?;
    store
        .contact_store
        .add_device(envelope.source_account_id(), envelope.source_device_id())
        .await?;
    debug!("Processed Message from '{}'", envelope.source_account_id());
    store.message_store.store_message(envelope).await?;
    Ok(())
}

pub async fn prepare_message<T: StoreType, R: Rng + CryptoRng>(
    store: &mut Store<T>,
    api_client: &impl ApiClient,
    recipient: AccountId,
    msg: impl Into<Vec<u8>>,
    mut rng: &mut R,
) -> Result<ClientEnvelope, LogicError> {
    if !store.contact_store.contains_contact(recipient).await? {
        debug!("Unknown recipient '{recipient}', fetching keys...");
        fetch_prekeys(store, api_client, recipient, None, &mut rng).await?;
    }

    let my_id = store.account_store.get_account_id().await?;
    if !store.contact_store.contains_contact(my_id).await? {
        debug!("No Contact for self, fetching keys...");
        fetch_prekeys(store, api_client, my_id, None, &mut rng).await?;
    }
    let envelope = encrypt(msg, vec![recipient, my_id], store).await?;
    Ok(envelope)
}

pub async fn handle_message_response<T: StoreType, R: Rng + CryptoRng>(
    store: &mut Store<T>,
    api_client: &impl ApiClient,
    mut rng: &mut R,
    status: MessageStatus,
) -> Result<(), LogicError> {
    match status {
        MessageStatus::ExtraDevices(device_lists) => {
            debug!("Sent message contained extra devices, removing devices...");
            for list in device_lists {
                debug!(
                    "Removing devices '{:?}' from contact '{}'",
                    list.devices, list.account_id
                );
                for device in list.devices {
                    store
                        .contact_store
                        .remove_device(list.account_id, device)
                        .await?;
                }
            }
            Ok(())
        }
        MessageStatus::MissingDevices(device_lists) => {
            debug!("Sent message contained missing devices, fetching keys...");
            for list in device_lists {
                fetch_prekeys(
                    store,
                    api_client,
                    list.account_id,
                    Some(list.devices),
                    &mut rng,
                )
                .await?;
            }
            Err(LogicError::MissingDevices)
        }
        MessageStatus::Ok => Ok(()),
    }
}
