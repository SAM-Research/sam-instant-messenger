use sam_common::api::account::{RegistrationRequest, RegistrationResponse};
use uuid::Uuid;

use crate::{
    logic::device::create_device,
    managers::{
        entities::account::Account,
        traits::{
            account_manager::AccountManager,
            device_manager::DeviceManager,
            key_manager::{
                LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager,
            },
            message_manager::MessageManager,
        },
    },
    state::{state_type::StateType, ServerState},
    ServerError,
};

pub async fn delete_account<T: StateType>(
    state: &mut ServerState<T>,
    account_id: Uuid,
) -> Result<(), ServerError> {
    {
        for device_id in state.devices.get_devices(account_id).await? {
            if let Ok(msgs) = state.messages.get_envelope_ids(account_id, device_id).await {
                for msg_id in msgs {
                    state
                        .messages
                        .remove_envelope(account_id, device_id, msg_id)
                        .await?;
                }
            }

            for id in state.keys.get_pre_key_ids(account_id, device_id).await? {
                state.keys.remove_pre_key(account_id, device_id, id).await?
            }
            state
                .keys
                .remove_signed_pre_key(account_id, device_id)
                .await?;

            for id in state.keys.get_pq_pre_key_ids(account_id, device_id).await? {
                state
                    .keys
                    .remove_pq_pre_key(account_id, device_id, id)
                    .await?
            }

            state
                .keys
                .remove_last_resort_key(account_id, device_id)
                .await?;

            state.devices.remove_device(account_id, device_id).await?;
        }
    }

    state.accounts.remove_account(account_id).await
}

pub async fn create_account<T: StateType>(
    state: &mut ServerState<T>,
    registration: RegistrationRequest,
    username: String,
    password: String,
) -> Result<RegistrationResponse, ServerError> {
    let account = Account::builder()
        .id(Uuid::new_v4())
        .username(username)
        .identity(registration.identity_key)
        .build();

    state.accounts.add_account(&account).await?;

    create_device(
        state,
        *account.id(),
        account.identity(),
        registration.device_activation,
        1,
        password,
    )
    .await?;
    Ok(RegistrationResponse {
        account_id: *account.id(),
    })
}

#[cfg(test)]
mod test {
    use libsignal_protocol::IdentityKeyPair;
    use rand::rngs::OsRng;
    use sam_common::api::{device::DeviceActivationInfo, Key, RegistrationRequest};

    use crate::{
        logic::{
            account::{create_account, delete_account},
            test_utils::create_publish_key_bundle,
        },
        managers::{
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
    async fn test_create_account() {
        let mut state = ServerState::in_memory_default(LINK_SECRET.to_string());

        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);
        let id = pair.identity_key();
        let reg = RegistrationRequest {
            identity_key: *pair.identity_key(),
            device_activation: DeviceActivationInfo {
                name: "Alice Phone".to_string(),
                registration_id: 1,
                key_bundle: create_publish_key_bundle(
                    Some(vec![0]),
                    Some(1),
                    Some(vec![33]),
                    Some(2),
                    &pair,
                    rng,
                ),
            },
        };

        let alice_id = create_account(
            &mut state,
            reg,
            "RealAlice".to_string(),
            "bob<3".to_string(),
        )
        .await
        .map(|r| r.account_id)
        .expect("Alice can create account");

        // Check if account is created
        let account = state
            .accounts
            .get_account(alice_id)
            .await
            .expect("Alice has an account");

        assert!(*account.id() == alice_id);
        assert!(*account.identity() == *id);
        assert!(account.username() == "RealAlice");

        // Check if device is created
        let device = state
            .devices
            .get_device(alice_id, 1)
            .await
            .expect("Alice has primary device");

        assert!(device.registration_id() == 1);
        assert!(device.name() == "Alice Phone");
        device
            .password()
            .verify("bob<3".to_string())
            .expect("Alice loves bob");

        // check if keys are inserted

        let ec_key_ids = state.keys.get_pre_key_ids(alice_id, 1).await.unwrap();
        let signed_ec_id = state
            .keys
            .get_signed_pre_key(alice_id, 1)
            .await
            .unwrap()
            .id();

        assert!(ec_key_ids == vec![0]);
        assert!(signed_ec_id == 1);

        let pq_key_ids = state.keys.get_pq_pre_key_ids(alice_id, 1).await.unwrap();
        let last_resort_id = state
            .keys
            .get_last_resort_key(alice_id, 1)
            .await
            .unwrap()
            .id();

        assert!(pq_key_ids == vec![33]);
        assert!(last_resort_id == 2);
    }

    #[tokio::test]
    async fn test_delete_account() {
        let mut state = ServerState::in_memory_default(LINK_SECRET.to_string());

        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);
        let reg = RegistrationRequest {
            identity_key: *pair.identity_key(),
            device_activation: DeviceActivationInfo {
                name: "Alice Phone".to_string(),
                registration_id: 1,
                key_bundle: create_publish_key_bundle(
                    Some(vec![0]),
                    Some(1),
                    Some(vec![33]),
                    Some(2),
                    &pair,
                    rng,
                ),
            },
        };

        let alice_id = create_account(
            &mut state,
            reg,
            "RealAlice".to_string(),
            "bob<3".to_string(),
        )
        .await
        .map(|r| r.account_id)
        .expect("Alice can create account");

        delete_account(&mut state, alice_id)
            .await
            .expect("Alice can delete account");

        assert!(state.accounts.get_account(alice_id).await.is_err());
        assert!(state.devices.get_device(alice_id, 1).await.is_err());
        assert!(state.keys.get_last_resort_key(alice_id, 1).await.is_err());
        assert!(state.keys.get_signed_pre_key(alice_id, 1).await.is_err());
        assert!(state.keys.get_pre_key_ids(alice_id, 1).await.is_err());
        assert!(state.keys.get_pq_pre_key_ids(alice_id, 1).await.is_err());
    }
}
