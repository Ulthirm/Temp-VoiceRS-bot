use crate::discord;

#[poise::command(track_edits, slash_command)]
pub async fn setusermod(
    ctx: discord::Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), discord::Error> {
    Ok(())
}
