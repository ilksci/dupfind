use std::ffi::OsStr;
use std::path::Path;

/// 扫描阶段的过滤规则
#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    /// 最小文件大小（字节）
    pub min_size: Option<u64>,
    /// 允许的扩展名（小写，不含点），空 vec 表示允许全部
    pub extensions: Vec<String>,
    /// 路径排除模式，路径中包含任一字符串则跳过
    pub exclude_patterns: Vec<String>,
    /// 文件类型过滤器（如 ["image", "video"]），None 表示不过滤
    pub type_filter: Option<Vec<String>>,
}

impl FilterConfig {
    /// 判定文件是否通过所有过滤规则
    pub fn matches(&self, path: &Path, size: u64, detected_type: Option<&str>) -> bool {
        // 大小检查
        if let Some(min) = self.min_size {
            if size < min {
                return false;
            }
        }

        // 扩展名检查
        if !self.extensions.is_empty() {
            if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                let ext_lower = ext.to_lowercase();
                if !self
                    .extensions
                    .iter()
                    .any(|e| e.as_str() == ext_lower.as_str())
                {
                    return false;
                }
            } else {
                return false; // 无扩展名则拒绝
            }
        }

        // 路径排除检查
        if !self.exclude_patterns.is_empty() {
            let path_str = path.to_string_lossy();
            if self
                .exclude_patterns
                .iter()
                .any(|pat| path_str.contains(pat.as_str()))
            {
                return false;
            }
        }

        // 文件类型检查
        if let Some(ref type_filter) = self.type_filter {
            let matched = match detected_type {
                Some(dt) => {
                    let dt_lower = dt.to_lowercase();
                    type_filter
                        .iter()
                        .any(|tf| dt_lower.contains(&tf.to_lowercase()))
                }
                None => false,
            };
            if !matched {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_size_filter() {
        let cfg = FilterConfig {
            min_size: Some(100),
            ..Default::default()
        };
        assert!(cfg.matches(Path::new("a.txt"), 200, None));
        assert!(!cfg.matches(Path::new("a.txt"), 50, None));
    }

    #[test]
    fn test_extension_filter() {
        let cfg = FilterConfig {
            extensions: vec!["jpg".into(), "png".into()],
            ..Default::default()
        };
        assert!(cfg.matches(Path::new("photo.jpg"), 100, None));
        assert!(cfg.matches(Path::new("img.PNG"), 100, None));
        assert!(!cfg.matches(Path::new("doc.txt"), 100, None));
        assert!(!cfg.matches(Path::new("noext"), 100, None));
    }

    #[test]
    fn test_exclude_filter() {
        let cfg = FilterConfig {
            exclude_patterns: vec!["node_modules".into(), ".git".into()],
            ..Default::default()
        };
        assert!(!cfg.matches(
            PathBuf::from("proj/node_modules/pkg/file.js").as_path(),
            100,
            None
        ));
        assert!(cfg.matches(Path::new("src/main.rs"), 100, None));
    }

    #[test]
    fn test_type_filter() {
        let cfg = FilterConfig {
            type_filter: Some(vec!["image".into()]),
            ..Default::default()
        };
        // 模拟 image/jpeg 检测结果
        assert!(cfg.matches(Path::new("photo.jpg"), 1000, Some("image/jpeg (jpg)")));
        // 文本文件不会匹配 image 过滤器
        assert!(!cfg.matches(Path::new("notes.txt"), 100, Some("text/plain (txt)")));
        // 无法识别的文件不匹配
        assert!(!cfg.matches(Path::new("unknown.bin"), 100, None));
    }
}
