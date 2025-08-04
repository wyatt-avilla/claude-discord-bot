use poise::serenity_prelude as serenity;

type CommandError = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, super::client::CustomData, CommandError>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), CommandError> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

/// Displays your server's config
#[poise::command(slash_command, prefix_command)]
pub async fn get_config(ctx: Context<'_>) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    let config = match ctx.data().db.get_config(guild_id.get()) {
        Ok(cfg) => cfg,
        Err(e) => {
            ctx.say(format!("Couldn't fetch server config ({e})"))
                .await?;
            return Ok(());
        }
    };

    ctx.say(config.to_string()).await?;
    Ok(())
}
