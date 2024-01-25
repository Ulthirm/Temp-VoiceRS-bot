use crate::{commands, config};
use poise::futures_util::TryStreamExt;
use poise::serenity_prelude as serenity;
//use serenity::{Client,async_trait,all::{GatewayIntents,EventHandler,Interaction,CreateInteractionResponseMessage,CreateInteractionResponse,Ready,GuildId,Command}};
use sqlx::SqlitePool;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

// Types used by all command functions
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

//database struct
#[derive(sqlx::FromRow)]
struct User {
    vc_id: i64,
    guild_id: i64,
    last_update: i64,
    user_count: i32,
}

enum CustomEvent {
    PollingDeleteVC { vc_id: i64, guild_id: i64 },
}

// Custom user data passed to all command functions
pub struct Data {
    pub pool: Arc<SqlitePool>,
    custom_event_sender: tokio::sync::mpsc::Sender<CustomEvent>,
}

pub async fn start_discord_bot(sqlite: Arc<SqlitePool>) -> Result<(), Box<dyn std::error::Error>> {
    // get the config
    let config = config::get_config();

    // Get the Discord token
    let token = &config.discord.bot_token;

    // TODO: Remove this debug line
    // Security risk after testing
    debug!("Discord token: {}", token);

    let data = Data {
        pool: sqlite.clone(),
        custom_event_sender: tokio::sync::mpsc::channel(100).0,
    };

    let intents =
        serenity::GatewayIntents::GUILD_VOICE_STATES | serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    pool: sqlite.clone(),
                    custom_event_sender: tokio::sync::mpsc::channel(100).0,
                })
            })
        })
        .options(poise::FrameworkOptions {
            //, commands::createvc(), commands::setmodrole(), commands::setusermod()
            commands: vec![
                commands::help::help(),
                commands::createvc::entrance(),
                commands::setmodrole::setmodrole(),
                commands::setusermod::setusermod(),
                commands::contextmenu::user_info(),
            ],

            event_handler: |ctx, event, framework, data| {
                Box::pin(polling_event(ctx, event, framework, data))
            },

            ..Default::default()
        })
        .build();

    // Create a new instance of the Client
    let client_result = serenity::Client::builder(token, intents)
        .framework(framework)
        .await;

    let mut client = match client_result {
        Ok(client) => client,
        Err(e) => return Err(Box::new(e)),
    };

    // Start the client
    match client.start().await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Client error: {:?}", e);
            Err(Box::new(e))
        }
    }
}

async fn polling_event(
    ctx: poise::Context<'_, Data, Error>,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
            // Start the polling task on ready
            tokio::spawn(start_polling(data.pool.clone()));
        }

        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            // Handle voice state updates

            debug!("Voice state update: {:?} -> {:?}", old, new);
            // Get the current time in seconds in EPOCH
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // Get the DB from ctx
            let pool = data.pool.clone();

            // User joined or moved in a voice channel
            if let Some(channel_id) = new.channel_id {
                let channel_id_i64 = channel_id.get() as i64;
                info!("User joined a voice channel: {}", channel_id_i64);
                // User joined or moved in a voice channel
                // Update the user count and last_update for this channel
                let query = sqlx::query(
                    "UPDATE users SET last_update = ?, user_count = user_count + 1 WHERE vc_id = ?",
                )
                .bind(now)
                .bind(channel_id_i64);

                query.execute(&*pool).await.map_err(|e| {
                    error!("Failed to update database: {:?}", e);
                    Error::from(e)
                })?;
            }
            // User left a voice channel
            if let Some(old) = old {
                if let Some(old_channel_id) = old.channel_id {
                    let old_channel_id_i64 = old_channel_id.get() as i64;
                    info!("User left a voice channel: {}", old_channel_id_i64);

                    // Update the user count and last_update for this channel
                    let query = sqlx::query(
                        "UPDATE users SET last_update = ?, user_count = CASE WHEN user_count - 1 < 0 THEN 0 ELSE user_count - 1 END WHERE vc_id = ?"
                    )
                    .bind(now)
                    .bind(old_channel_id_i64);

                    query.execute(&*pool).await.map_err(|e| {
                        error!("Failed to update database: {:?}", e);
                        Error::from(e)
                    })?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn start_polling<'a>(pool: Arc<SqlitePool>) -> Result<(), Error> {
    debug!("Starting polling task");

    // get the config
    let config = config::get_config();
    let voice_timeout = config.voice.global_timeout;
    let delay = Duration::from_secs(voice_timeout);

    loop {
        // Sleep for the delay
        tokio::time::sleep(delay).await;
        debug!("Polling task woke up");

        // Get the current time in seconds in EPOCH
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // begin processing the VCs recorded in the DB and check for inactivity
        let query =
            sqlx::query_as::<_, User>("SELECT vc_id, guild_id, last_update, user_count FROM users");
        let mut rows = query.fetch(&*pool);

        while let Some(row) = rows.try_next().await.unwrap() {
            let vc_id = row.vc_id;
            let guild_id = row.guild_id;
            let last_update = row.last_update;
            let user_count = row.user_count;

            // Retrieve and update the user count for the VC in the guild
            // ... (Implement the logic to interact with the guild and VC)

            // If VC has been vacant for longer than voice_timeout, delete it
            if user_count == 0 && (now - last_update) > voice_timeout as i64 {
                info!(
                    "VC {} in guild {} has been vacant for longer than voice_timeout, deleting it",
                    vc_id, guild_id
                );
                // Delete the VC from the guild
            }
        }
    }
}
