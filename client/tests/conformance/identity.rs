use libsignal_protocol::{Direction, IdentityKeyPair, IdentityKeyStore, ProtocolAddress};
use rand::rngs::OsRng;
use rstest::rstest;

use super::{bob_address, in_mem, key_pair, sqlite};

#[rstest]
#[case(in_mem().await.identity_key_store)]
#[case(sqlite().await.identity_key_store)]
#[tokio::test]
async fn no_identity_in_new_store(#[case] identity_key_store: impl IdentityKeyStore) {
    assert_eq!(
        identity_key_store
            .get_identity(&bob_address())
            .await
            .expect("should be able to retrieve None"),
        None
    );
}

#[rstest]
#[case(in_mem().await.identity_key_store)]
#[case(sqlite().await.identity_key_store)]
#[tokio::test]
async fn own_identity_in_new_store(#[case] identity_key_store: impl IdentityKeyStore) {
    assert!(identity_key_store.get_identity_key_pair().await.is_ok());
}

#[rstest]
#[case(in_mem().await.identity_key_store)]
#[case(sqlite().await.identity_key_store)]
#[tokio::test]
async fn save_and_retrieve_identity(#[case] mut identity_key_store: impl IdentityKeyStore) {
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

#[rstest]
#[case(in_mem().await.identity_key_store)]
#[case(sqlite().await.identity_key_store)]
#[tokio::test]
async fn save_identity_returns_false_if_key_is_known(
    #[case] mut identity_key_store: impl IdentityKeyStore,
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

#[rstest]
#[case(in_mem().await.identity_key_store)]
#[case(sqlite().await.identity_key_store)]
#[tokio::test]
async fn save_identity_returns_false_if_key_overwritten(
    #[case] mut identity_key_store: impl IdentityKeyStore,
) {
    let bob_address = ProtocolAddress::new("bob".to_owned(), 0.into());
    let bob_key_pair = IdentityKeyPair::generate(&mut OsRng);

    assert!(!identity_key_store
        .save_identity(&bob_address, bob_key_pair.identity_key())
        .await
        .expect("can attempt to store identity key"));

    let id_key_pair = IdentityKeyPair::generate(&mut OsRng);
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

#[rstest]
#[case(in_mem().await.identity_key_store)]
#[case(sqlite().await.identity_key_store)]
#[tokio::test]
async fn identity_is_trusted_on_first_use(#[case] mut identity_key_store: impl IdentityKeyStore) {
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

#[rstest]
#[case(in_mem().await.identity_key_store)]
#[case(sqlite().await.identity_key_store)]
#[tokio::test]
async fn identity_is_not_trusted_before_first_use(
    #[case] mut identity_key_store: impl IdentityKeyStore,
) {
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
