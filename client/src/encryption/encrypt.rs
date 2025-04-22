use std::{collections::HashMap, time::SystemTime};

use libsignal_core::ProtocolAddress;
use libsignal_protocol::{
    message_decrypt, message_encrypt, CiphertextMessage, PlaintextContent, PreKeySignalMessage,
    SenderKeyMessage, SignalMessage,
};
use log::debug;
use rand::{CryptoRng, Rng};
use sam_common::{
    sam_message::{ClientEnvelope, SamMessage, SamMessageType, ServerEnvelope},
    AccountId,
};

use crate::storage::{AccountStore, ContactStore, Store, StoreType};

use super::{
    envelope::DecryptedEnvelope,
    error::EncryptionError,
    padding::{pad_message, unpad_message},
};

/// Encrypt a message an put it into a [ClientEnvelope].
///
/// Anything that implements `Into<Vec<u8>>` can be encrypted. The message will be converted to
/// bytes, padded and then encrypted.
///
/// # Arguments
///
/// * `message` - The message to be encrypted.
/// * `recipient` - The [AccountId] of the contact that the message should be encrypted for.
/// * `store` - The [Store] containing Signal Protocol related artefacts required for encryption.
///
/// # Returns
///
/// * `Ok(ClientEnvelope)` The encrypted message.
/// * `Err(ClientError)` if encryption fails.
pub async fn encrypt(
    message: impl Into<Vec<u8>>,
    recipients: Vec<AccountId>,
    store: &mut Store<impl StoreType>,
) -> Result<ClientEnvelope, EncryptionError> {
    let bytes = pad_message(&message.into());

    let mut recipient_addrs = HashMap::new();
    let my_id = store.account_store.get_account_id().await?;
    let my_device_id = store.account_store.get_device_id().await?;

    for recipient in recipients {
        let mut devices = store.contact_store.get_all_devices(recipient).await?;
        if recipient == my_id {
            devices.retain(|id| *id != my_device_id);
        }
        recipient_addrs.insert(recipient, devices);
    }

    let addr_len = recipient_addrs.values().map(|v| v.len()).sum();
    let mut messages = Vec::with_capacity(addr_len);
    for (recipient, addresses) in recipient_addrs {
        debug!(
            "Encrypting for recipient '{recipient}' devices '{:?}'",
            addresses
        );
        for device_id in addresses {
            let addr = ProtocolAddress::new(recipient.to_string(), (*device_id).into());
            let message = message_encrypt(
                &bytes,
                &addr,
                &mut store.session_store,
                &mut store.identity_key_store,
                SystemTime::now(),
            )
            .await?;

            messages.push(
                SamMessage::builder()
                    .r#type(Into::<SamMessageType>::into(message.message_type()).into())
                    .destination_account_id(recipient.into_bytes().to_vec())
                    .destination_device_id(addr.device_id().into())
                    .content(message.serialize().to_vec())
                    .build(),
            );
        }
    }

    Ok(ClientEnvelope::builder().messages(messages).build())
}

/// Decrypt a message an put it into a [ClientEnvelope].
///
/// Anything that implements `From<Vec<u8>>` can be decrypt. The message will be converted to
/// bytes, decrypted and unpadded.
///
/// # Arguments
///
/// * `envelope` - The message to be decrypted.
/// * `store` - The [Store] containing Signal Protocol related artefacts required for decryption.
///
/// # Returns
///
/// * `Ok(DecryptedEnvelope)` The an envelope type containing the decrypted message.
/// * `Err(ClientError)` if decryption fails.
pub async fn decrypt<T: Rng + CryptoRng + Default>(
    envelope: ServerEnvelope,
    store: &mut Store<impl StoreType>,
    rng: &mut T,
) -> Result<DecryptedEnvelope, EncryptionError> {
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
        .inspect_err(|e| debug!("{e}"))
        .map_err(|_| EncryptionError::InvalidAccountId("Could not parse bytes".to_owned()))?;
    debug!("Decrypting message from '{source}'");
    let bytes = unpad_message(
        &message_decrypt(
            &message,
            &ProtocolAddress::new(source.to_string(), envelope.source_device_id.into()),
            &mut store.session_store,
            &mut store.identity_key_store,
            &mut store.pre_key_store,
            &store.signed_pre_key_store,
            &mut store.kyber_pre_key_store,
            rng,
        )
        .await?,
    )
    .ok_or(EncryptionError::FailedToUnpadMessage)?;

    Ok(DecryptedEnvelope::builder()
        .source_account_id(source)
        .source_device_id(envelope.source_device_id.into())
        .content(bytes)
        .build())
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use libsignal_core::ProtocolAddress;
    use libsignal_protocol::{
        process_prekey_bundle, IdentityKeyPair, IdentityKeyStore, SignalProtocolError,
    };
    use rand::{rngs::OsRng, CryptoRng, Rng};
    use sam_common::{
        address::RegistrationId, api::PreKeyBundle, sam_message::ServerEnvelope, AccountId,
        DeviceId,
    };

    use crate::{
        encryption::encrypt::{decrypt, encrypt},
        logic::into_libsignal_bundle,
        storage::{
            inmem::InMemoryStoreConfig,
            key_generation::{KyberKeyGenerator, PreKeyGenerator, SignedPreKeyGenerator},
            AccountStore, ContactStore, Store, StoreConfig, StoreType,
        },
    };

    pub async fn create_pre_key_bundle<R: Rng + CryptoRng>(
        store: &mut Store<impl StoreType>,
        device_id: DeviceId,
        csprng: &mut R,
    ) -> Result<PreKeyBundle, SignalProtocolError> {
        let pair = store
            .identity_key_store
            .get_identity_key_pair()
            .await
            .expect("Can get identity");
        Ok(PreKeyBundle {
            device_id: *device_id,
            registration_id: store
                .identity_key_store
                .get_local_registration_id()
                .await
                .expect("Can get reg id"),
            pre_key: Some(
                store
                    .pre_key_store
                    .generate_key(csprng)
                    .await
                    .expect("Can create pre key")
                    .into(),
            ),
            pq_pre_key: store
                .kyber_pre_key_store
                .generate_key(pair.private_key())
                .await
                .expect("Can create pq")
                .into(),
            signed_pre_key: store
                .signed_pre_key_store
                .generate_key(csprng, pair.private_key())
                .await
                .expect("can create signed pre key")
                .into(),
        })
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct MyStruct {
        string: String,
    }

    impl From<MyStruct> for Vec<u8> {
        fn from(val: MyStruct) -> Self {
            val.string.into_bytes()
        }
    }

    impl From<Vec<u8>> for MyStruct {
        fn from(value: Vec<u8>) -> Self {
            Self {
                string: String::from_utf8(value).expect("Can deserialize struct"),
            }
        }
    }

    #[tokio::test]
    async fn encrypt_and_decrypt_message() {
        let mut csprng = OsRng;
        let alice_key_pair = IdentityKeyPair::generate(&mut csprng);
        let alice_registration_id = RegistrationId::generate(&mut csprng);
        let mut alice_store = InMemoryStoreConfig::default()
            .create_store(alice_key_pair, alice_registration_id)
            .await
            .expect("Can create alice store");

        let bob_key_pair = IdentityKeyPair::generate(&mut csprng);
        let bob_registration_id = RegistrationId::generate(&mut csprng);

        let mut bob_store = InMemoryStoreConfig::default()
            .create_store(bob_key_pair, bob_registration_id)
            .await
            .expect("Can create bob store");
        let bob = AccountId::generate();
        let alice = AccountId::generate();

        let my_struct = MyStruct {
            string: "Hello, World!".to_owned(),
        };
        alice_store
            .account_store
            .set_account_id(alice)
            .await
            .expect("Can add self account id");
        alice_store
            .account_store
            .set_device_id(1.into())
            .await
            .expect("can add self device id");
        alice_store
            .contact_store
            .add_device(bob, 1.into())
            .await
            .expect("Can add bobs device");

        let bob_bundle = create_pre_key_bundle(&mut bob_store, 1.into(), &mut csprng)
            .await
            .expect("Can create bob's pre key bundle");

        let id_pair = bob_store
            .identity_key_store
            .get_identity_key_pair()
            .await
            .expect("Can get bob idenity");

        let bob_identity = id_pair.identity_key();
        let signal_bundle =
            into_libsignal_bundle(bob_bundle, *bob_identity).expect("Can create signal bunlde");

        let _ = process_prekey_bundle(
            &ProtocolAddress::new(bob.to_string(), 1.into()),
            &mut alice_store.session_store,
            &mut alice_store.identity_key_store,
            &signal_bundle,
            SystemTime::now(),
            &mut csprng,
        )
        .await;

        let client_envelope = encrypt(my_struct.clone(), vec![bob], &mut alice_store)
            .await
            .expect("Can encrypt message");

        let message = client_envelope
            .messages
            .first()
            .expect("should contain one message");

        let envelope = ServerEnvelope::builder()
            .id(vec![])
            .source_device_id(1)
            .source_account_id(alice.into_bytes().to_vec())
            .destination_device_id(1)
            .destination_account_id(bob.into_bytes().to_vec())
            .r#type(message.r#type)
            .content(message.content.clone())
            .build();

        let decrypted = decrypt(envelope, &mut bob_store, &mut csprng)
            .await
            .expect("should be able to decrypt");

        let the_struct: MyStruct = decrypted.content().expect("Can deserialize struct");

        assert_eq!(my_struct, the_struct)
    }
}
