use std::fs;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use crate::error::Result;
use crate::hasher::DuplicateGroup;
use crate::reporter::Reporter;

pub struct HtmlReporter;

impl Reporter for HtmlReporter {
    fn write(&self, groups: &[DuplicateGroup], output: &Path) -> Result<()> {
        let file = fs::File::create(output)?;
        let mut w = BufWriter::new(file);

        write!(w, "{}", build_html(groups))?;
        Ok(())
    }
}

/// 生成自包含 HTML 报告
fn build_html(groups: &[DuplicateGroup]) -> String {
    let total_files: usize = groups.iter().map(|g| g.files.len()).sum();
    let wasted_bytes: u64 = groups
        .iter()
        .map(|g| g.size * (g.files.len() - 1) as u64)
        .sum();

    let mut groups_html = String::new();
    for (i, g) in groups.iter().enumerate() {
        let mut files_html = String::new();
        for f in &g.files {
            files_html.push_str(&format!(
                "<li class=\"file\"><code>{}</code></li>\n",
                escape_html(&f.path.to_string_lossy())
            ));
        }

        groups_html.push_str(&format!(
            r#"<div class="group">
<h3>🔍 组 {idx} — {count} 个重复文件 — 每组占用 {size}</h3>
<p class="hash">SHA-256: <code>{hash}</code></p>
<ol>{files}</ol>
</div>
"#,
            idx = i + 1,
            count = g.files.len(),
            size = format_bytes(g.size),
            hash = g.hash,
            files = files_html,
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="UTF-8"><title>dupfind 重复文件报告</title>
<style>
body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;max-width:960px;margin:0 auto;padding:20px;background:#f5f5f5}}
.summary{{background:#fff;padding:20px;border-radius:8px;box-shadow:0 2px 4px rgba(0,0,0,.1);margin-bottom:20px}}
.summary h2{{margin-top:0;color:#333}}
.summary-stats{{display:flex;gap:20px;flex-wrap:wrap}}
.stat{{flex:1;min-width:120px;background:#f0f4ff;padding:12px;border-radius:6px;text-align:center}}
.stat .num{{font-size:28px;font-weight:700;color:#1a73e8}}
.stat .label{{font-size:13px;color:#666;margin-top:4px}}
.group{{background:#fff;padding:16px;border-radius:8px;box-shadow:0 1px 3px rgba(0,0,0,.08);margin-bottom:12px}}
.group h3{{margin-top:0;font-size:16px;color:#444}}
.hash{{font-size:12px;color:#999;word-break:break-all}}
.file{{padding:4px 0;font-size:14px}}
code{{background:#f0f0f0;padding:1px 6px;border-radius:3px;font-size:13px}}
</style></head>
<body>
<div class="summary">
<h2>📊 扫描摘要</h2>
<div class="summary-stats">
<div class="stat"><div class="num">{total_groups}</div><div class="label">重复组数</div></div>
<div class="stat"><div class="num">{total_files}</div><div class="label">重复文件总数</div></div>
<div class="stat"><div class="num">{wasted}</div><div class="label">可释放空间</div></div>
</div></div>
{groups}
<footer style="text-align:center;color:#aaa;margin-top:30px;font-size:13px">由 dupfind v0.2 生成</footer>
</body></html>"#,
        total_groups = groups.len(),
        total_files = total_files,
        wasted = format_bytes(wasted_bytes),
        groups = groups_html,
    )
}

/// 人类可读的字节表示
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[(&str, f64)] = &[
        ("TB", 1_099_511_627_776.0),
        ("GB", 1_073_741_824.0),
        ("MB", 1_048_576.0),
        ("KB", 1_024.0),
        ("B", 1.0),
    ];
    for (unit, div) in UNITS {
        let val = bytes as f64 / div;
        if val >= 1.0 || *unit == "B" {
            return format!("{:.1} {}", val, unit);
        }
    }
    format!("{bytes} B")
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
