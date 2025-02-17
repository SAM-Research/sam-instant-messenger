use libsignal_protocol::IdentityKey;
use sam_common::api::keys::{Key, KeyBundle, KeyBundleResponse, PublishKeyBundle};
use uuid::Uuid;

use crate::{
    managers::traits::{
        account_manager::AccountManager,
        device_manager::DeviceManager,
        key_manager::{LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager},
    },
    state::{state_type::StateType, ServerState},
    ServerError,
};

pub async fn get_keybundle<T: StateType>(
    state: &mut ServerState<T>,
    account_id: Uuid,
    registration_id: u32,
    device_id: u32,
) -> Result<KeyBundle, ServerError> {
    let pre_key = state.keys.get_pre_key(account_id, device_id).await?;
    let pq_pre_key = state.keys.get_pq_pre_key(account_id, device_id).await?;
    let signed_pre_key = state.keys.get_signed_pre_key(account_id, device_id).await?;

    let pre_key = match pre_key {
        Some(key) => {
            state
                .keys
                .remove_pre_key(account_id, device_id, key.id())
                .await?;
            Some(key)
        }
        None => None,
    };
    let pq_pre_key = match pq_pre_key {
        Some(key) => {
            state
                .keys
                .remove_pq_pre_key(account_id, device_id, key.id())
                .await?;
            key
        }
        None => {
            state
                .keys
                .get_last_resort_key(account_id, device_id)
                .await?
        }
    };

    Ok(KeyBundle {
        device_id,
        registration_id,
        pre_key,
        pq_pre_key,
        signed_pre_key,
    })
}

pub async fn add_keybundle<T: StateType>(
    state: &mut ServerState<T>,
    identity: &IdentityKey,
    account_id: Uuid,
    device_id: u32,
    key_bundle: PublishKeyBundle,
) -> Result<(), ServerError> {
    if let Some(pre_keys) = key_bundle.pre_keys {
        for pre_key in pre_keys {
            state
                .keys
                .add_pre_key(account_id, device_id, pre_key)
                .await?;
        }
    }
    if let Some(key) = key_bundle.signed_pre_key {
        state
            .keys
            .set_signed_pre_key(account_id, device_id, identity, key)
            .await?;
    }

    if let Some(pre_keys) = key_bundle.pq_pre_keys {
        for pre_key in pre_keys {
            state
                .keys
                .add_pq_pre_key(account_id, device_id, identity, pre_key)
                .await?;
        }
    }

    if let Some(key) = key_bundle.pq_last_resort_pre_key {
        state
            .keys
            .set_last_resort_key(account_id, device_id, identity, key)
            .await?
    }
    Ok(())
}

pub async fn get_keybundles<T: StateType>(
    state: &mut ServerState<T>,
    account_id: Uuid,
) -> Result<KeyBundleResponse, ServerError> {
    let account = state.accounts.get_account(account_id).await?;
    let identity_key = account.identity();

    let devices = {
        let mut device_vec = vec![];
        for id in state.devices.get_devices(account_id).await? {
            let device = state.devices.get_device(account_id, id).await?;
            device_vec.push(device);
        }
        device_vec
    };

    let bundles = {
        let mut bundle_vec = vec![];
        for device in devices {
            bundle_vec.push(
                get_keybundle(state, account_id, device.registration_id(), device.id()).await?,
            );
        }
        bundle_vec
    };

    Ok(KeyBundleResponse {
        identity_key: *identity_key,
        bundles,
    })
}

pub async fn publish_keybundle<T: StateType>(
    state: &mut ServerState<T>,
    account_id: Uuid,
    device_id: u32,
    bundle: PublishKeyBundle,
) -> Result<(), ServerError> {
    let account = state.accounts.get_account(account_id).await?;
    let identity = account.identity();

    add_keybundle(state, identity, account_id, device_id, bundle).await
}

#[cfg(test)]
mod test {
    use libsignal_protocol::IdentityKeyPair;
    use rand::rngs::OsRng;
    use sam_common::api::Key;
    use uuid::Uuid;

    use crate::{
        auth::password::Password,
        logic::{
            keys::{add_keybundle, get_keybundle, get_keybundles, publish_keybundle},
            test_utils::create_publish_key_bundle,
        },
        managers::{
            entities::{account::Account, device::Device},
            in_memory::test_utils::LINK_SECRET,
            traits::{
                account_manager::AccountManager,
                device_manager::DeviceManager,
                key_manager::{
                    LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager,
                },
            },
        },
        state::ServerState,
    };

    #[tokio::test]
    async fn test_add_keybundle() {
        let mut state = ServerState::in_memory_default(LINK_SECRET.to_string());
        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);

        let account_id = Uuid::new_v4();

        let key_bundle = create_publish_key_bundle(
            Some(vec![1, 2]),
            Some(1),
            Some(vec![1, 2]),
            Some(33),
            &pair,
            rng,
        );

        add_keybundle(&mut state, pair.identity_key(), account_id, 1, key_bundle)
            .await
            .expect("User can create key bundle");
        assert!(state.keys.get_last_resort_key(account_id, 1).await.is_ok());
        assert!(state.keys.get_signed_pre_key(account_id, 1).await.is_ok());
        assert!(state.keys.get_pre_key_ids(account_id, 1).await.is_ok());
        assert!(state.keys.get_pq_pre_key_ids(account_id, 1).await.is_ok());
    }

    #[tokio::test]
    async fn test_get_keybundle() {
        let mut state = ServerState::in_memory_default(LINK_SECRET.to_string());
        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);

        let account_id = Uuid::new_v4();

        let key_bundle = create_publish_key_bundle(
            Some(vec![1, 2]),
            Some(22),
            Some(vec![1]),
            Some(33),
            &pair,
            rng,
        );

        add_keybundle(&mut state, pair.identity_key(), account_id, 1, key_bundle)
            .await
            .expect("User can create key bundle");

        // testing if we get keys
        let bundle = get_keybundle(&mut state, account_id, 1, 1)
            .await
            .expect("User have uploaded bundles");

        assert!(bundle.device_id == 1);
        assert!(bundle.registration_id == 1);
        assert!(bundle.pre_key.is_some_and(|k| k.id() == 1));
        assert!(bundle.signed_pre_key.id() == 22);
        assert!(bundle.pq_pre_key.id() == 1);
        // testing if we get last resort key
        let bundle = get_keybundle(&mut state, account_id, 1, 1)
            .await
            .expect("User have uploaded bundles");

        assert!(bundle.pq_pre_key.id() == 33)
    }

    #[tokio::test]
    async fn test_add_publish_keybundle() {
        let mut state = ServerState::in_memory_default(LINK_SECRET.to_string());
        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);

        let account = Account::builder()
            .id(Uuid::new_v4())
            .identity(*pair.identity_key())
            .username("Alice".to_string())
            .build();

        state
            .accounts
            .add_account(&account)
            .await
            .expect("Can add account");

        let key_bundle = create_publish_key_bundle(
            Some(vec![1, 2]),
            Some(1),
            Some(vec![1, 2]),
            Some(33),
            &pair,
            rng,
        );

        let account_id = *account.id();
        publish_keybundle(&mut state, account_id, 1, key_bundle)
            .await
            .expect("Alice can publish bundle");

        assert!(state.keys.get_last_resort_key(account_id, 1).await.is_ok());
        assert!(state.keys.get_signed_pre_key(account_id, 1).await.is_ok());
        assert!(state.keys.get_pre_key_ids(account_id, 1).await.is_ok());
        assert!(state.keys.get_pq_pre_key_ids(account_id, 1).await.is_ok());
    }

    #[tokio::test]
    async fn test_get_keybundles() {
        let mut state = ServerState::in_memory_default(LINK_SECRET.to_string());
        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);

        let account = Account::builder()
            .id(Uuid::new_v4())
            .identity(*pair.identity_key())
            .username("Alice".to_string())
            .build();

        state
            .accounts
            .add_account(&account)
            .await
            .expect("Can add account");

        let device = Device::builder()
            .id(1)
            .name("Alice Secret Phone".to_string())
            .password(Password::generate("dave<3".to_string()).expect("Alice can create password"))
            .creation(0)
            .registration_id(1)
            .build();

        let account_id = *account.id();
        state
            .devices
            .add_device(account_id, &device)
            .await
            .expect("Alice can add device");

        let key_bundle = create_publish_key_bundle(
            Some(vec![1, 2]),
            Some(22),
            Some(vec![1, 2]),
            Some(33),
            &pair,
            rng,
        );

        publish_keybundle(&mut state, account_id, 1, key_bundle)
            .await
            .expect("Alice can publish bundle");

        let bundles = get_keybundles(&mut state, account_id)
            .await
            .expect("User can get alices bundles");

        assert!(bundles.identity_key.serialize() == account.identity().serialize());
        assert!(bundles.bundles.len() == 1);

        let bundle = bundles.bundles.first().unwrap();
        assert!(bundle.device_id == 1);
        assert!(bundle.registration_id == 1);
        assert!(bundle.pre_key.clone().is_some_and(|k| k.id() == 1));
        assert!(bundle.signed_pre_key.id() == 22);
        assert!(bundle.pq_pre_key.id() == 1);
    }
}
