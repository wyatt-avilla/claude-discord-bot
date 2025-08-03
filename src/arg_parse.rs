use std::{io::Read, path::PathBuf};

use clap::Parser;

fn validate_nonempty_readable_token_file(s: &str) -> Result<String, String> {
    let path = PathBuf::from(s);

    if !path.exists() {
        return Err(format!("Path '{s}' does not exist"));
    }

    if !path.is_file() {
        return Err(format!("Path '{s}' is not a file"));
    }

    match std::fs::File::open(&path) {
        Ok(mut file) => {
            let mut file_contents = String::new();
            let _ = file
                .read_to_string(&mut file_contents)
                .map_err(|e| format!("Cannot read file '{s}': {e}"))?;

            if file_contents.is_empty() {
                Err(format!("'{s}' is an empty file"))
            } else {
                Ok(file_contents.trim().to_string())
            }
        }
        Err(e) => Err(format!("Cannot read file '{s}': {e}")),
    }
}

/// CLI interface for the Claude Discord bot
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to file containing (only) a Discord token
    #[arg(short('t'), long, value_parser = validate_nonempty_readable_token_file)]
    pub discord_token_file: String,

    /// Log level, one of (INFO, WARN, ERROR, DEBUG, TRACE)
    #[arg(short, long, default_value_t = tracing::Level::INFO)]
    pub log_level: tracing::Level,
}
