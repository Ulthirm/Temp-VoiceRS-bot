use colored::Colorize;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    {fs, fs::OpenOptions},
    {io, io::Write},
};
use toml::Value;
use tracing::{debug, level_filters::LevelFilter, warn};

//expect root Table and configure subtables, osc
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub logging: Logging,
    pub features: Features,
    pub moderation: Moderation,
    pub voice: Voice,
    pub discord: Discord,
    pub misc: Misc,
}

// This is a struct for the logging level
#[derive(Serialize, Deserialize, Debug)]
pub struct Logging {
    #[serde(default = "default_logging_level")]
    pub level: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Moderation {
    #[serde(default = "default_moderator")]
    pub moderator_roles: Vec<String>,
    #[serde(default = "default_moderator")]
    pub moderator_users: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Voice {
    #[serde(default = "default_voice")]
    pub global_timeout: u64,
}

// This is for disabled features
// Wow I'm a real programmer now, I'm writing comments for my code
#[derive(Serialize, Deserialize, Debug)]
pub struct Features {
    #[serde(default = "default_features")]
    pub disabled_features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Discord {
    #[serde(default = "default_discord")]
    pub bot_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Misc {
    #[serde(default = "default_discord")]
    pub vc_rules: String,
    #[serde(default = "default_discord")]
    pub vc_custom_prefix: String,
    #[serde(default = "default_discord")]
    pub vc_custom_suffix: String,
    #[serde(default = "default_features")]
    pub vc_mandatory_roles: Vec<String>,
    #[serde(default = "default_discord")]
    pub vc_no_permission: String,
    #[serde(default = "default_voice")]
    pub vc_category: u64,
}

// Default values for the config for the deserializer
// These do not declare the default values in the file
// just the values if the data isnt capable of being deseriazed properly
fn default_logging_level() -> String {
    "Debug".to_string()
}

fn default_moderator() -> Vec<String> {
    vec!["".to_string(), "".to_string()]
}

fn default_voice() -> u64 {
    300
}

fn default_features() -> Vec<String> {
    vec!["".to_string(), "".to_string()]
}

fn default_discord() -> String {
    "".to_string()
}

// Make CONFIG a public static so it's accessible from other modules
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config_path = "config.toml";
    match fs::read_to_string(config_path) {
        Ok(config_str) => match toml::from_str::<Config>(&config_str) {
            Ok(config) => {
                verify_config(&config);
                config
            }
            Err(e) => {
                println!(
                    "{}{}{}",
                    "Warn:".yellow().bold(),
                    "Failed to parse config: ",
                    e
                );
                repair_config(config_str).expect("Failed to repair config");
                let repaired_config_str =
                    fs::read_to_string(config_path).expect("Failed to read repaired config");
                toml::from_str(&repaired_config_str).expect("Failed to parse repaired config")
            }
        },
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                create_config().expect("Failed to create config");
            } else {
                panic!("Failed to read config file: {}", e);
            }
            let new_config_str =
                fs::read_to_string(config_path).expect("Failed to read new config");
            toml::from_str(&new_config_str).expect("Failed to parse new config")
        }
    }
});

pub fn get_config() -> &'static Config {
    &CONFIG
}

// Wow what a name, I wonder what this function is for
fn create_config() -> io::Result<()> {
    println!(
        "{}{}",
        "Info:".green().bold(),
        "Creating a new config file..."
    );

    let mut config_file = fs::File::create("config.toml")?;

    // This is the default config data that will be written to the file.
    // My CoDE Is SelF DoCuMeNtInG
    let config_data = r#"[logging]
    # This is the log level that VoiceRS will use.
    # Default: Info
    level = "Info"

    [features]
    # This defines features to be disabled
    # Note that disabling features may have unpredictable behaviors
    # options: 
    # default: ["",""]
    disabled_features = ["",""]

    [moderation]
    # This is the moderator roles and users added by the server owner
    # This uses the snowflake ID of the role or user
    # For more info on snowflake IDs, see https://discord.com/developers/docs/reference#snowflakes
    # default: ["",""]
    moderator_roles = ["",""]
    moderator_users = ["",""]

    [voice]
    # This defines the Global Timeout period for voice channels
    # This is the time in seconds before a voice channel is automatically closed
    # default: 300
    global_timeout = 300

    [discord]
    # This defines the Discord token for the bot
    # This is required for the bot to function
    # default: ""
    bot_token = ""

    [misc]
    # This defined the custom rules link for the bot
    # This is optional
    # default: ""
    vc_rules = ""
    vc_custom_prefix = ""
    vc_custom_suffix = ""
    vc_mandatory_roles = ["",""]
    vc_no_permission = ""
    vc_category = ""
    "#;

    let config_bytes = config_data.as_bytes();
    config_file.write_all(config_bytes)?; //write default config
    Ok(())
}

// generate the logging config for each logger implementation across the files

pub fn get_logging_config() -> LevelFilter {
    // This might be unnecessary and could probably be directly called in the let level line
    let log_level_str = &CONFIG.logging.level;

    // Parse the log level from string, defaulting to 'Debug' if there's an error

    let level_filter = LevelFilter::from_str(log_level_str).unwrap_or_else(|_| {
        eprintln!(
            "Warn: Unable to parse log level from config: {}. Defaulting to 'Debug'",
            log_level_str
        );
        LevelFilter::DEBUG
    });

    println!(
        "{}{}{:?}",
        "Info:".green().bold(),
        "Logging level: ",
        level_filter
    );

    level_filter
}

// generate the features config for each feature implementation across the files
pub fn get_features_config() -> &'static Features {
    &CONFIG.features
    // eventually we might want to do some processing to verify the features are valid or not blank
}

pub fn get_vcmisc_config() -> &'static Misc {
    &CONFIG.misc
}

fn verify_config(config: &Config) {
    // Verify the logging level
    let log_level_str = &config.logging.level;
    if log_level_str.is_empty() {
        println!("{}{}","Warn:".yellow().bold(), "Empty log level found in config\n This is not a valid log level and will be defaulted to 'Debug'");
    }

    // Verify the features
    let features = &config.features.disabled_features;
    for feature in features {
        if feature.is_empty() {
            println!("{}{}","Warn:".yellow().bold(), "Empty disabled feature found in config\n This is not a valid feature and will be ignored.");
        }
    }

    // Verify the voice timeout
    let voice_timeout = &config.voice.global_timeout;
    if *voice_timeout == 0 {
        println!("{}{}","Warn:".yellow().bold(), "Invalid voice timeout found in config\n This is not a valid timeout and will be defaulted to 300 seconds.");
    }

    // Verify the moderator roles
    let moderator_roles = &config.moderation.moderator_roles;
    for role in moderator_roles {
        if role.is_empty() {
            println!("{}{}","Warn:".yellow().bold(), "Empty moderator role found in config\n This is not a valid role and will be ignored.");
        }
    }

    // Verify the moderator users
    let moderator_users = &config.moderation.moderator_users;
    for user in moderator_users {
        if user.is_empty() {
            println!("{}{}","Warn:".yellow().bold(), "Empty moderator user found in config\n This is not a valid user and will be ignored.");
        }
    }

    // Verify the discord token
    let discord_token = &config.discord.bot_token;
    if discord_token.is_empty() {
        println!("{}{}","ERROR:".red().bold(), "Empty discord token found in config\n This is not a valid token and will be ignored.\n This means the bot will not work.");
    }
}

// I hate this function
// We abuse the errors to make it do what we want
fn repair_config(config_str: String) -> io::Result<()> {
    println!(
        "{}{}",
        "Warn:".yellow().bold(),
        "Repairing the Config file..."
    );

    let current_config_str = config_str;

    let logging: Logging = toml::from_str(&current_config_str).unwrap_or_else(|_| Logging {
        level: default_logging_level(),
    });

    let features: Features = toml::from_str(&current_config_str).unwrap_or_else(|_| Features {
        disabled_features: default_features(),
    });

    let moderation: Moderation =
        toml::from_str(&current_config_str).unwrap_or_else(|_| Moderation {
            moderator_roles: default_moderator(),
            moderator_users: default_moderator(),
        });

    let voice: Voice = toml::from_str(&current_config_str).unwrap_or_else(|_| Voice {
        global_timeout: default_voice(),
    });

    let discord: Discord = toml::from_str(&current_config_str).unwrap_or_else(|_| Discord {
        bot_token: default_discord(),
    });

    let misc: Misc = toml::from_str(&current_config_str).unwrap_or_else(|_| Misc {
        vc_rules: default_discord(),
        vc_custom_prefix: default_discord(),
        vc_custom_suffix: default_discord(),
        vc_mandatory_roles: default_features(),
        vc_no_permission: default_discord(),
        vc_category: default_voice(),
    });

    let rebuilt_config = Config {
        logging: logging,
        features: features,
        moderation: moderation,
        voice: voice,
        discord: discord,
        misc: misc,
    };

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("config.toml")?;

    writeln!(file, "{}", toml::to_string(&rebuilt_config).unwrap())?;
    Ok(())
}

// Internal function to update the config after a scan
// Might be useful for a future feature to update the config without restarting the bot
// which matters since this is a Discord bot
pub fn _update_config(key: &str, value: &str, operation: &str) -> io::Result<()> {
    debug!("Updating config key: {} to value: {}", key, value);
    let config_path = "config.toml";
    let config_str = fs::read_to_string(config_path)?;

    // Mutable config because we update it in the match
    // Who knows why the compiler gets mad here
    let mut config: Value = toml::from_str(&config_str).expect("Failed to parse config");

    match key {
        "moderation.moderator_roles" => {
            let mut roles = config["moderation"]["moderator_roles"]
                .as_array()
                .expect("Failed to get moderator roles")
                .clone(); // Clone the array to get a Vec<Value>

            if operation == "add" {
                roles.push(Value::String(value.to_string()));
            } else if operation == "remove" {
                roles.retain(|x| x.as_str().unwrap() != value);
            } else {
                warn!("Invalid operation: {}", operation);
                return Ok(());
            }

            config["moderation"]["moderator_roles"] = Value::Array(roles);
        }
        "moderation.moderator_users" => {
            let mut users = config["moderation"]["moderator_users"]
                .as_array()
                .expect("Failed to get moderator users")
                .clone(); // Clone the array to get a Vec<Value>

            if operation == "add" {
                users.push(Value::String(value.to_string()));
            } else if operation == "remove" {
                users.retain(|x| x.as_str().unwrap() != value);
            } else {
                warn!("Invalid operation: {}", operation);
                return Ok(());
            }

            config["moderation"]["moderator_users"] = Value::Array(users);
        }
        "voice.global_timeout" => {
            let timeout_value = value.parse::<i64>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Failed to parse value")
            })?;
            config["voice"]["global_timeout"] = Value::Integer(timeout_value);
        }
        _ => {
            warn!("Invalid key: {}", key);
            return Ok(());
        }
    }

    Ok(())
}
