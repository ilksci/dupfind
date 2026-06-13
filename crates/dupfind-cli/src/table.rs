use tabled::Tabled;

use dupfind_core::DuplicateGroup;

/// 终端表格行
#[derive(Tabled)]
pub struct DuplicateRow {
    /// 组号
    pub group: usize,
    /// 哈希值（截短显示）
    pub hash: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 文件路径
    pub path: String,
}

/// 在终端以表格形式打印重复文件组
pub fn print_table(groups: &[DuplicateGroup]) {
    let mut rows: Vec<DuplicateRow> = Vec::new();

    for (gi, group) in groups.iter().enumerate() {
        let short_hash = &group.hash[..usize::min(12, group.hash.len())];
        for file in &group.files {
            rows.push(DuplicateRow {
                group: gi + 1,
                hash: short_hash.to_string(),
                size: group.size,
                path: file.path.to_string_lossy().to_string(),
            });
        }
    }

    let table = tabled::Table::new(rows).to_string();
    println!("{table}");
}
