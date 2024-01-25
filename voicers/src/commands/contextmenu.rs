use crate::discord;
use poise::serenity_prelude as serenity;

/// Query information about a Discord profile
#[poise::command(context_menu_command = "User information", slash_command)]
pub async fn user_info(
    ctx: discord::Context<'_>,
    #[description = "Discord profile to query information about"] user: serenity::User,
) -> Result<(), discord::Error> {
    let response = format!(
        "**Name**: {}\n**Created**: {}",
        user.name,
        user.created_at()
    );

    ctx.say(response).await?;
    Ok(())
}
