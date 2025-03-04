use std::time::SystemTime;

use libsignal_core::ProtocolAddress;
use libsignal_protocol::{
    message_decrypt, message_encrypt, CiphertextMessage, PlaintextContent, PreKeySignalMessage,
    SenderKeyMessage, SignalMessage,
};
use rand::rngs::OsRng;
use sam_common::{
    sam_message::{ClientEnvelope, SamMessage, SamMessageType, ServerEnvelope},
    AccountId,
};

use crate::{
    storage::{ContactStore, Store, StoreType},
    ClientError,
};

use super::envelope::DecryptedEnvelope;

pub async fn encrypt(
    message: impl Into<&[u8]>,
    recipient: AccountId,
    store: &mut Store<impl StoreType>,
) -> Result<ClientEnvelope, ClientError> {
    let bytes = message.into();

    let addresses = store
        .contact_store
        .get_all_devices(recipient)
        .await?
        .into_iter()
        .map(|dev| ProtocolAddress::new(recipient.to_string(), (*dev).into()));

    let mut messages = Vec::with_capacity(addresses.len());

    for address in addresses {
        let message = message_encrypt(
            bytes,
            &address,
            &mut store.session_store,
            &mut store.identity_key_store,
            SystemTime::now(),
        )
        .await?;

        messages.push(
            SamMessage::builder()
                .r#type(Into::<SamMessageType>::into(message.message_type()).into())
                .destination_account_id(recipient.into_bytes().to_vec())
                .destination_device_id(address.device_id().into())
                .content(message.serialize().to_vec())
                .build(),
        );
    }

    Ok(ClientEnvelope::builder().messages(messages).build())
}

pub async fn decrypt(
    envelope: ServerEnvelope,
    store: &mut Store<impl StoreType>,
) -> Result<DecryptedEnvelope, ClientError> {
    let message = match envelope.r#type() {
        SamMessageType::SignalMessage => {
            CiphertextMessage::SignalMessage(SignalMessage::try_from(envelope.content.as_slice())?)
        }
        SamMessageType::PreKeySignalMessage => CiphertextMessage::PreKeySignalMessage(
            PreKeySignalMessage::try_from(envelope.content.as_slice())?,
        ),
        SamMessageType::SenderKeyMessage => CiphertextMessage::SenderKeyMessage(
            SenderKeyMessage::try_from(envelope.content.as_slice())?,
        ),
        SamMessageType::PlaintextContent => CiphertextMessage::PlaintextContent(
            PlaintextContent::try_from(envelope.content.as_slice())?,
        ),
    };

    let source = AccountId::try_from(envelope.source_account_id)
        .map_err(|_| ClientError::InvalidAccountId("Could not parse bytes".to_owned()))?;

    let bytes = message_decrypt(
        &message,
        &ProtocolAddress::new(source.to_string(), envelope.source_device_id.into()),
        &mut store.session_store,
        &mut store.identity_key_store,
        &mut store.pre_key_store,
        &store.signed_pre_key_store,
        &mut store.kyber_pre_key_store,
        &mut OsRng,
    )
    .await?;

    Ok(DecryptedEnvelope::builder()
        .source_account_id(source)
        .source_device_id(envelope.source_device_id.into())
        .content(bytes)
        .build())
}
