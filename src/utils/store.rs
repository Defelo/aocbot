use matrix_sdk::Client;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct Store {
    client: Client,
}

impl Store {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &[u8]) -> anyhow::Result<Option<T>> {
        Ok(self
            .client
            .store()
            .get_custom_value(key)
            .await?
            .and_then(|x| rmp_serde::from_slice(&x).ok()))
    }

    pub async fn set<T: Serialize>(&self, key: &[u8], value: &T) -> anyhow::Result<()> {
        self.client
            .store()
            .set_custom_value_no_read(key, rmp_serde::to_vec(value)?)
            .await?;
        Ok(())
    }
}
