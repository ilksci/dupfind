use thiserror::Error;

/// 统一错误类型
#[derive(Error, Debug)]
pub enum DupfindError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("目录遍历错误: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("CSV 报告错误: {0}")]
    Csv(#[from] csv::Error),

    #[error("JSON 序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("配置文件解析错误: {0}")]
    Config(#[from] toml::de::Error),

    #[error("回收站操作失败: {0}")]
    Trash(String),

    #[error("无效的大小格式: '{0}'，请使用数字加可选后缀 B/KB/MB/GB")]
    InvalidSize(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DupfindError>;
