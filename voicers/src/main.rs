use swing::Logger;
use std::{thread::sleep, time::Duration};

// MODULES BABBBBBYYYYYY
mod config;
mod commands;
mod discord;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Initialize the logging
        let logging_config = config::get_logging_config();
        Logger::with_config(logging_config).init().unwrap();
    
        // initialize the features config
        let features_config = config::get_features_config();
        log::debug!("Disabled features: {:?}", features_config.disabled_features);

        log::info!("Starting voiceRS...");

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
