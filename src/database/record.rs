use bincode::{self, Decode, Encode};
use redb::Value;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

#[derive(Debug, Serialize, Deserialize, Decode, Encode, Default)]
pub struct Record {
    claude_api_key: Option<String>,
    random_interaction_chance_denominator: Option<NonZeroU64>,
}

impl Record {
    pub fn get_encoding_config() -> bincode::config::Configuration {
        bincode::config::standard()
            .with_little_endian()
            .with_variable_int_encoding()
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
