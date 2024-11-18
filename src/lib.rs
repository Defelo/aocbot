use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Context as _};
use matrix_sdk::{
    matrix_auth::MatrixSession,
    ruma::api::client::{
        membership::join_room_by_id,
        uiaa::{self, AuthData, UserIdentifier},
    },
    RoomState,
};
use reqwest::Url;
use tracing::{error, info};

use crate::{
    aoc::client::AocClient,
    context::{Context, ContextGarygrady},
    matrix::create_client,
    utils::store::Store,
};

mod aoc;
mod config;
mod context;
mod mastodon;
mod matrix;
mod tasks;
mod utils;

const SESSION_STORE_KEY: &[u8] = b"session";

pub async fn setup(config_path: impl Iterator<Item = &str>) -> anyhow::Result<()> {
    let config = config::load(config_path).context("Failed to load config")?;

    let client = create_client(&config.matrix.homeserver, &config.matrix.store_path).await?;

    let store = Store::new(client.clone());

    // don't overwrite an existing session
    if store
        .get::<MatrixSession>(SESSION_STORE_KEY)
        .await?
        .is_some()
    {
        bail!("The store already contains a session.");
    }

    // Matrix login
    eprintln!("Enter either a login token or a username and password for the bot account:");
    let line = read_line()?;
    let mut parts = line.split_whitespace();
    let (first, second, third) = (parts.next(), parts.next(), parts.next());
    let login = match (first, second, third) {
        (Some(login_token), None, _) => client
            .matrix_auth()
            .login_token(login_token.rsplit('=').next().unwrap_or_default()),
        (Some(username), Some(password), None) => {
            client.matrix_auth().login_username(username, password)
        }
        _ => bail!("Failed to read credentials from stdin"),
    };
    let response = login.initial_device_display_name("Bot").await?;
    info!(user_id = ?response.user_id, devicd_id = ?response.device_id, "Login successful");

    // save session
    let session = MatrixSession::from(&response);
    store
        .set::<MatrixSession>(SESSION_STORE_KEY, &session)
        .await?;

    // create and upload cross signing keys
    info!("Trying to bootstrap cross signing");
    if let Err(err) = client.encryption().bootstrap_cross_signing(None).await {
        let Some(response) = err.as_uiaa_response() else {
            return Err(err.into());
        };
        let session = response
            .session
            .as_ref()
            .ok_or_else(|| anyhow!("no uiaa session"))?
            .to_owned();
        let auth = if let (Some(username), Some(password)) = (first, second) {
            let mut password = uiaa::Password::new(
                UserIdentifier::UserIdOrLocalpart(username.into()),
                password.into(),
            );
            password.session = Some(session);
            AuthData::Password(password)
        } else {
            let sso_url = client.homeserver().join(&format!(
                "_matrix/client/v3/auth/m.login.sso/fallback/web?session={session}",
            ))?;
            eprintln!("Complete uiaa via sso at {sso_url}, then press enter");
            read_line()?;
            AuthData::fallback_acknowledgement(session)
        };
        client
            .encryption()
            .bootstrap_cross_signing(Some(auth))
            .await?;
    }

    // enable backups for encryption keys
    client.encryption().backups().create().await?;

    // join the matrix room
    let room_id = config.matrix.room_id;
    info!("Trying to join room {room_id}");
    while let Err(err) = client
        .send(join_room_by_id::v3::Request::new(room_id.clone()), None)
        .await
    {
        error!("Failed to join room {room_id}: {err}");
        eprintln!("Press enter to try again");
        read_line()?;
    }

    Ok(())
}

pub async fn run(config_path: impl Iterator<Item = &str>) -> anyhow::Result<()> {
    let config = config::load(config_path).context("Failed to load config")?;

    let client = create_client(&config.matrix.homeserver, &config.matrix.store_path).await?;

    let store = Store::new(client.clone());

    // Matrix Login
    let session = store
        .get::<MatrixSession>(SESSION_STORE_KEY)
        .await?
        .ok_or_else(|| anyhow!("Create a session first using `aocbot setup`"))?;
    client.matrix_auth().restore_session(session).await?;

    let response = client.whoami().await?;
    info!(user_id = %response.user_id, devicd_id = %response.device_id.unwrap(), "Matrix login successful");

    // Advent of Code Login
    let aoc_session = std::fs::read_to_string(&config.aoc.session_file)?;
    let aoc_client = AocClient::new(
        aoc_session.trim(),
        Duration::from_secs(config.aoc.default_cache_ttl),
        config
            .aoc
            .cache_ttl_rules
            .iter()
            .map(|r| (r.minutes_after_unlock, Duration::from_secs(r.ttl)))
            .collect(),
        store.clone(),
    )
    .await?;
    info!(
        user_id = aoc_client.whoami().user_id,
        invite_code = aoc_client.whoami().invite_code,
        "AoC login successful"
    );

    // Mastodon setup
    let garygrady_server = Url::parse("https://mastodon.social/")?;
    let garygrady_id = mastodon::lookup_account(&garygrady_server, "garygrady")
        .await?
        .id;
    let garygrady = ContextGarygrady {
        server: garygrady_server,
        user_id: garygrady_id,
    };

    // Setup matrix bot
    let bot = matrix::Bot::setup(client.clone()).await?;

    // Find matrix room
    let room_id = &config.matrix.room_id;
    let room = client
        .get_room(room_id)
        .ok_or_else(|| anyhow!("Failed to find matrix room '{room_id}'"))?;
    if room.state() != RoomState::Joined {
        info!("Trying to join room {}", room.room_id());
        room.join().await?;
    }

    let context = Arc::new(Context::new(config, store, room, aoc_client, garygrady));

    tasks::start(Arc::clone(&context));

    bot.start(context).await
}

fn read_line() -> anyhow::Result<String> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(line)
}
