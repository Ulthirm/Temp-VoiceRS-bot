use crate::{commands, config};
use poise::futures_util::TryStreamExt;
use poise::serenity_prelude as serenity;
use serenity::model::id::ChannelId;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{sync::Arc, time::Duration};
use serenity::futures::Stream;
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
}

pub async fn start_discord_bot(sqlite: Arc<SqlitePool>) -> Result<(), Box<dyn std::error::Error>> {
    // get the config
    let config = config::get_config();

    // Get the Discord token
    let token = &config.discord.bot_token;

    // TODO: Remove this debug line
    // Security risk after testing
    debug!("Discord token: {}", token);

    let intents = serenity::GatewayIntents::GUILD_VOICE_STATES
        | serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    pool: sqlite.clone(),
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
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
            let (sender, receiver) = tokio::sync::mpsc::channel(100);
            //let (pollingsyncsender, pollingsyncreceiver) = tokio::sync::mpsc::channel(100);
            // Start the polling task on ready
            tokio::spawn(start_polling(data.pool.clone(), sender, ctx.clone()));

            let http_clone = Arc::clone(&ctx.http);
            let pool_clone = data.pool.clone();

            tokio::spawn(handle_polling_delete_event(
                receiver, http_clone, pool_clone,
            ));
        }

        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            // Get the Misc confit
            let vcmisc_config = config::get_vcmisc_config();

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
                // Send them the rules
                // Asynchronous context, make sure to await the ctx.send method

                let text_channel_id = channel_id;
                let user_id = serenity::UserId::from(new.user_id);

                // Prepare the message content
                let message_content = format!("<@{}> \n{} The mods will have direct access to this channel \n Please be sure to follow all the rules and guidelines of the server {} \n{}", user_id, vcmisc_config.vc_custom_prefix, vcmisc_config.vc_rules, vcmisc_config.vc_custom_suffix);

                // Send the message to the associated text channel
                if let Err(e) = text_channel_id.say(&ctx.http, &message_content).await {
                    error!("Failed to send message to text channel: {:?}", e);
                }

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

async fn start_polling<'a>(
    pool: Arc<SqlitePool>,
    sender: tokio::sync::mpsc::Sender<CustomEvent>,
    ctx: serenity::Context,
) -> Result<(), Error> {
    debug!("Starting polling task");

    // get the config
    let config = config::get_config();
    let voice_timeout = config.voice.global_timeout;
    let sync_interval = 4; // Interval for syncing with the database
    let delay = Duration::from_secs(voice_timeout);
    let mut loop_counter = 0;

    loop {
        // Sleep for the delay
        tokio::time::sleep(delay).await;
        debug!("Polling task woke up");
        loop_counter += 1;

        if loop_counter == sync_interval {
            debug!("Syncing with the database");

            // Sync the DB with the guilds and VCs
            let rows = sqlx::query_as::<_, User>(
                "SELECT vc_id, guild_id, last_update, user_count FROM users",
            )
            .fetch_all(&*pool)
            .await
            .unwrap();
            for row in rows {
                let vc_id = row.vc_id as u64; // Convert to u64 if necessary
                let guild_id = row.guild_id as u64; // Convert to u64 if necessary
                let channel_id = serenity::ChannelId::from(vc_id);
                let guild_id = serenity::GuildId::from(guild_id);

                let current_user_count = get_user_count_in_vc(&ctx.clone(), guild_id, channel_id)
                    .await
                    .unwrap();

                info!(
                    "Syncing VC {} in guild {} with DB user count {}",
                    vc_id, guild_id, current_user_count
                );

                // Only update the database if the user count has changed
                if current_user_count as i32 != row.user_count {
                    let update_query = sqlx::query::<sqlx::Sqlite>(
                        "UPDATE users SET user_count = ? WHERE vc_id = ?",
                    )
                    .bind(current_user_count as i32)
                    .bind(vc_id as i64);

                    update_query
                        .execute(&*pool)
                        .await
                        .expect("Failed to execute update");
                    info!(
                        "Updated VC {} in guild {} with new user count {}",
                        vc_id, guild_id, current_user_count
                    );
                } else {
                    info!("No update needed for VC {} in guild {}", vc_id, guild_id);
                }
            }

            loop_counter = 0;
        }

        // Process voice channels for inactivity
        let mut rows = get_users(&pool);

        while let Some(row) = rows.try_next().await.unwrap() {
            let vc_id = row.vc_id;
            let guild_id = row.guild_id;
            let last_update = row.last_update;
            let user_count = row.user_count;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // If VC has been vacant for longer than voice_timeout, delete it
            if user_count == 0 && (now - last_update) > voice_timeout as i64 {
                info!(
                    "VC {} in guild {} has been vacant for longer than voice_timeout, deleting it",
                    vc_id, guild_id
                );
                delete_voice_channel(&sender, vc_id, guild_id).await;
            }
        }
    }
}

fn get_users(pool: &Arc<sqlx::Pool<sqlx::Sqlite>>) -> std::pin::Pin<Box<dyn Stream<Item = Result<User, sqlx::Error>> + Send + '_>> {
    let query =
        sqlx::query_as::<_, User>("SELECT vc_id, guild_id, last_update, user_count FROM users");
    let mut rows = query.fetch(&**pool);
    rows
}

async fn delete_voice_channel(sender: &tokio::sync::mpsc::Sender<CustomEvent>, vc_id: i64, guild_id: i64) {
    sender
        .send(CustomEvent::PollingDeleteVC { vc_id, guild_id })
        .await
        .unwrap();
}

async fn get_user_count_in_vc(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    vc_id: serenity::ChannelId,
) -> Result<usize, serenity::Error> {
    // Attempt to access the guild from the cache
    if let Some(guild) = ctx.cache.guild(guild_id) {
        // Filter the voice_states for the specific voice channel and count
        let user_count = guild
            .voice_states
            .values()
            .filter(|voice_state| voice_state.channel_id == Some(vc_id))
            .count();

        Ok(user_count)
    } else {
        Err(serenity::Error::Other("Guild not found in cache"))
    }
}

async fn handle_polling_delete_event(
    mut receiver: tokio::sync::mpsc::Receiver<CustomEvent>,
    http: Arc<serenity::Http>,
    pool: Arc<SqlitePool>,
) {
    while let Some(event) = receiver.recv().await {
        match event {
            CustomEvent::PollingDeleteVC { vc_id, guild_id } => {
                debug!(
                    "Deleting voice channel with ID {} in guild id {}",
                    vc_id, guild_id
                );
                // Delete the voice channel
                match http
                    .delete_channel(
                        ChannelId::from(vc_id as u64),
                        Some("Cleaning up inactive voice channel"),
                    )
                    .await
                {
                    Ok(_) => {
                        // Delete the VC from the DB
                        let query = sqlx::query("DELETE FROM users WHERE vc_id = ?").bind(vc_id);
                        query.execute(&*pool).await.unwrap();

                        info!("Deleted voice channel with ID {}", vc_id);
                    }
                    Err(why) => error!("Failed to delete voice channel: {:?}", why),
                }
            }
        }
    }
}
