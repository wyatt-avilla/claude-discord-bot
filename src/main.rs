mod arg_parse;
mod database;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = arg_parse::Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .init();

    let db_client = database::Client::new()?;

    db_client.set_config(0, &database::Record::default())?;
    let rec = db_client.get_config(0)?;
    dbg!(rec);

    Ok(())
}
