# 🔍 dupfind

> 高性能重复文件查找与清理工具，用 Rust 编写

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## 📖 简介

`dupfind` 是一个命令行工具，用于扫描本地目录、识别重复文件并支持安全清理。

**核心功能：**

- 递归扫描目录，支持过滤规则（大小、扩展名、路径排除）
- **三级去重策略**：大小分桶 → 前缀哈希（4 KiB）→ 完整哈希
- 多算法支持：**BLAKE3**（默认，快速）和 **SHA-256**（密码学安全）
- 基于 `rayon` 的并行哈希计算，充分利用多核 CPU
- 交互式 TUI 界面，支持逐组确认删除
- **安全删除**：`--dry-run` 预览模式 + `--trash` 回收站支持
- 导出报告：JSON / CSV / **HTML**（自包含，浏览器可查看）
- 终端表格直出（`--table`）
- 符号链接检测与跳过
- 配置文件支持（`.dupfind.toml`）
- 结构化日志（`-v` / `-vv`）

## 🏗️ 项目结构

```
dupfind/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # 程序入口
│   ├── lib.rs               # 顶层调度（五阶段流水线）
│   ├── cli.rs               # 命令行参数（clap derive）
│   ├── config.rs            # 配置文件加载（.dupfind.toml）
│   ├── error.rs             # 统一错误类型（thiserror）
│   ├── table.rs             # 终端表格输出（tabled）
│   ├── scanner/
│   │   ├── mod.rs           # 递归扫描 + 符号链接检测
│   │   └── filter.rs        # 过滤规则（大小/扩展名/路径）
│   ├── hasher/
│   │   ├── mod.rs           # 三级去重策略调度
│   │   ├── algorithms.rs    # HashAlgorithm trait + SHA256/BLAKE3 实现
│   │   └── parallel.rs      # rayon 并行 + channel 流水线（演示用）
│   ├── reporter/
│   │   ├── mod.rs           # Reporter trait + 格式调度
│   │   ├── json.rs          # JSON 报告
│   │   ├── csv.rs           # CSV 报告
│   │   └── html.rs          # HTML 自包含报告（新增）
│   └── cleaner/
│       ├── mod.rs           # 清理策略 + dry-run + trash
│       └── interactive.rs   # TUI 交互式界面
└── tests/                   # 集成测试
```

## 🎯 设计说明

### 三级去重策略

```
全部文件 → [阶段1] 大小分桶（唯一大小丢弃）
       → [阶段2] 前缀哈希（首 4 KiB，SHA-256 碰撞检测）
       → [阶段3] 完整哈希（BLAKE3 或 SHA-256 最终确认）
       → 重复文件组
```

### 核心抽象

```rust
/// 哈希算法 trait — 便于 v3 扩展更多算法
pub trait HashAlgorithm: Send + Sync {
    fn hash(&self, reader: &mut dyn Read) -> io::Result<String>;
    fn name(&self) -> &'static str;
}

/// 报告导出 trait
pub trait Reporter {
    fn write(&self, groups: &[DuplicateGroup], output: &Path) -> Result<()>;
}

/// 清理配置
pub struct CleanOptions {
    pub dry_run: bool,    // 预览模式
    pub use_trash: bool,  // 回收站模式
}
```

### v2 新增特性（vs v1）

| 特性 | 说明 |
|------|------|
| 三级去重 | 前缀哈希预筛选，减少完整哈希的 IO 开销 |
| BLAKE3 | 多线程友好，比 SHA-256 快 5-10x |
| HashAlgorithm trait | 算法可插拔，v3 可加 XXHash/MD5 |
| dry-run | 安全预览，只显示不删除 |
| trash | 移入系统回收站，可恢复 |
| HTML 报告 | 自包含 HTML，浏览器可直接查看 |
| 终端表格 | `-t` 直接在终端打印结果 |
| 配置文件 | `.dupfind.toml` 支持 |
| 日志系统 | `-v`/`-vv` + `RUST_LOG` 环境变量 |
| 符号链接检测 | 自动跳过并报告 |
| channel 流水线 | 演示 `thread` + `mpsc` + `Arc<Mutex>` |
| 新保留策略 | Largest / Smallest |

### v3 预计优化

- Workspace 拆分（core / scanner / hasher / reporter / cleaner / cli）
- ratatui 仪表盘升级
- tokio 异步 IO
- 本地 Web 服务器（`--serve`）
- 相似文件/图片感知哈希检测
- 文件类型魔术字节识别
- CI/CD 配置
- criterion 基准测试 + proptest 属性测试

## 🚀 编译与运行

**环境要求：** Rust 1.75+

```bash
git clone git@github.com:ilksci/dupfind.git
cd dupfind
cargo build --release

# 基础用法
./target/release/dupfind --path ~/Downloads

# 常用参数
dupfind --path <目录>            # 扫描目录
        --min-size 1MB           # 最小文件大小过滤
        --ext jpg,png,mp4        # 只扫描指定扩展名
        --exclude node_modules   # 排除路径
        --hash-algo sha256       # 选择哈希算法（默认 blake3）
        --output report.json     # 导出报告（支持 json/csv/html）
        --table                  # 终端表格输出
        --delete interactive     # 交互式删除
        --delete keep-newest     # 自动保留最新文件
        --delete keep-largest    # 自动保留最大文件
        --dry-run                # 安全预览（不实际删除）
        --trash                  # 移入回收站
        -v                       # 详细日志
```

## 🧪 测试

```bash
cargo test          # 全部测试（21 个）
cargo test scanner  # 扫描模块测试
cargo test hasher   # 哈希模块测试
cargo test reporter # 报告模块测试
```

## 📦 主要依赖

| crate | 用途 |
|-------|------|
| `clap` | 命令行参数解析 |
| `rayon` | 数据并行哈希 |
| `sha2` / `blake3` | 哈希算法 |
| `serde` / `serde_json` | JSON 序列化 |
| `csv` | CSV 报告 |
| `walkdir` | 递归目录遍历 |
| `indicatif` | 进度条 |
| `crossterm` | 跨平台 TUI |
| `thiserror` | 错误类型派生 |
| `toml` | 配置文件解析 |
| `log` + `env_logger` | 结构化日志 |
| `trash` | 系统回收站 |
| `tabled` | 终端表格 |

## 📄 License

MIT
