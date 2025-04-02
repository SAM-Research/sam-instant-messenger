use crate::storage::{ProvidesKeyId, SamStore, SamStoreType};
use crate::{signal_time_now, ClientError};
use async_trait::async_trait;
use libsignal_core::curve::{KeyPair, PrivateKey};
use libsignal_protocol::kem::{self, KeyType};
use libsignal_protocol::{
    GenericSignedPreKey, IdentityKey, IdentityKeyPair, KyberPreKeyId, KyberPreKeyRecord,
    KyberPreKeyStore, PreKeyBundle, PreKeyId, PreKeyRecord, PreKeyStore, PublicKey, SignedPreKeyId,
    SignedPreKeyRecord, SignedPreKeyStore,
};

use rand::{CryptoRng, Rng};
use sam_common::api::keys::RegistrationPreKeys;
use sam_common::api::{EcPreKey, PqPreKey};

use super::{SignalStore, SignalStoreType};

#[async_trait(?Send)]
pub trait PreKeyGenerator {
    async fn generate_key<R: Rng + CryptoRng>(
        &mut self,
        csprng: &mut R,
    ) -> Result<PreKeyRecord, ClientError>;
}

#[async_trait(?Send)]
impl<T: PreKeyStore + ProvidesKeyId<PreKeyId>> PreKeyGenerator for T {
    async fn generate_key<R>(&mut self, csprng: &mut R) -> Result<PreKeyRecord, ClientError>
    where
        R: Rng + CryptoRng,
    {
        let id = self.next_key_id().await?;

        let key_pair = KeyPair::generate(csprng);
        let record = PreKeyRecord::new(id, &key_pair);
        self.save_pre_key(id, &record).await?;
        Ok(record)
    }
}

#[async_trait(?Send)]
pub trait SignedPreKeyGenerator {
    async fn generate_key<R: Rng + CryptoRng>(
        &mut self,
        csprng: &mut R,
        private_key: &PrivateKey,
    ) -> Result<SignedPreKeyRecord, ClientError>;
}

#[async_trait(?Send)]
impl<T: SignedPreKeyStore + ProvidesKeyId<SignedPreKeyId>> SignedPreKeyGenerator for T {
    async fn generate_key<R>(
        &mut self,
        csprng: &mut R,
        private_key: &PrivateKey,
    ) -> Result<SignedPreKeyRecord, ClientError>
    where
        R: Rng + CryptoRng,
    {
        let id = self.next_key_id().await?;
        let signed_pre_key_pair = KeyPair::generate(csprng);
        let signature =
            private_key.calculate_signature(&signed_pre_key_pair.public_key.serialize(), csprng)?;

        let record =
            SignedPreKeyRecord::new(id, signal_time_now(), &signed_pre_key_pair, &signature);

        self.save_signed_pre_key(id, &record).await?;

        Ok(record)
    }
}

#[async_trait(?Send)]
pub trait KyberKeyGenerator {
    async fn generate_key(
        &mut self,
        private_key: &PrivateKey,
    ) -> Result<KyberPreKeyRecord, ClientError>;
}

#[async_trait(?Send)]
impl<T: KyberPreKeyStore + ProvidesKeyId<KyberPreKeyId>> KyberKeyGenerator for T {
    async fn generate_key(
        &mut self,
        private_key: &PrivateKey,
    ) -> Result<KyberPreKeyRecord, ClientError> {
        let id = self.next_key_id().await?;
        let record = KyberPreKeyRecord::generate(KeyType::Kyber1024, id, private_key)?;

        self.save_kyber_pre_key(id, &record).await?;
        Ok(record)
    }
}

pub async fn generate_ec_pre_keys<G: PreKeyGenerator, R: Rng + CryptoRng>(
    generator: &mut G,
    amount: usize,
    mut csprng: &mut R,
) -> Result<Vec<EcPreKey>, ClientError> {
    let mut keys = Vec::with_capacity(amount);
    for _ in 0..amount {
        keys.push(generator.generate_key(&mut csprng).await?.into());
    }
    Ok(keys)
}

pub async fn generate_pq_pre_keys<G: KyberKeyGenerator>(
    signing_key: &PrivateKey,
    generator: &mut G,
    amount: usize,
) -> Result<Vec<PqPreKey>, ClientError> {
    let mut keys = Vec::with_capacity(amount);
    for _ in 0..amount {
        keys.push(generator.generate_key(signing_key).await?.into());
    }
    Ok(keys)
}

pub(crate) fn into_libsignal_bundle(
    bundle: sam_common::api::PreKeyBundle,
    identity_key: IdentityKey,
) -> Result<PreKeyBundle, ClientError> {
    Ok(PreKeyBundle::new(
        bundle.registration_id,
        bundle.device_id.into(),
        match bundle.pre_key {
            None => None,
            Some(key) => Some((key.key_id.into(), PublicKey::deserialize(&key.public_key)?)),
        },
        bundle.signed_pre_key.key_id.into(),
        PublicKey::deserialize(&bundle.signed_pre_key.public_key)?,
        bundle.signed_pre_key.signature.to_vec(),
        identity_key,
    )?
    .with_kyber_pre_key(
        bundle.pq_pre_key.key_id.into(),
        kem::PublicKey::deserialize(&bundle.pq_pre_key.public_key)?,
        bundle.pq_pre_key.signature.to_vec(),
    ))
}

pub async fn create_registration_pre_keys<S: SignalStoreType, R: Rng + CryptoRng>(
    store: &mut SignalStore<S>,
    prekey_count: usize,
    id_key_pair: IdentityKeyPair,
    mut csprng: &mut R,
) -> Result<RegistrationPreKeys, ClientError> {
    Ok(RegistrationPreKeys {
        pre_keys: Some(
            generate_ec_pre_keys(&mut store.pre_key_store, prekey_count, &mut csprng).await?,
        ),
        signed_pre_key: store
            .signed_pre_key_store
            .generate_key(&mut csprng, id_key_pair.private_key())
            .await?
            .into(),
        pq_pre_keys: Some(
            generate_pq_pre_keys(
                id_key_pair.private_key(),
                &mut store.kyber_pre_key_store,
                prekey_count,
            )
            .await?,
        ),
        pq_last_resort_pre_key: store
            .kyber_pre_key_store
            .generate_key(id_key_pair.private_key())
            .await?
            .into(),
    })
}

#[cfg(test)]
pub mod test {
    use super::*;
    use libsignal_protocol::{
        IdentityKeyPair, InMemKyberPreKeyStore, InMemPreKeyStore, InMemSignedPreKeyStore,
    };
    use rand::rngs::OsRng;

    #[tokio::test]
    async fn inmem_pre_key_generation() {
        let mut inmem_keystore = InMemPreKeyStore::new();
        let mut rng = OsRng;
        let returned_key = inmem_keystore.generate_key(&mut rng).await.unwrap();
        let stored_key = inmem_keystore
            .get_pre_key(returned_key.id().unwrap())
            .await
            .unwrap();

        assert_eq!(
            returned_key.public_key().unwrap(),
            stored_key.public_key().unwrap()
        );
    }

    #[tokio::test]
    async fn inmem_signed_pre_key_generation() {
        let mut inmem_keystore = InMemSignedPreKeyStore::new();
        let mut rng = OsRng;
        let identity_key_pair = IdentityKeyPair::generate(&mut rng);
        let returned_key = inmem_keystore
            .generate_key(&mut rng, identity_key_pair.private_key())
            .await
            .unwrap();
        let stored_key = inmem_keystore
            .get_signed_pre_key(returned_key.id().unwrap())
            .await
            .unwrap();

        assert_eq!(
            returned_key.public_key().unwrap(),
            stored_key.public_key().unwrap()
        );
        assert_eq!(
            returned_key.signature().unwrap(),
            stored_key.signature().unwrap()
        );
    }

    #[tokio::test]
    async fn inmem_kyber_pre_key_generation() {
        let mut inmem_keystore = InMemKyberPreKeyStore::new();
        let identity_key_pair = IdentityKeyPair::generate(&mut OsRng);
        let returned_key = inmem_keystore
            .generate_key(identity_key_pair.private_key())
            .await
            .unwrap();
        let stored_key = inmem_keystore
            .get_kyber_pre_key(returned_key.id().unwrap())
            .await
            .unwrap()
            .signature()
            .unwrap();

        assert_eq!(returned_key.signature().unwrap(), stored_key);
    }
}
