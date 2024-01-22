use crate::{config,commands};
use serenity::{Client,async_trait,all::{GatewayIntents,EventHandler,Interaction,Context,CreateInteractionResponseMessage,CreateInteractionResponse,Ready,GuildId,Command}};
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "id" => Some(commands::id::run(&command.data.options())),
                "attachmentinput" => Some(commands::attachmentinput::run(&command.data.options())),
                "modal" => {
                    commands::modal::run(&ctx, &command).await.unwrap();
                    None
                },
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = guild_id
            .set_commands(&ctx.http, vec![
                commands::ping::register(),
                commands::id::register(),
                commands::numberinput::register(),
                commands::attachmentinput::register(),
                commands::modal::register(),
            ])
            .await;

        println!("I now have the following guild slash commands: {commands:#?}");

        let guild_command =
            Command::create_global_command(&ctx.http, commands::wonderful_command::register())
                .await;

        println!("I created the following global slash command: {guild_command:#?}");
    }
}

pub async fn start_discord_bot() -> Result<(), Box<dyn std::error::Error>> {
    // get the config
    let config = config::get_config();

    // Get the Discord token
    let token = &config.discord.bot_token;

    // TODO: Remove this debug line
    // Security risk after testing
    log::debug!("Discord token: {}", token);

    let intents = GatewayIntents::empty();

    // Create a new instance of the Client
    let client_result = Client::builder(token, intents)
        .event_handler(Handler)
        .await;

    let mut client = match client_result {
        Ok(client) => client,
        Err(e) => return Err(Box::new(e)),
    };

    // Start the client
    match client.start().await {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Client error: {:?}", e);
            Err(Box::new(e))
        }
    }
}
