use std::{thread::sleep, time::Duration};
use tracing::{info,debug,error,Level};
use tracing_subscriber::FmtSubscriber;

// MODULES BABBBBBYYYYYY
mod config;
mod commands;
mod discord;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Initialize the logging
        let logging_config = config::get_logging_config();

        // Set up the tracing subscriber here
        let subscriber = FmtSubscriber::builder()
            .with_max_level(logging_config)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    
        
        info!("Starting voiceRS...");

        // initialize the features config
        let features_config = config::get_features_config();
        debug!("Disabled features: {:?}", features_config.disabled_features);

        // Tasks Vector for Tokio tasks
        let mut tasks = Vec::new();

        // Finally begin working on Discord bot
        // immediately async the bot onto it's own thread
        let _bot_handle = tasks.push(tokio::spawn(async {
            discord::start_discord_bot().await;
        }));




        // Wait for all the spawned tasks to complete
        for task in tasks {
            let _ = task.await?;
        }

        Ok(())
        
}
