use regex::Regex;
use serde::Deserialize;
use std::ops::Deref;

pub fn deserialize_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Regex::new(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct DeserializableRegex(#[serde(deserialize_with = "deserialize_regex")] Regex);

impl Deref for DeserializableRegex {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
