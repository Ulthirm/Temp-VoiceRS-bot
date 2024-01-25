use sqlx::sqlite::SqlitePool;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::{thread::sleep, time::Duration};
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

// MODULES BABBBBBYYYYYY
mod commands;
mod config;
mod discord;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize the logging
    let logging_config = config::get_logging_config();

    // Set up the tracing subscriber here
    let subscriber = FmtSubscriber::builder()
        .with_max_level(logging_config)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Initialize database
    let db_path = "VCs.db";
    initialize_database(db_path).expect("Failed to initialize database");

    info!("Database initializing...");
    // Connect to the database
    let database_url = format!("sqlite:{}", db_path);
    let pool = SqlitePool::connect(&database_url).await?;
    let shared_pool = Arc::new(pool);

    let create_table_query = r#"
    CREATE TABLE IF NOT EXISTS users (
        vc_id INTEGER PRIMARY KEY,
        guild_id INTEGER NOT NULL,
        last_update INTEGER NOT NULL,
        user_count INTEGER NOT NULL 
        );
    "#;
    sqlx::query(create_table_query)
        .execute(&*shared_pool)
        .await
        .expect("Failed to create VC table");

    info!("Starting voiceRS...");

    // initialize the features config
    let features_config = config::get_features_config();
    debug!("Disabled features: {:?}", features_config.disabled_features);

    // Tasks Vector for Tokio tasks
    let mut tasks = Vec::new();

    // Finally begin working on Discord bot
    // immediately async the bot onto it's own thread
    let _bot_handle = tasks.push(tokio::spawn(async {
        discord::start_discord_bot(shared_pool).await;
    }));

    // Wait for all the spawned tasks to complete
    for task in tasks {
        let _ = task.await?;
    }

    Ok(())
}

fn initialize_database(db_path: &str) -> std::io::Result<()> {
    if !Path::new(db_path).exists() {
        warn!("Database does not exist, creating it...");
        // Create an empty file to initialize the database
        fs::File::create(db_path)?;
    }
    Ok(())
}
