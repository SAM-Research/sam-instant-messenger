use sam_client::storage::sqlite::{connect_to_in_memory, identity::SqliteIdentityKeyStore};
use libsignal_protocol::{
    Direction, IdentityKeyPair, IdentityKeyStore, InMemIdentityKeyStore, ProtocolAddress,
};
use rand::rngs::OsRng;

use super::{bob_address, key_pair};

macro_rules! test_identity_key_store {
    ( [ $( ($struct:ty, $factory:expr) ),* ]) => {
        $(
            paste::paste! {
                #[tokio::test]
                async fn [< $struct _no_identity_in_new_store >]() {
                    no_identity_in_new_store($factory(key_pair()).await).await;
                }

                /// Test own identity exists in new store
                #[tokio::test]
                async fn [< $struct _own_identity_in_new_store >]() {
                    let alice_key_pair = key_pair();
                    own_identity_in_new_store(alice_key_pair, $factory(alice_key_pair).await).await;
                }

                /// Test that a new identity have been added
                #[tokio::test]
                async fn [< $struct _save_identity >]() {
                    save_and_retrieve_identity($factory(key_pair()).await).await;
                }

                #[tokio::test]
                async fn [< $struct _save_identity_returns_false_if_key_is_known >]() {
                    save_identity_returns_false_if_key_is_known($factory(key_pair()).await).await;
                }

                #[tokio::test]
                async fn [< $struct _save_identity_returns_false_if_key_overwritten >]() {
                    let alice_key_pair = key_pair();
                    save_identity_returns_false_if_key_overwritten(alice_key_pair, $factory(alice_key_pair).await).await;
                }

                #[tokio::test]
                async fn [< $struct _identity_is_trusted_on_first_use >]() {
                    identity_is_trusted_on_first_use($factory(key_pair()).await).await;
                }

                #[tokio::test]
                async fn [< $struct _identity_is_not_trusted_before_first_use >]() {
                    identity_is_not_trusted_before_first_use($factory(key_pair()).await).await;
                }
            }
        )*
    };
}

async fn no_identity_in_new_store(identity_key_store: impl IdentityKeyStore) {
    assert_eq!(
        identity_key_store
            .get_identity(&bob_address())
            .await
            .expect("should be able to retrieve None"),
        None
    );
}

async fn own_identity_in_new_store(
    alice_key_pair: IdentityKeyPair,
    identity_key_store: impl IdentityKeyStore,
) {
    assert_eq!(
        identity_key_store
            .get_identity_key_pair()
            .await
            .expect("should be able to retrieve own ID key pair")
            .public_key(),
        alice_key_pair.public_key()
    );
}

async fn save_and_retrieve_identity(mut identity_key_store: impl IdentityKeyStore) {
    let bob_address = ProtocolAddress::new("bob".to_owned(), 0.into());
    let bob_key_pair = IdentityKeyPair::generate(&mut OsRng);

    assert!(!identity_key_store
        .save_identity(&bob_address, bob_key_pair.identity_key())
        .await
        .expect("should be able to save an identity key"));

    assert_eq!(
        identity_key_store
            .get_identity(&bob_address)
            .await
            .expect("should retrive a saved identity key")
            .expect("the retrived identity key should exist"),
        *bob_key_pair.identity_key()
    );
}

async fn save_identity_returns_false_if_key_is_known(
    mut identity_key_store: impl IdentityKeyStore,
) {
    let bob_address = ProtocolAddress::new("bob".to_owned(), 0.into());
    let bob_key_pair = IdentityKeyPair::generate(&mut OsRng);

    assert!(!identity_key_store
        .save_identity(&bob_address, bob_key_pair.identity_key())
        .await
        .expect("should store can save an identity key"));

    assert!(!identity_key_store
        .save_identity(&bob_address, bob_key_pair.identity_key())
        .await
        .expect("should be able to attempt to store identity key"));

    assert_eq!(
        identity_key_store
            .get_identity(&bob_address)
            .await
            .expect("should retrive a saved identity key")
            .expect("the retrived identity key should exist"),
        *bob_key_pair.identity_key()
    );
}

async fn save_identity_returns_false_if_key_overwritten(
    id_key_pair: IdentityKeyPair,
    mut identity_key_store: impl IdentityKeyStore,
) {
    let bob_address = ProtocolAddress::new("bob".to_owned(), 0.into());
    let bob_key_pair = IdentityKeyPair::generate(&mut OsRng);

    assert!(!identity_key_store
        .save_identity(&bob_address, bob_key_pair.identity_key())
        .await
        .expect("can attempt to store identity key"));

    assert!(identity_key_store
        .save_identity(&bob_address, id_key_pair.identity_key())
        .await
        .expect("can attempt to store identity key"));

    assert_eq!(
        identity_key_store
            .get_identity(&bob_address)
            .await
            .expect("can retrive a saved identity key")
            .expect("the retrived identity key should exist"),
        *id_key_pair.identity_key()
    );
}

async fn identity_is_trusted_on_first_use(mut identity_key_store: impl IdentityKeyStore) {
    let bob_address = bob_address();
    let bob_key_pair = key_pair();
    // First use
    assert!(identity_key_store
        .is_trusted_identity(
            &bob_address,
            bob_key_pair.identity_key(),
            Direction::Sending
        )
        .await
        .unwrap());

    // Added identity
    identity_key_store
        .save_identity(&bob_address, bob_key_pair.identity_key())
        .await
        .unwrap();

    assert!(identity_key_store
        .is_trusted_identity(
            &bob_address,
            bob_key_pair.identity_key(),
            Direction::Sending
        )
        .await
        .unwrap());
}

async fn identity_is_not_trusted_before_first_use(mut identity_key_store: impl IdentityKeyStore) {
    let bob_address = bob_address();
    let bob_key_pair = key_pair();
    let charlie_address = ProtocolAddress::new("charlie".to_owned(), 0.into());
    let charlie_key_pair = key_pair();

    // Added identity
    identity_key_store
        .save_identity(&bob_address, bob_key_pair.identity_key())
        .await
        .unwrap();

    // Not trusted
    identity_key_store
        .save_identity(&charlie_address, charlie_key_pair.identity_key())
        .await
        .unwrap();

    assert!(!identity_key_store
        .is_trusted_identity(
            &bob_address,
            charlie_key_pair.identity_key(),
            Direction::Sending
        )
        .await
        .unwrap());
}

async fn sqlite(key_pair: IdentityKeyPair) -> SqliteIdentityKeyStore {
    let database = connect_to_in_memory().await;
    SqliteIdentityKeyStore::create(database, key_pair, 0u32)
        .await
        .unwrap()
}

async fn in_mem(key_pair: IdentityKeyPair) -> InMemIdentityKeyStore {
    InMemIdentityKeyStore::new(key_pair, 0u32)
}

test_identity_key_store!([
    (sqlite_identity_store, sqlite),
    (in_memory_identity_store, in_mem)
]);
