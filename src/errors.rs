use failure::Fail;
use std::io;

/// well-defined Result
pub type Result<T> = std::result::Result<T, KVStoreError>;

#[derive(Fail, Debug)]
/// 自定义KVStoreError枚举类型 well-defined Error
pub enum KVStoreError {
    /// Io error
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    /// Serde error
    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),

    /// Key not found error
    #[fail(display = "Key not found")]
    KeyNotFound,

    /// Unknown command type error
    #[fail(display = "Unknown command type")]
    UnknownCommandType,

    /// Unknown engine type error
    #[fail(display = "Change engine after initialization")]
    ChangeEngineError,

    /// common string error
    #[fail(display = "{}", _0)]
    CommonStringError(String),

}

/// 为自定义错误实现 From trait，代表 io::Error -> KVStoreError
impl From<io::Error> for KVStoreError {
    fn from(err: io::Error) -> Self {
        KVStoreError::Io(err)
    }
}

/// 为自定义错误实现 From trait，代表 serde_json::Error -> KVStoreError
impl From<serde_json::Error> for KVStoreError {
    fn from(err: serde_json::Error) -> Self {
        KVStoreError::Serde(err)
    }
}
