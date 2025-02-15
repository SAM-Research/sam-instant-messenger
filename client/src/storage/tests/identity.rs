use crate::storage::sqlite::{connect_to_in_memory, identity::SqliteIdentityKeyStore};
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
                    let alice_key_pair = key_pair();
                    let identity_key_store = $factory(alice_key_pair).await;

                    assert_eq!(
                        identity_key_store
                            .get_identity(&bob_address())
                            .await
                            .expect("should be able to retrieve None"),
                        None
                    );
                }

                /// Test own identity exists in new store
                #[tokio::test]
                async fn [< $struct _own_identity_in_new_store >]() {
                    let alice_key_pair = key_pair();
                    let identity_key_store = $factory(alice_key_pair).await;

                    assert_eq!(
                        identity_key_store
                            .get_identity_key_pair()
                            .await
                            .expect("should be able to retrieve own ID key pair")
                            .public_key(),
                        alice_key_pair.public_key()
                    );
                }

                /// Test that a new identity have been added
                #[tokio::test]
                async fn [< $struct _save_identity >]() {
                    let alice_key_pair = key_pair();
                    let mut identity_key_store = $factory(alice_key_pair).await;
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

                #[tokio::test]
                async fn [< $struct _save_identity_returns_false_if_key_is_known >]() {
                    let alice_key_pair = key_pair();
                    let mut identity_key_store = $factory(alice_key_pair).await;
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

                #[tokio::test]
                async fn [< $struct _save_identity_returns_false_if_key_overwritten >]() {
                    let alice_key_pair = key_pair();
                    let mut identity_key_store = $factory(alice_key_pair).await;
                    let bob_address = ProtocolAddress::new("bob".to_owned(), 0.into());
                    let bob_key_pair = IdentityKeyPair::generate(&mut OsRng);

                    assert!(!identity_key_store
                        .save_identity(&bob_address, bob_key_pair.identity_key())
                        .await
                        .expect("can attempt to store identity key"));

                    assert!(identity_key_store
                        .save_identity(&bob_address, alice_key_pair.identity_key())
                        .await
                        .expect("can attempt to store identity key"));

                    assert_eq!(
                        identity_key_store
                            .get_identity(&bob_address)
                            .await
                            .expect("can retrive a saved identity key")
                            .expect("the retrived identity key should exist"),
                        *alice_key_pair.identity_key()
                    );
                }

                #[tokio::test]
                async fn [< $struct _identity_is_trusted_on_first_use >]() {
                    let alice_key_pair = key_pair();
                    let mut identity_key_store = $factory(alice_key_pair).await;
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

                #[tokio::test]
                async fn [< $struct _identity_is_not_trusted_before_first_use >]() {
                    let alice_key_pair = key_pair();
                    let mut identity_key_store = $factory(alice_key_pair).await;
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
            }
        )*
    };
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
