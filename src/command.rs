use serde::{Deserialize, Serialize};

/// 支持序列化的 Command结构体 a struct which supports serialization and deserialization
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    /// SET 命令 for set command
    SET(String, String),
    /// RM 命令 for rm command
    RM(String),
}