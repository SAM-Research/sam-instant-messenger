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
    state: ServerState<T>,
    account_id: Uuid,
) -> Result<(), ServerError> {
    {
        let mut keys = state.keys.lock().await;
        let mut messages = state.messages.lock().await;
        let mut devices = state.devices.lock().await;
        for device_id in devices.get_devices(&account_id).await? {
            for msg_id in messages.get_messages(&account_id, &device_id).await? {
                messages
                    .remove_message(&account_id, &device_id, &msg_id)
                    .await?;
            }

            for id in keys.get_pre_keys(&account_id, &device_id).await? {
                keys.remove_pre_key(&account_id, &device_id, id).await?
            }

            keys.remove_signed_pre_key(&account_id, &device_id).await?;

            for id in keys.get_pq_pre_keys(&account_id, &device_id).await? {
                keys.remove_pq_pre_key(&account_id, &device_id, id).await?
            }

            keys.remove_last_resort_key(&account_id, &device_id).await?;

            devices.remove_device(&account_id, device_id).await?;
        }
    }

    let mut accounts = state.accounts.lock().await;
    accounts.remove_account(account_id).await
}

pub async fn create_account<T: StateType>(
    state: &ServerState<T>,
    registration: RegistrationRequest,
    username: String,
    password: String,
) -> Result<RegistrationResponse, ServerError> {
    let account = Account::builder()
        .id(Uuid::new_v4())
        .username(username)
        .identity(registration.identity_key)
        .build();

    state.accounts.lock().await.add_account(&account).await?;

    create_device(
        state,
        account.id(),
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
    use libsignal_protocol::{IdentityKey, KeyPair};
    use rand::rngs::OsRng;
    use sam_common::api::{
        device::DeviceActivationInfo, keys::PublishKeyBundle, RegistrationRequest,
    };

    use crate::{
        logic::account::create_account,
        managers::traits::{account_manager::AccountManager, device_manager::DeviceManager},
        state::ServerState,
    };

    static LINK_SECRET: &str = "test";

    #[tokio::test]
    async fn test_create_account() {
        let state = ServerState::in_memory_default(LINK_SECRET.to_string());

        let mut rng = OsRng;
        let pair = KeyPair::generate(&mut rng);
        let id = IdentityKey::new(pair.public_key);

        // TODO: We need some test utility to create these structs, maybe in the common module?
        let reg = RegistrationRequest {
            identity_key: id,
            device_activation: DeviceActivationInfo {
                name: "Alice Phone".to_string(),
                registration_id: 1u32,
                key_bundle: PublishKeyBundle {
                    // TODO: how to keys?
                    pre_keys: None,
                    signed_pre_key: None,
                    pq_pre_keys: None,
                    pq_last_resort_pre_key: None,
                },
            },
        };

        let alice_id = create_account(&state, reg, "RealAlice".to_string(), "bob<3".to_string())
            .await
            .map(|r| r.account_id)
            .expect("Alice can create account");

        let account = state
            .accounts
            .lock()
            .await
            .get_account(&alice_id)
            .await
            .expect("Alice has an account");

        assert!(*account.id() == alice_id);
        assert!(*account.identity() == id);
        assert!(account.username() == "RealAlice");

        let device = state
            .devices
            .lock()
            .await
            .get_device(&alice_id, &1)
            .await
            .expect("Alice has primary device");

        assert!(device.registration_id() == 1);
        assert!(device.name() == "Alice Phone");
        device
            .password()
            .verify("bob<3".to_string())
            .expect("Alice loves bob");
    }
}
