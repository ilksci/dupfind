# 🔍 dupfind

> 一个用 Rust 编写的本地文件重复查找与清理工具

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## 📖 项目简介

`dupfind` 是一个高性能的命令行工具，用于扫描本地目录、识别重复文件并支持交互式清理。

现有的重复文件查找工具（如 `fdupes`、`dupeGuru`）大多以 C/Python 实现，`dupfind` 探索用 Rust 实现同类工具的可行性，在保证安全性的同时通过多线程并行 Hash 计算大幅提升扫描速度。

**核心功能：**

- 递归扫描指定目录，支持过滤规则（文件大小、扩展名、路径）
- 使用 SHA-256 对文件内容进行 Hash，精准识别重复文件组
- 多线程并行计算，充分利用多核 CPU
- 交互式 TUI 界面，支持逐组确认删除
- 导出重复报告（JSON / CSV）

## 🏗️ 项目结构

```
dupfind/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs           # 程序入口，CLI 参数解析
│   ├── cli.rs            # 命令行接口定义（基于 clap）
│   ├── scanner/
│   │   ├── mod.rs        # 目录递归扫描，收集 FileInfo
│   │   └── filter.rs     # 扫描过滤规则（大小、扩展名、排除路径）
│   ├── hasher/
│   │   ├── mod.rs        # 文件 Hash 计算与重复分组
│   │   └── parallel.rs   # 基于 rayon 的并行 Hash 计算
│   ├── reporter/
│   │   ├── mod.rs        # Reporter trait 定义
│   │   ├── json.rs       # JSON 格式报告输出
│   │   └── csv.rs        # CSV 格式报告输出
│   ├── cleaner/
│   │   ├── mod.rs        # 删除策略与执行
│   │   └── interactive.rs# TUI 交互式确认界面
│   └── error.rs          # 统一错误类型定义
└── tests/
    ├── scanner_test.rs
    ├── hasher_test.rs
    └── reporter_test.rs
```

## 🎯 设计说明

### 模块职责

| 模块 | 职责 |
|------|------|
| `cli` | 解析命令行参数，构建运行配置 |
| `scanner` | 递归遍历文件系统，收集文件元信息 |
| `hasher` | 计算文件 Hash，将相同 Hash 的文件归为一组 |
| `reporter` | 将重复文件组序列化为报告文件 |
| `cleaner` | 根据用户策略执行文件删除操作 |
| `error` | 统一的错误类型，贯穿各模块 |

### 核心数据结构

```rust
// 文件信息
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub hash: Option<String>,
}

// 重复文件组
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<FileInfo>,
}

// 删除策略
pub enum KeepStrategy {
    Newest,       // 保留最新修改的文件
    Oldest,       // 保留最早修改的文件
    Shortest,     // 保留路径最短的文件
    Interactive,  // 交互式手动选择
}

// 报告导出 trait
pub trait Reporter {
    fn write(&self, groups: &[DuplicateGroup], output: &Path) -> Result<()>;
}
```

### 关键技术点

**两阶段 Hash 策略**：先按文件大小粗筛（大小不同必不重复），再对相同大小的文件计算完整 SHA-256，避免对全部文件做 Hash，显著减少 IO 开销。

**并行计算**：使用 `rayon` 对文件列表并行计算 Hash，在多核机器上线性提升速度。

**错误处理**：所有 IO 操作均返回 `Result<T, DupfindError>`，通过 `?` 传播，不使用 `unwrap`。

## 🚀 编译与运行

**环境要求：** Rust 1.75+

```bash
# 克隆项目
git clone https://github.com/ilksci/dupfind.git
cd dupfind

# 编译
cargo build --release

# 运行
./target/release/dupfind --path ~/Downloads

# 常用参数
dupfind --path <目录>           # 扫描指定目录
        --min-size 1MB          # 忽略小于指定大小的文件
        --ext jpg,png,mp4       # 只扫描指定扩展名
        --output report.json    # 导出 JSON 报告
        --output report.csv     # 导出 CSV 报告
        --delete interactive    # 交互式删除
        --delete keep-newest    # 自动保留最新文件
```

## 🧪 测试

```bash
cargo test          # 运行所有测试
cargo test scanner  # 只运行 scanner 模块测试
```

## 📦 依赖

| crate | 用途 |
|-------|------|
| `clap` | 命令行参数解析 |
| `rayon` | 数据并行（多线程 Hash） |
| `sha2` | SHA-256 Hash 计算 |
| `serde` / `serde_json` | JSON 序列化 |
| `csv` | CSV 报告导出 |
| `walkdir` | 递归目录遍历 |
| `indicatif` | 进度条显示 |
| `crossterm` | 跨平台终端控制（TUI） |
| `thiserror` | 错误类型定义 |

## 📄 License

MIT
