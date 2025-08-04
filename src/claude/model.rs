use std::fmt::Display;

pub enum Model {
    Opus4,
    Sonnet4,
    Sonnet37,
    Sonnet35,
    Haiku35,
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Model::Opus4 => "claude-opus-4-0",
            Model::Sonnet4 => "claude-sonnet-4-0",
            Model::Sonnet37 => "claude-3-7-sonnet-latest",
            Model::Sonnet35 => "claude-3-5-sonnet-latest",
            Model::Haiku35 => "claude-3-5-haiku-latest",
        })
    }
}
