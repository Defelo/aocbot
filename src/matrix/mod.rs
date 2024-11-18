use std::{path::Path, sync::Arc};

use anyhow::Context as _;
use matrix_sdk::{config::SyncSettings, ruma::api::client::filter::FilterDefinition, Client};

use crate::context::Context;

mod commands;
mod event_handlers;
pub mod utils;

pub async fn create_client(homeserver_url: &str, store_path: &Path) -> anyhow::Result<Client> {
    Client::builder()
        .homeserver_url(homeserver_url)
        .sqlite_store(store_path, None)
        .build()
        .await
        .context("Failed to create client")
}

pub struct Bot {
    client: Client,
    sync_settings: SyncSettings,
}

impl Bot {
    pub async fn setup(client: Client) -> anyhow::Result<Self> {
        let mut filter = FilterDefinition::ignore_all();
        filter.room.timeline.not_senders = vec![client.user_id().unwrap().into()];
        filter.room.timeline.types = Some(
            [
                "m.room.message",
                "m.room.encrypted",
                "m.room.encryption",
                "m.room.member",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
        );
        filter.room.rooms = None;

        let sync_settings = SyncSettings::new().filter(filter.into());
        let sync_response = client.sync_once(sync_settings.clone()).await?;
        let sync_settings = sync_settings.token(sync_response.next_batch);

        Ok(Self {
            client,
            sync_settings,
        })
    }

    pub async fn start(self, context: Arc<Context>) -> anyhow::Result<()> {
        event_handlers::add_event_handlers(&self.client, context);

        self.client.sync(self.sync_settings).await?;

        Ok(())
    }
}
