use poise::{PrefixFrameworkOptions, serenity_prelude as serenity};
use thiserror::Error;

pub struct CustomData {
    pub db: crate::database::Client,
    pub claude: crate::claude::Client,
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
        claude_client: crate::claude::Client,
    ) -> Result<Bot, DiscordBotError> {
        let intents =
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                event_handler: |ctx, event, framework, data| {
                    Box::pin(super::event_handler::event_handler(
                        ctx, event, framework, data,
                    ))
                },
                prefix_options: PrefixFrameworkOptions {
                    mention_as_prefix: false,
                    ..Default::default()
                },
                commands: vec![
                    super::command::get_config(),
                    super::command::set_api_key(),
                    super::command::set_model(),
                    super::command::set_random_interaction_chance(),
                    super::command::add_active_channel_id(),
                    super::command::clear_active_channels(),
                ],
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(CustomData {
                        db: database_client,
                        claude: claude_client,
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
