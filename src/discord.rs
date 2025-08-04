use poise::serenity_prelude as serenity;
use thiserror::Error;

type CommandError = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, CustomData, CommandError>;

struct CustomData {
    db: super::database::Client,
}

pub struct Bot {
    client: serenity::Client,
}

#[derive(Debug, Error)]
pub enum DiscordBotError {
    #[error("Couldn't create client ({0})")]
    Creation(serenity::Error),

    #[error("Couldn't start client ({0})")]
    Start(serenity::Error),
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), CommandError> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

impl Bot {
    pub async fn new(
        discord_token: &str,
        database_client: super::database::Client,
    ) -> Result<Bot, DiscordBotError> {
        let intents = serenity::GatewayIntents::non_privileged();
        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![age()],
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(CustomData {
                        db: database_client,
                    })
                })
            })
            .build();

        let client = serenity::ClientBuilder::new(discord_token, intents)
            .framework(framework)
            .await
            .map_err(DiscordBotError::Creation)?;

        Ok(Self { client })
    }

    pub async fn run(&mut self) -> Result<(), DiscordBotError> {
        self.client.start().await.map_err(DiscordBotError::Start)
    }
}
