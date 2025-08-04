use std::num::NonZeroU64;

use poise::serenity_prelude as serenity;

type CommandError = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, super::client::CustomData, CommandError>;

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

/// Sets the Claude API key
#[poise::command(slash_command, prefix_command)]
pub async fn set_api_key(
    ctx: Context<'_>,
    #[description = "API key from the Anthropic console"] api_key: String,
) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    ctx.data().db.set_claude_api_key(guild_id.get(), &api_key)?;

    Ok(())
}

/// Sets the random interaction chance
#[poise::command(slash_command, prefix_command)]
pub async fn set_random_interaction_chance(
    ctx: Context<'_>,
    #[description = "The denominator for the `1/denominator` chance that Claude reacts on a per-message basis"]
    denominator: NonZeroU64,
) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    ctx.data()
        .db
        .set_random_interaction_denominator(guild_id.get(), denominator)?;

    Ok(())
}

/// Add a channel id to the list of Claude's active channels
#[poise::command(slash_command, prefix_command)]
pub async fn add_active_channel_id(
    ctx: Context<'_>,
    #[description = "The channel ID. You can right click on a channel to find its ID."]
    channel_id: serenity::ChannelId,
) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    ctx.data()
        .db
        .add_active_channel_id(guild_id.get(), channel_id.get())?;

    Ok(())
}
