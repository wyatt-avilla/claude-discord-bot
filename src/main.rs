mod arg_parse;

use clap::Parser;

fn main() {
    let args = arg_parse::Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .init();

    println!("Hello, world!");
}
