use crate::{config, discord};
use std::sync::Arc;
use tracing::error;
/*
use serenity::all::{
    ChannelType, CreateChannel, PermissionOverwrite, PermissionOverwriteType, Permissions, RoleId,
    UserId,GuildId
};
*/

use poise::serenity_prelude as serenity;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

async fn autocomplete_type(ctx: discord::Context<'_>, _args: &str) -> Vec<String> {
    vec!["Private".to_string(), "Public".to_string()]
}

// Check if the user has the admin role or another role set by the server owner
// If they do then they can create a VC
// if they dont then respond with the message (vc_no_permission) set in the config
async fn is_admin_or_approved(ctx: discord::Context<'_>) -> Result<bool, discord::Error> {
    let vcmisc_config = config::get_vcmisc_config();

    let approved_role_ids = &config::get_config().misc.vc_mandatory_roles;

    // Convert the string role IDs to RoleId objects, ignoring invalid entries
    let approved_role_ids: Vec<serenity::RoleId> = approved_role_ids
        .iter()
        .filter_map(|id_str| {
            if id_str.is_empty() {
                None // Skip empty strings
            } else {
                // Parse the string as u64, and then convert to RoleId
                id_str.parse::<u64>().ok().map(serenity::RoleId::from)
            }
        })
        .collect();

    // This does black magic BUT the basic gist is
    // 1. Get the member object of the user who sent the message
    // 2a. Check if the member has the admin role
    // 2b. Check if the member has any of the roles set in the config
    // 3. If the member has the admin role or any of the roles set in the config then return true
    // 4. If the member does not have the admin role or any of the roles set in the config then return false
    // 4a. Respond with the config message (vc_no_permission)
    match ctx.author_member().await {
        Some(member) => {
            let is_admin = member
                .permissions
                .map(serenity::Permissions::administrator)
                .unwrap_or_default();
            let is_approved = member
                .roles
                .iter()
                .any(|role_id| approved_role_ids.contains(role_id));
            debug!(
                "is_admin: {} is_approved: {} responseMessage: {}",
                is_admin, is_approved, vcmisc_config.vc_no_permission
            );
            if !is_admin && !is_approved {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("{}", vcmisc_config.vc_no_permission))
                        .ephemeral(true),
                )
                .await?;
            }
            Ok(is_admin || is_approved)
        }
        None => Ok(false),
    }
}

/// Create a voice channel for the user to join
#[poise::command(
    rename = "createvc",
    track_edits,
    slash_command,
    check = "is_admin_or_approved"
)]
pub async fn entrance(
    ctx: discord::Context<'_>,
    #[description = "Private or Public"]
    #[autocomplete = "autocomplete_type"]
    vctype: String,
    #[description = "Name of the voice channel"] vcname: Option<String>,
    #[description = "Ping a user or role to add them to private VC"] pingadd1: Option<String>,
    #[description = "Ping a user or role to add them to private VC"] pingadd2: Option<String>,
    #[description = "Ping a user or role to add them to private VC"] pingadd3: Option<String>,
    #[description = "Ping a user or role to add them to private VC"] pingadd4: Option<String>,
    #[description = "Ping a user or role to add them to private VC"] pingadd5: Option<String>,
) -> Result<(), discord::Error> {
    info!("createvc command called");
    // Clone vcname for the debug statement
    let vcname_for_debug = vcname.clone();
    debug!(
        "received vctype: {} vcname: {} ping1: {} ping2: {} ping3: {} ping4: {} ping5: {}",
        vctype,
        vcname_for_debug.as_ref().unwrap_or(&"None".to_string()),
        pingadd1.as_ref().unwrap_or(&"None".to_string()),
        pingadd2.as_ref().unwrap_or(&"None".to_string()),
        pingadd3.as_ref().unwrap_or(&"None".to_string()),
        pingadd4.as_ref().unwrap_or(&"None".to_string()),
        pingadd5.as_ref().unwrap_or(&"None".to_string())
    );

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // verify that the VC name is not empty
    // if it is then use the author's name, the type of VC, and the current time to create a unique name
    let vcname = match vcname {
        Some(name) => name,
        None => format!("{}_{}_{}", ctx.author().name, vctype.to_lowercase(), now),
    };

    let mut user_ids = Vec::new();
    let mut role_ids = Vec::new();

    process_mention(pingadd1.as_ref(), &mut user_ids, &mut role_ids);
    process_mention(pingadd2.as_ref(), &mut user_ids, &mut role_ids);
    process_mention(pingadd3.as_ref(), &mut user_ids, &mut role_ids);
    process_mention(pingadd4.as_ref(), &mut user_ids, &mut role_ids);
    process_mention(pingadd5.as_ref(), &mut user_ids, &mut role_ids);

    debug!("naming new VC as: {}", vcname);

    create_voice_channel(ctx, &vcname, user_ids, role_ids, now).await?;

    Ok(())
}

// Helper function to process a single mention
fn process_mention(
    mention: Option<&String>,
    user_ids: &mut Vec<serenity::UserId>,
    role_ids: &mut Vec<serenity::RoleId>,
) {
    if let Some(mention) = mention {
        if mention.starts_with("<@&") {
            // Role mention
            if let Ok(role_id) = mention
                .trim_start_matches("<@&")
                .trim_end_matches('>')
                .parse::<u64>()
            {
                role_ids.push(serenity::RoleId::from(role_id));
            }
        } else if mention.starts_with("<@") {
            // User mention
            if let Ok(user_id) = mention
                .trim_start_matches("<@")
                .trim_end_matches('>')
                .parse::<u64>()
            {
                user_ids.push(serenity::UserId::from(user_id));
            }
        }
    }
}

async fn create_voice_channel(
    ctx: discord::Context<'_>,
    vcname: &str,
    user_ids: Vec<serenity::UserId>, // Vector of up to 5 user IDs
    role_ids: Vec<serenity::RoleId>, // Vector of role IDs
    now: u64,
) -> Result<(), discord::Error> {
    let vcmisc_config = config::get_vcmisc_config();

    let vcrules = format!("{}", vcmisc_config.vc_rules);
    let vccustomprefix = format!("{}", vcmisc_config.vc_custom_prefix);
    let vccustomsuffix = format!("{}", vcmisc_config.vc_custom_suffix);

    ctx.send(poise::CreateReply::default()
        .content(format!("{} \nCreating a new voice channel named: {} \n The mods will have direct access to this channel \n Please be sure to follow all the rules and guidelines of the server {} \n{}",vccustomprefix, vcname,vcrules,vccustomsuffix))
        .ephemeral(true)).await?;

    let mut permissions = Vec::new();

    // Permissions for specific users
    for user_id in user_ids {
        permissions.push(serenity::PermissionOverwrite {
            allow: serenity::Permissions::VIEW_CHANNEL,
            deny: serenity::Permissions::empty(),
            kind: serenity::PermissionOverwriteType::Member(user_id),
        });
    }

    // Permissions for specific roles
    for role_id in role_ids {
        permissions.push(serenity::PermissionOverwrite {
            allow: serenity::Permissions::VIEW_CHANNEL,
            deny: serenity::Permissions::empty(),
            kind: serenity::PermissionOverwriteType::Role(role_id),
        });
    }

    let moderator_role_ids = &config::get_config().moderation.moderator_roles;

    // Permissions for moderator roles
    for role_id_str in moderator_role_ids {
        if let Ok(role_id) = role_id_str.parse::<u64>() {
            let role_id = serenity::RoleId::from(role_id);
            permissions.push(serenity::PermissionOverwrite {
                allow: serenity::Permissions::VIEW_CHANNEL | serenity::Permissions::MANAGE_CHANNELS,
                deny: serenity::Permissions::empty(),
                kind: serenity::PermissionOverwriteType::Role(role_id),
            });
        }
        // Optionally, handle the case where parsing fails
    }

    // Retrieve the guild ID
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            // Handle the error appropriately, e.g., log an error and return
            return Err(discord::Error::from("Command must be used in a guild."));
        }
    };

    // Permissions for the @everyone role
    permissions.push(serenity::PermissionOverwrite {
        allow: serenity::Permissions::empty(),
        deny: serenity::Permissions::VIEW_CHANNEL,
        kind: serenity::PermissionOverwriteType::Role(serenity::GuildId::everyone_role(&guild_id)),
    });

    // Permission for the user who sent the request
    permissions.push(serenity::PermissionOverwrite {
        allow: serenity::Permissions::VIEW_CHANNEL,
        deny: serenity::Permissions::empty(),
        kind: serenity::PermissionOverwriteType::Member(ctx.author().id),
    });

    // Creating the channel builder
    let vc_builder = serenity::CreateChannel::new(vcname)
        .kind(serenity::ChannelType::Voice) // Set the channel type to Voice
        .permissions(permissions); // Optional: Set permissions

    // Using the builder to create the channel
    match guild_id.create_channel(&ctx, vc_builder).await {
        Ok(channel) => {
            // Assuming channel is of type GuildChannel
            let channel_id_i64 = channel.id.get() as i64; // Convert ChannelId to i64

            let pool = &ctx.data().pool;
            let now_i64 = now as i64; // Convert `u64` to `i64`

            debug!("Insert table query for channel ID: {}", channel_id_i64);
            let insert_table_query =
                sqlx::query("INSERT INTO users (vc_id, last_update, user_count) VALUES (?, ?, ?)")
                    .bind(channel_id_i64)
                    .bind(now_i64)
                    .bind(0);

            // Execute the query
            debug!("Executing insert table query");
            match insert_table_query.execute(&**pool).await {
                Ok(_) => debug!("Successfully inserted data into users table"),
                Err(e) => error!("Failed to insert data into users table: {:?}", e),
            }
        }
        Err(e) => {
            error!("Failed to create voice channel: {:?}", e);
            // Handle the error as needed
        }
    }

    Ok(())
}
