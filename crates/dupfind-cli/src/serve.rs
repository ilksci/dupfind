//! 本地 Web 服务器 — 通过浏览器查看重复文件报告。
//!
//! 使用 `axum` 提供：
//! - `GET /` — 交互式仪表盘 HTML 页面
//! - `GET /api/groups` — JSON API 返回重复组
//! - `GET /api/stats` — 统计摘要

use std::sync::Arc;
use std::net::SocketAddr;

use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::get,
    Json, Router,
};
use dupfind_core::DuplicateGroup;
use tower_http::cors::CorsLayer;

/// 应用程序共享状态
#[derive(Clone)]
pub struct AppState {
    pub groups: Arc<Vec<DuplicateGroup>>,
    pub total_dup_count: usize,
    pub wasted_bytes: u64,
}

/// 启动 Web 服务器
pub async fn start_server(groups: Vec<DuplicateGroup>, total_dup_count: usize, wasted_bytes: u64, port: u16) {
    let state = AppState {
        total_dup_count,
        wasted_bytes,
        groups: Arc::new(groups),
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/groups", get(groups_handler))
        .route("/api/stats", get(stats_handler))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("\n🌐 Web 服务器已启动: http://{}", addr);
    println!("   按 Ctrl+C 停止服务器。\n");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("无法绑定端口 {}: {}", port, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("服务器错误: {}", e);
    }
}

/// 主页面 HTML
async fn index_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    Html(build_dashboard_html(&state))
}

/// API: 返回所有重复组
async fn groups_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DuplicateGroup>>, StatusCode> {
    Ok(Json((*state.groups).clone()))
}

/// API: 返回统计摘要
async fn stats_handler(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let wasted_mb = state.wasted_bytes as f64 / 1_048_576.0;
    Json(serde_json::json!({
        "total_groups": state.groups.len(),
        "total_dup_count": state.total_dup_count,
        "wasted_bytes": state.wasted_bytes,
        "wasted_mb": format!("{:.1}", wasted_mb),
    }))
}

/// 生成自包含仪表盘 HTML
fn build_dashboard_html(state: &AppState) -> String {
    let groups_json = serde_json::to_string(&*state.groups).unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>dupfind v3 — 重复文件仪表盘</title>
<style>
*{{box-sizing:border-box;margin:0;padding:0}}
body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#f5f5f5;color:#333;min-height:100vh}}
.header{{background:#fff;padding:20px 30px;border-bottom:1px solid #e0e0e0;box-shadow:0 1px 3px rgba(0,0,0,.06)}}
.header h1{{font-size:24px;color:#1a73e8}}
.stats{{display:flex;gap:20px;margin:24px 30px;flex-wrap:wrap}}
.stat-card{{background:#fff;padding:20px 24px;border-radius:8px;flex:1;min-width:150px;text-align:center;box-shadow:0 1px 3px rgba(0,0,0,.08)}}
.stat-card .num{{font-size:32px;font-weight:700;color:#1a73e8}}
.stat-card .label{{font-size:13px;color:#666;margin-top:4px}}
.container{{margin:0 30px 30px}}
.search{{margin-bottom:16px}}
.search input{{width:100%;padding:10px 16px;border-radius:6px;border:1px solid #ddd;background:#fff;color:#333;font-size:14px;outline:none}}
.search input:focus{{border-color:#1a73e8;box-shadow:0 0 0 3px rgba(26,115,232,.15)}}
.group{{background:#fff;border-radius:8px;padding:16px;margin-bottom:12px;border-left:3px solid #1a73e8;box-shadow:0 1px 3px rgba(0,0,0,.08)}}
.group-header{{display:flex;justify-content:space-between;align-items:center;margin-bottom:10px;flex-wrap:wrap;gap:8px}}
.group-header h3{{font-size:16px;color:#333}}
.group-header .meta{{font-size:12px;color:#888}}
.file-list{{list-style:none}}
.file-list li{{padding:4px 0;font-size:13px;font-family:'Cascadia Code',Consolas,monospace;word-break:break-all;color:#444}}
.hash{{font-size:11px;color:#999;word-break:break-all}}
.empty{{text-align:center;color:#999;padding:40px;font-size:16px}}
footer{{text-align:center;color:#aaa;padding:20px;font-size:12px}}
</style>
</head>
<body>
<div class="header"><h1>🔍 dupfind v3 仪表盘</h1></div>
<div class="stats">
<div class="stat-card"><div class="num">{groups}</div><div class="label">重复组</div></div>
<div class="stat-card"><div class="num">{files}</div><div class="label">重复文件</div></div>
<div class="stat-card"><div class="num">{wasted}</div><div class="label">可释放空间</div></div>
</div>
<div class="container">
<div class="search"><input type="text" id="search" placeholder="搜索文件路径或哈希..." oninput="filter()"></div>
<div id="groups"></div>
</div>
<footer>dupfind v0.3 — Web 仪表盘</footer>
<script>
const DATA = {groups_json};

function formatBytes(b) {{
    const units = ['B','KB','MB','GB','TB'];
    let i = 0, v = b;
    while (v >= 1024 && i < 4) {{ v /= 1024; i++; }}
    return v.toFixed(1) + ' ' + units[i];
}}

function render(groups) {{
    const el = document.getElementById('groups');
    if (!groups.length) {{ el.innerHTML = '<div class="empty">没有找到匹配的重复组。</div>'; return; }}
    el.innerHTML = groups.map((g, i) => `
        <div class="group">
            <div class="group-header">
                <h3>组 ${{i + 1}} — ${{g.files.length}} 个重复文件</h3>
                <span class="meta">每组 ${{formatBytes(g.size)}} | 哈希: <span class="hash">${{g.hash.substring(0,16)}}…</span></span>
            </div>
            <ul class="file-list">${{g.files.map(f => `<li>${{f.path}}</li>`).join('')}}</ul>
        </div>
    `).join('');
}}

function filter() {{
    const q = document.getElementById('search').value.toLowerCase();
    if (!q) {{ render(DATA); return; }}
    const filtered = DATA.filter(g =>
        g.hash.toLowerCase().includes(q) ||
        g.files.some(f => f.path.toLowerCase().includes(q))
    );
    render(filtered);
}}

render(DATA);
</script>
</body></html>"#,
        groups = state.groups.len(),
        files = state.total_dup_count,
        wasted = format_bytes_dashboard(state.wasted_bytes),
        groups_json = groups_json,
    )
}

/// 人类可读的字节表示（仪表盘用）
pub fn format_bytes_dashboard(bytes: u64) -> String {
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
