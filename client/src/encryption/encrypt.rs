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
    message: impl Into<Vec<u8>>,
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
            &bytes,
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

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use libsignal_core::ProtocolAddress;
    use libsignal_protocol::{
        kem, process_prekey_bundle, GenericSignedPreKey as _, IdentityKeyPair,
        IdentityKeyStore as _, KeyPair, KyberPreKeyRecord, KyberPreKeyStore as _, PreKeyBundle,
        PreKeyRecord, PreKeyStore as _, SignalProtocolError, SignedPreKeyRecord,
        SignedPreKeyStore as _, Timestamp,
    };
    use rand::{rngs::OsRng, CryptoRng, Rng};
    use sam_common::{address::RegistrationId, sam_message::ServerEnvelope, AccountId, DeviceId};

    use crate::{
        encryption::encrypt::{decrypt, encrypt},
        storage::{inmem::InMemoryStoreConfig, ContactStore, Store, StoreConfig, StoreType},
    };

    pub async fn create_pre_key_bundle<R: Rng + CryptoRng>(
        store: &mut Store<impl StoreType>,
        device_id: DeviceId,
        mut csprng: &mut R,
    ) -> Result<PreKeyBundle, SignalProtocolError> {
        // z is random
        let pre_key_pair = KeyPair::generate(&mut csprng); // OPK - only one but should be more -> publish

        let signed_pre_key_pair = KeyPair::generate(&mut csprng); // SPKB - changes periodically -> publish
        let kyber_pre_key_pair = kem::KeyPair::generate(kem::KeyType::Kyber1024); // PQSPKB - changes periodically -> publish

        let signed_pre_key_signature = store
            .identity_key_store // Sig(IKB, EncodeEC(SPKB), ZSPK) - changes periodically -> publish
            .get_identity_key_pair() // IKB - Bob only needs to upload his identity key to the server once -> publish
            .await?
            .private_key()
            .calculate_signature(&signed_pre_key_pair.public_key.serialize(), &mut csprng)?;

        let kyber_pre_key_signature = store
            .identity_key_store // Sig(IKB, EncodeKEM(PQSPKB), ZPQSPK) - changes periodically -> publish
            .get_identity_key_pair()
            .await?
            .private_key()
            .calculate_signature(&kyber_pre_key_pair.public_key.serialize(), &mut csprng)?;

        let pre_key_id: u32 = csprng.gen(); // IdEC(OPKB1) -> publish
        let signed_pre_key_id: u32 = csprng.gen(); // IdEC(SPKB) -> publish
        let kyber_pre_key_id: u32 = csprng.gen(); // IdKEM(PQSPKB) -> publish

        // <-- publish -->
        // one-time pqkem prekeys - these are not generated and should be, so users can verify integrity
        // should also generate signatures for each of the keys - (Sig(IKB, EncodeKEM(PQOPKB), Z1)
        // this can be used: kem::KeyPair::generate(kem::KeyType::Kyber1024)

        let pre_key_bundle = PreKeyBundle::new(
            store.identity_key_store.get_local_registration_id().await?, // the users unique id
            (*device_id).into(),
            Some((pre_key_id.into(), pre_key_pair.public_key)),
            signed_pre_key_id.into(),
            signed_pre_key_pair.public_key,
            signed_pre_key_signature.to_vec(),
            *store
                .identity_key_store
                .get_identity_key_pair()
                .await?
                .identity_key(),
        )?;
        let pre_key_bundle = pre_key_bundle.with_kyber_pre_key(
            kyber_pre_key_id.into(),
            kyber_pre_key_pair.public_key.clone(),
            kyber_pre_key_signature.to_vec(),
        );

        store
            .pre_key_store
            .save_pre_key(
                pre_key_id.into(),
                &PreKeyRecord::new(pre_key_id.into(), &pre_key_pair),
            )
            .await?;

        let timestamp = Timestamp::from_epoch_millis(csprng.gen());

        store
            .signed_pre_key_store
            .save_signed_pre_key(
                signed_pre_key_id.into(),
                &SignedPreKeyRecord::new(
                    signed_pre_key_id.into(),
                    timestamp,
                    &signed_pre_key_pair,
                    &signed_pre_key_signature,
                ),
            )
            .await?;

        store
            .kyber_pre_key_store
            .save_kyber_pre_key(
                kyber_pre_key_id.into(),
                &KyberPreKeyRecord::new(
                    kyber_pre_key_id.into(),
                    Timestamp::from_epoch_millis(43),
                    &kyber_pre_key_pair,
                    &kyber_pre_key_signature,
                ),
            )
            .await?;
        Ok(pre_key_bundle)
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
            .contact_store
            .add_device(bob, 1.into())
            .await
            .expect("Can add bobs device");

        let bob_bundle = create_pre_key_bundle(&mut bob_store, 1.into(), &mut csprng)
            .await
            .expect("Can create bob's pre key bundle");

        let _ = process_prekey_bundle(
            &ProtocolAddress::new(bob.to_string(), 1.into()),
            &mut alice_store.session_store,
            &mut alice_store.identity_key_store,
            &bob_bundle,
            SystemTime::now(),
            &mut csprng,
        )
        .await;

        let client_envelope = encrypt(my_struct.clone(), bob, &mut alice_store)
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

        let decrypted = decrypt(envelope, &mut bob_store)
            .await
            .expect("should be able to decrypt");

        let the_struct: MyStruct = decrypted.content().expect("Can deserialize struct");

        assert_eq!(my_struct, the_struct)
    }
}
