//! 相似文件检测模块（v3 新增）。
//!
//! 支持两种模式：
//! 1. **图片感知哈希** — 使用 `img_hash` 计算图片的 dHash 感知哈希，
//!    对视觉上相似的图片进行分组（即使尺寸、格式不同）。
//! 2. **文本相似度** — 基于归一化文本的编辑距离，检测内容相近的文本文件。

use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Read};

use dupfind_core::FileInfo;

/// 相似文件组
#[derive(Debug, Clone)]
pub struct SimilarGroup {
    /// 相似度阈值（0-100）
    pub threshold: u8,
    /// 组内文件
    pub files: Vec<FileInfo>,
    /// 相似依据描述
    pub reason: String,
}

/// 图片感知哈希检测相似图片
///
/// 使用 `img_hash` 库的 DoubleGradient 算法：
/// - 将图片缩小为 9×8 灰度像素
/// - 计算相邻像素梯度差
/// - 生成 64 位哈希
/// - 汉明距离 ≤ threshold 视为相似
pub fn find_similar_images(files: &[FileInfo], threshold: u8) -> Vec<SimilarGroup> {
    let hasher = img_hash::HasherConfig::new()
        .hash_size(8, 8)
        .hash_alg(img_hash::HashAlg::DoubleGradient)
        .to_hasher();

    // 收集图片哈希
    let mut entries: Vec<(img_hash::ImageHash<Box<[u8]>>, FileInfo)> = Vec::new();

    for f in files {
        if let Ok(img) = image::open(&f.path) {
            let hash = hasher.hash_image(&img);
            entries.push((hash, f.clone()));
        }
    }

    if entries.len() < 2 {
        return vec![];
    }

    // 汉明距离分组
    let mut used = vec![false; entries.len()];
    let mut groups: Vec<SimilarGroup> = Vec::new();

    for i in 0..entries.len() {
        if used[i] {
            continue;
        }

        let mut group_files = vec![entries[i].1.clone()];
        used[i] = true;

        for j in (i + 1)..entries.len() {
            if used[j] {
                continue;
            }

            let dist = entries[i].0.dist(&entries[j].0);
            if dist <= threshold as u32 {
                group_files.push(entries[j].1.clone());
                used[j] = true;
            }
        }

        if group_files.len() >= 2 {
            groups.push(SimilarGroup {
                threshold,
                files: group_files,
                reason: "图片感知哈希 (dHash)".into(),
            });
        }
    }

    groups
}

/// 文本相似度检测
///
/// 策略：
/// 1. 归一化文本（去除空白归一化、转小写）
/// 2. 对相同大小的文件，比较归一化后内容的编辑距离
/// 3. 相似度 = 1 - (编辑距离 / max(len_a, len_b)) ≥ threshold%
pub fn find_similar_text(files: &[FileInfo], threshold: u8) -> Vec<SimilarGroup> {
    // 按大小分桶（相差超过 10% 的不可能相似）
    let mut size_buckets: HashMap<u64, Vec<&FileInfo>> = HashMap::new();
    for f in files {
        size_buckets.entry(f.size).or_default().push(f);
    }

    let mut groups: Vec<SimilarGroup> = Vec::new();

    for bucket in size_buckets.values() {
        if bucket.len() < 2 {
            continue;
        }

        let contents: Vec<(String, &FileInfo)> = bucket
            .iter()
            .filter_map(|f| {
                let content = normalize_text(f)?;
                Some((content, *f))
            })
            .collect();

        let mut used = vec![false; contents.len()];

        for i in 0..contents.len() {
            if used[i] {
                continue;
            }

            let mut group_files = vec![contents[i].1.clone()];
            used[i] = true;

            for j in (i + 1)..contents.len() {
                if used[j] {
                    continue;
                }

                let similarity = text_similarity(&contents[i].0, &contents[j].0);
                if similarity >= threshold {
                    group_files.push(contents[j].1.clone());
                    used[j] = true;
                }
            }

            if group_files.len() >= 2 {
                groups.push(SimilarGroup {
                    threshold,
                    files: group_files,
                    reason: format!("文本相似度 ≥ {}%", threshold),
                });
            }
        }
    }

    groups
}

/// 读取并归一化文本文件内容
fn normalize_text(f: &FileInfo) -> Option<String> {
    let file = fs::File::open(&f.path).ok()?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content).ok()?;

    // 归一化：转小写，压缩空白
    let normalized: String = content
        .to_lowercase()
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    let words: Vec<&str> = normalized.split_whitespace().collect();
    Some(words.join(" "))
}

/// 计算两个文本的相似度（0-100）
fn text_similarity(a: &str, b: &str) -> u8 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 100;
    }
    let dist = edit_distance(a, b);
    let similarity = 100.0_f64 * (1.0 - dist as f64 / max_len as f64);
    similarity.round() as u8
}

/// 莱文斯坦编辑距离
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = a_chars.len();
    let m = b_chars.len();

    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in dp.iter_mut().enumerate().take(n + 1) {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate().take(m + 1) {
        *cell = j;
    }

    for i in 1..=n {
        for j in 1..=m {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[n][m]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("abc", "abd"), 1);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("abc", ""), 3);
    }

    #[test]
    fn test_text_similarity() {
        assert_eq!(text_similarity("hello world", "hello world"), 100);
        assert_eq!(text_similarity("a", "b"), 0);
    }
}
