use serde::Deserialize;
use std::path::PathBuf;

/// 配置文件结构（对应 .dupfind.toml）
#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub path: Option<PathBuf>,
    pub min_size: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub hash_algo: Option<String>,
}

impl Config {
    /// 从给定路径或默认位置加载配置，文件不存在返回 None
    pub fn load(explicit: Option<&PathBuf>) -> Option<Self> {
        let paths: Vec<PathBuf> = match explicit {
            Some(p) => vec![p.clone()],
            None => {
                let mut v = vec![PathBuf::from(".dupfind.toml")];
                if let Some(mut home) = dirs_next() {
                    home.push("dupfind");
                    home.push("config.toml");
                    v.push(home);
                }
                v
            }
        };
        for p in paths {
            if p.exists() {
                let content = std::fs::read_to_string(&p).ok()?;
                log::info!("加载配置文件: {}", p.display());
                return toml::from_str(&content).ok();
            }
        }
        None
    }
}

/// 获取用户配置目录（跨平台）
fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}
