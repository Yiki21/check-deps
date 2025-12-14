use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StringOrNumber<T> {
    String(String),
    Number(T),
}

pub fn deserialize_number<'de, T, D>(deserialize: D) -> Result<T, D::Error>
where
    T: std::str::FromStr + Deserialize<'de>,
    D: serde::Deserializer<'de>,
    T::Err: Display,
{
    match StringOrNumber::<T>::deserialize(deserialize)? {
        StringOrNumber::String(s) => s
            .parse::<T>()
            .map_err(|e| serde::de::Error::custom(format!("Failed to parse string: {}", e))),
        StringOrNumber::Number(n) => Ok(n),
    }
}
