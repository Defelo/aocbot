use std::collections::HashMap;

use matrix_sdk::{ruma::OwnedUserId, Room};
use reqwest::Url;

use crate::{
    aoc::client::AocClient,
    config::{Config, User},
    mastodon,
    utils::store::Store,
};

pub struct Context {
    pub config: Config,
    pub store: Store,
    pub room: Room,
    pub aoc_client: AocClient,
    pub garygrady: ContextGarygrady,
    pub users: ContextUsers,
}

pub struct ContextUsers {
    pub by_aoc: HashMap<u64, User>,
    pub by_matrix: HashMap<OwnedUserId, User>,
}

pub struct ContextGarygrady {
    pub server: Url,
    pub user_id: mastodon::Id,
}

impl Context {
    pub fn new(
        config: Config,
        store: Store,
        room: Room,
        aoc_client: AocClient,
        garygrady: ContextGarygrady,
    ) -> Self {
        let users = ContextUsers::from_config(&config);

        Self {
            config,
            store,
            room,
            aoc_client,
            garygrady,
            users,
        }
    }
}

impl ContextUsers {
    fn from_config(config: &Config) -> Self {
        let by_aoc = config
            .users
            .iter()
            .flat_map(|user| Some((user.aoc?, user.clone())))
            .collect();
        let by_matrix = config
            .users
            .iter()
            .flat_map(|user| Some((user.matrix.clone()?, user.clone())))
            .collect();

        Self { by_aoc, by_matrix }
    }
}
