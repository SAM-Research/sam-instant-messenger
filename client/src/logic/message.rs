use log::error;
use rand::{CryptoRng, Rng};
use sam_common::{sam_message::ServerEnvelope, AccountId};
use tokio::sync::mpsc::Receiver;

use crate::{
    encryption::{decrypt, encrypt},
    net::{
        protocol::{MessageStatus, SamProtocolClient},
        ApiClient,
    },
    storage::{AccountStore, ContactStore, MessageStore, Store, StoreType},
    ClientError,
};

use super::key::fetch_prekeys;

pub async fn process_messages<T: StoreType>(
    store: &mut Store<T>,
    envelope_queue: &mut Receiver<ServerEnvelope>,
    block: bool,
) -> Result<(), ClientError> {
    if !block && envelope_queue.is_empty() {
        return Ok(());
    }
    while let Some(envelope) = envelope_queue.recv().await {
        // TODO: How should we handle failure to decrypt and/or store message?
        let envelope = match decrypt(envelope, store).await {
            Ok(denvelope) => denvelope,
            Err(e) => {
                error!("Failed to decrypt message: {e}");
                break;
            }
        };

        store
            .contact_store
            .add_device(envelope.source_account_id(), envelope.source_device_id())
            .await?;

        let _ = store
            .message_store
            .store_message(envelope)
            .await
            .inspect_err(|e| error!("Failed to store message {e}"));
        if envelope_queue.is_empty() {
            break;
        }
    }
    Ok(())
}

pub async fn send_message<T: StoreType, R: Rng + CryptoRng>(
    store: &mut Store<T>,
    api_client: &impl ApiClient,
    ws_client: &mut impl SamProtocolClient,
    recipient: AccountId,
    msg: impl Into<Vec<u8>>,
    mut rng: &mut R,
) -> Result<(), ClientError> {
    if !store.contact_store.contains_contact(recipient).await? {
        fetch_prekeys(store, api_client, recipient, None, &mut rng).await?;
    }

    let my_id = store.account_store.get_account_id().await?;
    if !store.contact_store.contains_contact(my_id).await? {
        fetch_prekeys(store, api_client, my_id, None, &mut rng).await?;
    }
    let envelope = encrypt(msg, vec![recipient, my_id], store).await?;
    let status = ws_client.send_message(envelope).await?;
    match status {
        MessageStatus::ExtraDevices(device_lists) => {
            for list in device_lists {
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
            Err(ClientError::MissingDevices)
        }
        MessageStatus::Ok => Ok(()),
    }
}
