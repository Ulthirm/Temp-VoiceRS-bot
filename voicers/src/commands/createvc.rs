use crate::discord;
use tracing::{info,debug};

use std::time::{SystemTime, UNIX_EPOCH};

async fn autocomplete_type(ctx: discord::Context<'_>, _args: &str) -> Vec<String> {
    vec!["Private".to_string(), "Public".to_string()]
}

/// Create a voice channel for the user to join
/// 
#[poise::command(track_edits, slash_command)]
pub async fn createvc(    
ctx: discord::Context<'_>,
#[description = "Private or Public"]
#[autocomplete = "autocomplete_type"]
vctype: String,
#[description = "Name of the voice channel"]
vcname: Option<String>,
#[description = "Ping a user or role to add them to private VC"]
pingadd1: Option<String>,
#[description = "Ping a user or role to add them to private VC"]
pingadd2: Option<String>,
#[description = "Ping a user or role to add them to private VC"]
pingadd3: Option<String>,
#[description = "Ping a user or role to add them to private VC"]
pingadd4: Option<String>,
#[description = "Ping a user or role to add them to private VC"]
pingadd5: Option<String>,
) -> Result<(), discord::Error>  {
    info!("createvc command called");

    // Clone vcname for the debug statement
    let vcname_for_debug = vcname.clone();
    debug!("received vctype: {} vcname: {} ping1: {} ping2: {} ping3: {} ping4: {} ping5: {}", 
        vctype, 
        vcname_for_debug.unwrap_or_else(|| "None".to_string()), 
        pingadd1.unwrap_or_else(|| "None".to_string()), 
        pingadd2.unwrap_or_else(|| "None".to_string()), 
        pingadd3.unwrap_or_else(|| "None".to_string()), 
        pingadd4.unwrap_or_else(|| "None".to_string()), 
        pingadd5.unwrap_or_else(|| "None".to_string())
    );
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // verify that the VC name is not empty
    let vcname = match vcname {
        Some(name) => name,
        None => format!("{}_{}_{}", ctx.author().name, vctype.to_lowercase(), now),
    };
    
    debug!("naming new VC as: {}", vcname);
    ctx.say("createvc command called").await?;
    Ok(())
    
}