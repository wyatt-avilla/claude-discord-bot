pub enum ErrorReply {
    CantSeeReplies,
    InactiveChannel,
    MissingAPIKey,
    SomethingWentWrong,
    MaxTokens,
    TermsOfServiceViolation,
}

impl ErrorReply {
    pub fn pretty_str(&self) -> &'static str {
        match self {
            ErrorReply::CantSeeReplies => {
                "*Claude can't see replies. View the tracking issue* [here](<https://github.com/wyatt-avilla/claude-discord-bot/issues/18>)."
            }
            ErrorReply::InactiveChannel => {
                "*Claude isn't configured to be active in this channel.*"
            }
            ErrorReply::MissingAPIKey => "*Anthropic API key not set.*",
            ErrorReply::SomethingWentWrong => "*An error occurred while Claude tried to respond*",
            ErrorReply::MaxTokens => {
                "*Claude hit the max amount of tokens while trying to respond*"
            }
            ErrorReply::TermsOfServiceViolation => {
                "*Content in this interaction violates Anthropic's terms of service*"
            }
        }
    }
}
