# 🔍 dupfind

> 高性能重复文件查找与清理工具，用 Rust 编写

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## 📖 简介

`dupfind` 是一个命令行工具，用于扫描本地目录、识别重复文件并支持安全清理。

**核心功能：**

- 递归扫描目录，支持过滤规则（大小、扩展名、路径排除、**文件类型**）
- **三级去重策略**：大小分桶 → 前缀哈希（4 KiB）→ 完整哈希
- 多算法支持：**BLAKE3**（默认，快速）和 **SHA-256**（密码学安全）
- 基于 `rayon` 的并行哈希计算，充分利用多核 CPU
- **tokio 异步 I/O** 可选路径（`--async`）
- 交互式 **ratatui TUI** 仪表盘，支持逐组确认删除
- **相似文件检测**：图片感知哈希 + 文本编辑距离（`--similar`）
- **魔术字节识别**：自动检测文件真实类型，支持 `--type` 过滤
- **安全删除**：`--dry-run` 预览模式 + `--trash` 回收站支持
- 导出报告：JSON / CSV / **HTML**（自包含，浏览器可查看）
- **本地 Web 仪表盘**（`--serve`）— 浏览器查看重复文件报告
- 终端表格直出（`--table`）
- 符号链接检测与跳过
- 配置文件支持（`.dupfind.toml`）
- 结构化日志（`-v` / `-vv`）

## 🏗️ 项目结构（v3 工作区）

```
dupfind/
├── Cargo.toml              # 工作区根 + 二进制入口
├── README.md
├── .github/workflows/ci.yml # CI/CD
├── benches/                 # criterion 基准测试
├── tests/                   # 集成测试 + proptest 属性测试
├── src/                     # 根 package（重导出 + bin）
└── crates/
    ├── dupfind-core/        # 核心类型、错误、HashAlgorithm/Reporter trait
    ├── dupfind-scanner/     # 目录扫描、文件过滤、魔术字节识别
    ├── dupfind-hasher/      # 三级去重引擎、并行哈希、异步 I/O、相似检测
    ├── dupfind-reporter/    # JSON / CSV / HTML 报告导出
    ├── dupfind-cleaner/     # 自动/交互式清理（ratatui 仪表盘）
    └── dupfind-cli/         # CLI 参数、配置、表格输出、顶层调度、Web 服务器
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
/// 哈希算法 trait — v3 可扩展更多算法
pub trait HashAlgorithm: Send + Sync {
    fn hash(&self, reader: &mut dyn Read) -> io::Result<String>;
    fn name(&self) -> &'static str;
}

/// 报告导出 trait
pub trait Reporter {
    fn write(&self, groups: &[DuplicateGroup], output: &Path) -> Result<()>;
}
```

| 特性 | 说明 |
|------|------|
| **三级去重** | 大小分桶 → 前缀哈希（4 KiB）→ 完整哈希，大幅减少无效 I/O |
| **BLAKE3 / SHA-256** | 可插拔哈希算法，BLAKE3 多线程友好比 SHA-256 快 5-10x |
| **并行哈希** | 基于 rayon 的并行计算 + tokio 异步 I/O（`--async`） |
| **ratatui TUI** | 交互式仪表盘，逐组确认删除 |
| **安全删除** | `--dry-run` 预览 + `--trash` 回收站，双重保护 |
| **魔术字节识别** | 检测文件真实类型，支持 `--type image,video` 过滤 |
| **相似文件检测** | `--similar` 图片感知哈希 + 文本编辑距离 |
| **多格式报告** | JSON / CSV / 自包含 HTML |
| **Web 仪表盘** | `--serve` 启动本地 Web 服务器，浏览器查看报告 |
| **终端表格** | `-t` 直接在终端打印结果 |
| **配置文件** | `.dupfind.toml` 支持，命令行参数互补 |
| **符号链接** | 自动检测并跳过，防止误操作 |
| **结构化日志** | `-v` / `-vv` + `RUST_LOG` 环境变量 |
| **跨平台** | Windows / macOS / Linux 全支持 |
| **CI/CD** | GitHub Actions 三平台矩阵构建 + 安全审计 |
| **属性测试** | proptest 验证哈希算法确定性 |

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
        --type image,video       # 按文件类型过滤（魔术字节识别）
        --hash-algo sha256       # 选择哈希算法（默认 blake3）
        --async                  # 使用 tokio 异步 I/O 计算哈希
        --output report.json     # 导出报告（支持 json/csv/html）
        --table                  # 终端表格输出
        --delete interactive     # 交互式删除（ratatui 仪表盘）
        --delete keep-newest     # 自动保留最新文件
        --delete keep-largest    # 自动保留最大文件
        --similar                # 相似文件检测模式
        --threshold 90           # 相似度阈值（默认 90%）
        --serve 8080             # 启动 Web 仪表盘（默认端口 8080）
        --dry-run                # 安全预览（不实际删除）
        --trash                  # 移入回收站
        -v                       # 详细日志
```

## 🧪 测试

```bash
cargo test --workspace       # 全部测试（31+ 个）
cargo test -p dupfind-core   # 核心模块测试
cargo test -p dupfind-scanner # 扫描模块测试
cargo test -p dupfind-hasher # 哈希 + 相似检测测试
cargo test -p dupfind-reporter # 报告模块测试
cargo test -p dupfind-cli    # CLI 参数解析测试
cargo bench                  # 基准测试（hash + scan）
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
| `crossterm` + `ratatui` | 跨平台 TUI 仪表盘 |
| `thiserror` | 错误类型派生 |
| `toml` | 配置文件解析 |
| `log` + `env_logger` | 结构化日志 |
| `trash` | 系统回收站 |
| `tabled` | 终端表格 |
| `infer` | 魔术字节文件类型识别 |
| `tokio` | 异步 I/O 运行时 |
| `axum` + `tower-http` | Web 仪表盘服务器 |
| `image` + `img_hash` | 图片感知哈希相似检测 |
| `criterion` | 基准测试 |
| `proptest` | 属性测试 |

## 📄 License

MIT
