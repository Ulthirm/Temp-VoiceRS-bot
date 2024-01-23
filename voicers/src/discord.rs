use crate::{config,commands};
use poise::serenity_prelude as serenity;
use serenity::{Client,async_trait,all::{GatewayIntents,EventHandler,Interaction,CreateInteractionResponseMessage,CreateInteractionResponse,Ready,GuildId,Command}};
use std::env;
use std::{sync::Arc,time::Duration,collections::HashMap};
use tokio::sync::Mutex;
use tracing::{info,debug,error};

// Types used by all command functions
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {}

pub async fn start_discord_bot() -> Result<(), Box<dyn std::error::Error>> {
    // get the config
    let config = config::get_config();

    // Get the Discord token
    let token = &config.discord.bot_token;

    // TODO: Remove this debug line
    // Security risk after testing
    debug!("Discord token: {}", token);

    let intents = GatewayIntents::empty();

    let framework = poise::Framework::builder()
    .setup(move |ctx, _ready, framework| {
        Box::pin(async move {
            println!("Logged in as {}", _ready.user.name);
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(Data {})
        })
    })
    .options(poise::FrameworkOptions {
        //, commands::createvc(), commands::setmodrole(), commands::setusermod()
        commands: vec![commands::help::help(), commands::createvc::createvc(), commands::setmodrole::setmodrole(), commands::setusermod::setusermod()],
        ..Default::default()
    })
    .build();

    // Create a new instance of the Client
    let client_result = Client::builder(token, intents)
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
