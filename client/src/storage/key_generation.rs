use crate::storage::ProvidesKeyId;
use crate::{signal_time_now, ClientError};
use async_trait::async_trait;
use libsignal_core::curve::KeyPair;
use libsignal_protocol::kem::KeyType;
use libsignal_protocol::{
    GenericSignedPreKey, IdentityKeyPair, KyberPreKeyId, KyberPreKeyRecord, KyberPreKeyStore,
    PreKeyId, PreKeyRecord, PreKeyStore, SignedPreKeyId, SignedPreKeyRecord, SignedPreKeyStore,
};
use rand::{CryptoRng, Rng};

#[async_trait(?Send)]
pub trait PreKeyGeneration<T> {
    async fn generate_key<R: Rng + CryptoRng>(&mut self, csprng: &mut R) -> Result<T, ClientError>;
}

#[async_trait(?Send)]
impl<T: PreKeyStore + ProvidesKeyId<PreKeyId>> PreKeyGeneration<PreKeyRecord> for T {
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
pub trait SignedPreKeyGeneration<T> {
    async fn generate_key<R: Rng + CryptoRng>(
        &mut self,
        csprng: &mut R,
        identity_key: IdentityKeyPair,
    ) -> Result<T, ClientError>;
}

#[async_trait(?Send)]
impl<T: SignedPreKeyStore + ProvidesKeyId<SignedPreKeyId>>
    SignedPreKeyGeneration<SignedPreKeyRecord> for T
{
    async fn generate_key<R>(
        &mut self,
        cspring: &mut R,
        identity_key_pair: IdentityKeyPair,
    ) -> Result<SignedPreKeyRecord, ClientError>
    where
        R: Rng + CryptoRng,
    {
        let id = self.next_key_id().await?;
        let signed_pre_key_pair = KeyPair::generate(cspring);
        let signature = identity_key_pair
            .private_key()
            .calculate_signature(&signed_pre_key_pair.public_key.serialize(), cspring)?;

        let record =
            SignedPreKeyRecord::new(id, signal_time_now(), &signed_pre_key_pair, &signature);

        self.save_signed_pre_key(id, &record).await?;

        Ok(record)
    }
}

#[async_trait(?Send)]
pub trait KyberKeyGeneration<T> {
    async fn generate_key(&mut self, identity_key_pair: IdentityKeyPair) -> Result<T, ClientError>;
}

#[async_trait(?Send)]
impl<T: KyberPreKeyStore + ProvidesKeyId<KyberPreKeyId>> KyberKeyGeneration<KyberPreKeyRecord>
    for T
{
    async fn generate_key(
        &mut self,
        identity_key_pair: IdentityKeyPair,
    ) -> Result<KyberPreKeyRecord, ClientError> {
        let id = self.next_key_id().await?;
        let record =
            KyberPreKeyRecord::generate(KeyType::Kyber1024, id, identity_key_pair.private_key())?;

        self.save_kyber_pre_key(id, &record).await?;
        Ok(record)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use libsignal_protocol::{InMemKyberPreKeyStore, InMemPreKeyStore, InMemSignedPreKeyStore};

    #[tokio::test]
    async fn inmem_pre_key_generation() {
        let mut inmem_keystore = InMemPreKeyStore::new();
        let mut rand = rand::thread_rng();
        let store = inmem_keystore.generate_key(&mut rand).await.unwrap();
        let key = inmem_keystore
            .get_pre_key(store.id().unwrap())
            .await
            .unwrap();

        assert_eq!(store.public_key().unwrap(), key.public_key().unwrap());
    }

    #[tokio::test]
    async fn inmem_signed_pre_key_generation() {
        let mut inmem_keystore = InMemSignedPreKeyStore::new();
        let mut rand = rand::thread_rng();
        let identity_key_pair = IdentityKeyPair::generate(&mut rand::thread_rng());
        let store = inmem_keystore
            .generate_key(&mut rand, identity_key_pair)
            .await
            .unwrap();
        let key = inmem_keystore
            .get_signed_pre_key(store.id().unwrap())
            .await
            .unwrap();

        assert_eq!(store.public_key().unwrap(), key.public_key().unwrap());
        assert_eq!(store.signature().unwrap(), key.signature().unwrap());
    }

    #[tokio::test]
    async fn inmem_kyber_pre_key_generation() {
        let mut inmem_keystore = InMemKyberPreKeyStore::new();
        let identity_key_pair = IdentityKeyPair::generate(&mut rand::thread_rng());
        let store = inmem_keystore
            .generate_key(identity_key_pair)
            .await
            .unwrap();
        let key_signature = inmem_keystore
            .get_kyber_pre_key(store.id().unwrap())
            .await
            .unwrap()
            .signature()
            .unwrap();

        assert_eq!(store.signature().unwrap(), key_signature);
    }
}
