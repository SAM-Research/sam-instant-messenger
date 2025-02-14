use rand::rngs::OsRng;

use crate::{
    net::HttpClient,
    register::register_client,
    storage::{Store, StoreConfig, StoreType},
    ClientError,
};

#[derive(Debug)]
pub struct Client<S: StoreType> {
    pub store: Store<S>,
}

impl<S: StoreType> Client<S> {
    pub async fn register(
        storage_config: impl StoreConfig<StoreType = S>,
    ) -> Result<Client<S>, ClientError> {
        register_client(storage_config, HttpClient, &mut OsRng).await
    }
}
