use integral_enum::integral_enum;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[integral_enum]
pub enum LogLevel {
    Debug,
    Info,
    Error,
    Disabled,
}

entity! {
    struct LoggingConfig {
        level: LogLevel
    }
}
