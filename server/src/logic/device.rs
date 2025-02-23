use libsignal_protocol::IdentityKey;
use sam_common::{
    address::{AccountId, DeviceId},
    api::{
        device::{DeviceActivationInfo, LinkDeviceRequest, LinkDeviceResponse},
        LinkDeviceToken,
    },
    time_now_millis,
};

use crate::{
    auth::{
        device::{create_token, verify_token},
        password::Password,
    },
    managers::{
        entities::device::Device,
        traits::{account_manager::AccountManager, device_manager::DeviceManager},
    },
    state::{state_type::StateType, ServerState},
    ServerError,
};

use super::keys::add_keybundle;

pub async fn create_device_token<T: StateType>(
    state: &ServerState<T>,
    account_id: AccountId,
) -> Result<LinkDeviceToken, ServerError> {
    Ok(create_token(
        &state.devices.link_secret().await?,
        account_id,
    ))
}

pub async fn link_device<T: StateType>(
    state: &mut ServerState<T>,
    device_link: LinkDeviceRequest,
    password: String,
) -> Result<LinkDeviceResponse, ServerError> {
    let account_id = verify_token(
        &state.devices.link_secret().await?,
        state.devices.provision_expire_seconds().await?,
        device_link.token,
    )?;

    let account = state.accounts.get_account(account_id).await?;

    let next_id = state.devices.next_device_id(account_id).await?;

    create_device(
        state,
        account_id,
        account.identity(),
        device_link.device_activation,
        next_id,
        password,
    )
    .await?;

    Ok(LinkDeviceResponse {
        account_id,
        device_id: next_id,
    })
}

pub async fn unlink_device<T: StateType>(
    state: &mut ServerState<T>,
    account_id: AccountId,
    device_id: DeviceId,
) -> Result<(), ServerError> {
    state.devices.remove_device(account_id, device_id).await
}

pub async fn create_device<T: StateType>(
    state: &mut ServerState<T>,
    account_id: AccountId,
    identity: &IdentityKey,
    device_info: DeviceActivationInfo,
    device_id: DeviceId,
    password: String,
) -> Result<(), ServerError> {
    let device = Device::builder()
        .id(device_id)
        .registration_id(device_info.registration_id)
        .name(device_info.name)
        .creation(time_now_millis())
        .password(Password::generate(password)?)
        .build();

    state.devices.add_device(account_id, &device).await?;

    add_keybundle(
        state,
        identity,
        account_id,
        device_id,
        device_info.key_bundle.into(),
    )
    .await
}

#[cfg(test)]
mod test {
    use libsignal_protocol::IdentityKeyPair;
    use rand::rngs::OsRng;
    use sam_common::{
        address::AccountId,
        api::{device::DeviceActivationInfo, Key, RegistrationRequest},
    };

    use crate::{
        logic::{
            account::create_account,
            device::{create_device, create_device_token, link_device, unlink_device},
        },
        managers::traits::{
            device_manager::DeviceManager,
            key_manager::{
                LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager,
            },
        },
        state::ServerState,
        test_utils::{create_device_link, create_publish_pre_keys},
    };

    #[tokio::test]
    async fn test_create_device() {
        let mut state = ServerState::in_memory_test();

        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);

        let device_info = DeviceActivationInfo {
            name: "a".to_string(),
            registration_id: 1.into(),
            key_bundle: create_publish_pre_keys(
                Some(vec![0]),
                Some(1),
                Some(vec![33]),
                Some(2),
                &pair,
                rng,
            )
            .try_into()
            .expect("Can make RegistrationPreKeys"),
        };

        let account_id = AccountId::generate();
        let account_pwd = "huntermotherboard7".to_string();

        create_device(
            &mut state,
            account_id,
            pair.identity_key(),
            device_info,
            1.into(),
            account_pwd.clone(),
        )
        .await
        .expect("Devices can be created");

        // Check if device is created
        let device = state
            .devices
            .get_device(account_id, 1.into())
            .await
            .expect("User has primary device");

        assert!(device.registration_id() == 1.into());
        assert!(device.name() == "a");
        device
            .password()
            .verify(account_pwd)
            .expect("Users device password is set correctly");

        // check if keys are inserted

        let ec_key_ids = state
            .keys
            .get_pre_key_ids(account_id, 1.into())
            .await
            .unwrap();
        let signed_ec_id = state
            .keys
            .get_signed_pre_key(account_id, 1.into())
            .await
            .unwrap()
            .id();

        assert!(ec_key_ids == Some(vec![0]));
        assert!(signed_ec_id == 1);

        let pq_key_ids = state
            .keys
            .get_pq_pre_key_ids(account_id, 1.into())
            .await
            .unwrap();
        let last_resort_id = state
            .keys
            .get_last_resort_key(account_id, 1.into())
            .await
            .unwrap()
            .id();

        assert!(pq_key_ids == Some(vec![33]));
        assert!(last_resort_id == 2);
    }

    #[tokio::test]
    async fn test_unlink_device() {
        let mut state = ServerState::in_memory_test();

        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);

        let device_info = DeviceActivationInfo {
            name: "a".to_string(),
            registration_id: 1.into(),
            key_bundle: create_publish_pre_keys(None, Some(1), None, Some(2), &pair, rng)
                .try_into()
                .expect("Can make RegistrationPreKeys"),
        };

        let account_id = AccountId::generate();
        let account_pwd = "huntermotherboard7".to_string();

        create_device(
            &mut state,
            account_id,
            pair.identity_key(),
            device_info,
            1.into(),
            account_pwd.clone(),
        )
        .await
        .expect("Devices can be created");

        unlink_device(&mut state, account_id, 1.into())
            .await
            .expect("Device exists");

        assert!(state
            .devices
            .get_device(account_id, 1.into())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_create_device_token() {
        let state = ServerState::in_memory_test();
        assert!(create_device_token(&state, AccountId::generate())
            .await
            .is_ok())
    }

    #[tokio::test]
    async fn test_link_device() {
        let mut state = ServerState::in_memory_test();

        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);
        let reg = RegistrationRequest {
            identity_key: *pair.identity_key(),
            device_activation: DeviceActivationInfo {
                name: "Alice Phone".to_string(),
                registration_id: 1.into(),
                key_bundle: create_publish_pre_keys(
                    Some(vec![0]),
                    Some(1),
                    Some(vec![33]),
                    Some(2),
                    &pair,
                    rng,
                )
                .try_into()
                .expect("Can make RegistrationPreKeys"),
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

        let token = create_device_token(&state, alice_id)
            .await
            .expect("Alice can create device token");

        let device_pwd = "charlie<3".to_string();

        let key_bundle = create_publish_pre_keys(None, Some(1), None, Some(2), &pair, rng)
            .try_into()
            .expect("Can make RegistrationPreKeys");
        let device_link = create_device_link(token, "Alice Laptop", 2.into(), key_bundle);

        let res = link_device(&mut state, device_link, device_pwd)
            .await
            .expect("Alice can link device");

        assert!(res.account_id == alice_id);
    }
}
