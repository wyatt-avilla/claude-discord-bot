mod arg_parse;
mod database;
mod discord;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = arg_parse::Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .init();

    let db_client = database::Client::new()?;

    let mut bot = discord::Bot::new(&args.discord_token_file, db_client).await?;
    bot.run().await?;

    Ok(())
}
