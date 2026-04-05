use std::num::NonZeroU64;

use poise::serenity_prelude::{self as serenity, Mentionable};

use crate::claude::Model;
use crate::discord::{CommandError, PoiseContext};

/// Displays your server's config
#[poise::command(slash_command)]
pub async fn get_config(ctx: PoiseContext<'_>) -> Result<(), CommandError> {
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
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn set_api_key(
    ctx: PoiseContext<'_>,
    #[description = "API key from the Anthropic console"] api_key: String,
) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    ctx.data().db.set_claude_api_key(guild_id.get(), &api_key)?;

    ctx.say("API key set").await?;

    Ok(())
}

/// Sets the Claude Model
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn set_model(
    ctx: PoiseContext<'_>,
    #[description = "Model name"] model: Model,
) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    ctx.data().db.set_model(guild_id.get(), model.clone())?;

    ctx.say(format!("Model set to '{}'", model.pretty_name()))
        .await?;

    Ok(())
}

/// Sets the random interaction chance
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn set_random_interaction_chance(
    ctx: PoiseContext<'_>,
    #[description = "The `1/denominator` chance that Claude reacts on a per-message basis. Set to 0 to disable."]
    denominator: u64,
) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    let denominator = NonZeroU64::new(denominator);

    ctx.data()
        .db
        .set_random_interaction_denominator(guild_id.get(), denominator)?;

    if let Some(d) = denominator {
        ctx.say(format!("Interaction chance set to 1/{d} per message"))
            .await?;
    } else {
        ctx.say("Disabled random interactions").await?;
    }

    Ok(())
}

/// Add a channel to the set of Claude's active channels
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn add_active_channel(
    ctx: PoiseContext<'_>,
    #[description = "The channel"]
    #[channel_types("Text")]
    channel: serenity::Channel,
) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    let channel_id = channel.id();

    ctx.data()
        .db
        .add_active_channel_id(guild_id.get(), channel_id.get())?;

    ctx.say(format!("Added {} to the set of active channels", {
        channel_id.mention()
    }))
    .await?;

    Ok(())
}

/// Remove a channel from the set of Claude's active channels
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn remove_active_channel(
    ctx: PoiseContext<'_>,
    #[description = "The channel"]
    #[channel_types("Text")]
    channel: serenity::Channel,
) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    let channel_id = channel.id();

    ctx.data()
        .db
        .remove_active_channel_id(guild_id.get(), channel_id.get())?;

    ctx.say(format!("Removed {} from the set of active channels", {
        channel_id.mention()
    }))
    .await?;

    Ok(())
}

/// Clears the set of active channel IDs
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn clear_active_channels(ctx: PoiseContext<'_>) -> Result<(), CommandError> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Couldn't get server id").await?;
        return Ok(());
    };

    ctx.data().db.clear_active_channel_ids(guild_id.get())?;

    ctx.say("Cleared the set of active channel ids").await?;

    Ok(())
}
