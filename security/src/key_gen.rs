use libsignal_protocol::{
    kem::KeyType, GenericSignedPreKey, KeyPair, KyberPreKeyId, KyberPreKeyRecord, PreKeyId,
    PreKeyRecord, PrivateKey, SignalProtocolError, SignedPreKeyId, SignedPreKeyRecord,
};
use rand::{CryptoRng, Rng};
use sam_common::time::signal_time_now;

pub async fn generate_ec_pre_key<R: Rng + CryptoRng>(id: PreKeyId, csprng: &mut R) -> PreKeyRecord {
    let key_pair = KeyPair::generate(csprng);
    PreKeyRecord::new(id, &key_pair)
}

pub async fn generate_signed_pre_key<R: Rng + CryptoRng>(
    id: SignedPreKeyId,
    private_key: &PrivateKey,
    csprng: &mut R,
) -> Result<SignedPreKeyRecord, SignalProtocolError> {
    let signed_pre_key_pair = KeyPair::generate(csprng);
    let signature =
        private_key.calculate_signature(&signed_pre_key_pair.public_key.serialize(), csprng)?;

    Ok(SignedPreKeyRecord::new(
        id,
        signal_time_now(),
        &signed_pre_key_pair,
        &signature,
    ))
}

pub async fn generate_pq_pre_key(
    id: KyberPreKeyId,
    private_key: &PrivateKey,
) -> Result<KyberPreKeyRecord, SignalProtocolError> {
    KyberPreKeyRecord::generate(KeyType::Kyber1024, id, private_key)
}
