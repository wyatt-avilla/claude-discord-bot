use poise::serenity_prelude as serenity;
use thiserror::Error;

pub struct CustomData {
    pub db: crate::database::Client,
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

impl Bot {
    pub async fn new(
        discord_token: &str,
        database_client: crate::database::Client,
    ) -> Result<Bot, DiscordBotError> {
        let intents = serenity::GatewayIntents::non_privileged();
        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![super::command::age(), super::command::get_config()],
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
