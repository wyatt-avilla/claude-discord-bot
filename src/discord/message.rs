use poise::serenity_prelude as serenity;

pub trait NormalizeContent {
    fn normalize_content(&self) -> String;
}

impl NormalizeContent for serenity::Message {
    fn normalize_content(&self) -> String {
        self.mentions.iter().fold(self.content.clone(), |acc, u| {
            acc.replace(
                format!("<@{}>", u.id).as_str(),
                format!("@{}", u.display_name()).as_str(),
            )
        })
    }
}
