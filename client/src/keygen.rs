use crate::signal_time_now;
use crate::storage::{ProvidesKeyId, Store, StoreType};
use crate::ClientError;
use async_trait::async_trait;
use libsignal_protocol::kem::KeyType;
use libsignal_protocol::{
    GenericSignedPreKey, IdentityKeyStore, KeyPair, KyberPreKeyRecord, KyberPreKeyStore,
    PreKeyRecord, PreKeyStore, SignedPreKeyRecord, SignedPreKeyStore,
};
use rand::{CryptoRng, Rng};

#[derive(Debug)]
pub struct PreKeyCollection {
    pub signed_pre_key: SignedPreKeyRecord,
    pub pq_last_resort_pre_key: KyberPreKeyRecord,
    pub pre_keys: Vec<PreKeyRecord>,
    pub pq_pre_keys: Vec<KyberPreKeyRecord>,
}

#[async_trait(?Send)]
pub trait KeyManager {
    async fn generate_pre_key<R: Rng + CryptoRng>(
        &mut self,
        csprng: &mut R,
    ) -> Result<PreKeyRecord, ClientError>;

    async fn generate_signed_pre_key<R: Rng + CryptoRng>(
        &mut self,
        csprng: &mut R,
    ) -> Result<SignedPreKeyRecord, ClientError>;

    async fn generate_kyber_pre_key(&mut self) -> Result<KyberPreKeyRecord, ClientError>;

    async fn generate_key_bundle<R: Rng + CryptoRng>(
        &mut self,
        csprng: &mut R,
    ) -> Result<PreKeyCollection, ClientError>;
}

#[async_trait(?Send)]
impl<T: StoreType> KeyManager for Store<T> {
    async fn generate_pre_key<R>(&mut self, csprng: &mut R) -> Result<PreKeyRecord, ClientError>
    where
        R: Rng + CryptoRng,
    {
        let id = self.pre_key_store.next_key_id().await?;

        let key_pair = KeyPair::generate(csprng);
        let record = PreKeyRecord::new(id, &key_pair);
        self.pre_key_store.save_pre_key(id, &record).await?;
        Ok(record)
    }

    async fn generate_signed_pre_key<R>(
        &mut self,
        csprng: &mut R,
    ) -> Result<SignedPreKeyRecord, ClientError>
    where
        R: Rng + CryptoRng,
    {
        let id = self.signed_pre_key_store.next_key_id().await?;
        let signed_pre_key_pair = KeyPair::generate(csprng);
        let signature = self
            .identity_key_store
            .get_identity_key_pair()
            .await?
            .private_key()
            .calculate_signature(&signed_pre_key_pair.public_key.serialize(), csprng)?;

        let record =
            SignedPreKeyRecord::new(id, signal_time_now(), &signed_pre_key_pair, &signature);

        self.signed_pre_key_store
            .save_signed_pre_key(id, &record)
            .await?;

        Ok(record)
    }

    async fn generate_kyber_pre_key(&mut self) -> Result<KyberPreKeyRecord, ClientError> {
        let id = self.kyber_pre_key_store.next_key_id().await?;
        let record = KyberPreKeyRecord::generate(
            KeyType::Kyber1024,
            id,
            self.identity_key_store
                .get_identity_key_pair()
                .await?
                .private_key(),
        )?;

        self.kyber_pre_key_store
            .save_kyber_pre_key(id, &record)
            .await?;
        Ok(record)
    }

    async fn generate_key_bundle<R>(
        &mut self,
        mut csprng: &mut R,
    ) -> Result<PreKeyCollection, ClientError>
    where
        R: Rng + CryptoRng,
    {
        let mut pre_keys: Vec<PreKeyRecord> = Vec::new();
        let mut pq_pre_keys: Vec<KyberPreKeyRecord> = Vec::new();

        for _ in 0..100 {
            pre_keys.push(self.generate_pre_key(&mut csprng).await?);

            pq_pre_keys.push(self.generate_kyber_pre_key().await?);
        }

        let signed_pre_key = self.generate_signed_pre_key(&mut csprng).await?;
        let pq_last_resort_pre_key = self.generate_kyber_pre_key().await?;

        Ok(PreKeyCollection {
            pre_keys,
            signed_pre_key,
            pq_pre_keys,
            pq_last_resort_pre_key,
        })
    }
}
