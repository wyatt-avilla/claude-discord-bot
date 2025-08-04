use bincode::{self, Decode, Encode};
use itertools::Itertools;
use redb::Value;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Display, num::NonZeroU64};

#[derive(Debug, Serialize, Deserialize, Decode, Encode, Default)]
pub struct Record {
    pub claude_api_key: Option<String>,
    pub random_interaction_chance_denominator: Option<NonZeroU64>,
    pub active_channel_ids: HashSet<u64>,
}

impl Record {
    pub fn get_encoding_config() -> bincode::config::Configuration {
        bincode::config::standard()
            .with_little_endian()
            .with_variable_int_encoding()
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unset = String::from("**Not Set**");

        let claude_api_key = &self.claude_api_key.as_ref().map(|key| {
            if key.len() <= 4 {
                key.clone()
            } else {
                "\\*".repeat(key.len() - 4) + &key[key.len() - 4..]
            }
        });

        let interaction_chance = self
            .random_interaction_chance_denominator
            .map(|denom| format!("1/{denom}"));

        let active_channel_ids = format!(
            "[{}]",
            self.active_channel_ids
                .clone()
                .iter()
                .map(std::string::ToString::to_string)
                .join(", ")
        );

        let lines = vec![
            format!(
                "Claude API key: {}",
                claude_api_key.clone().unwrap_or(unset.clone())
            ),
            format!(
                "Interaction chance: {}",
                interaction_chance.unwrap_or(unset.clone())
            ),
            format!(
                "Active channel ids: {}",
                if active_channel_ids.is_empty() {
                    unset.clone()
                } else {
                    active_channel_ids
                }
            ),
        ];

        f.write_str(lines.into_iter().join("\n").as_str())
    }
}

impl Value for Record {
    type SelfType<'a>
        = Record
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::decode_from_slice(data, Record::get_encoding_config())
            .unwrap()
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        bincode::encode_to_vec(value, Record::get_encoding_config()).unwrap()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("claude_discord_bot_record")
    }
}
