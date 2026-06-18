# 🔍 dupfind

> 高性能重复文件查找与清理工具，用 Rust 编写

[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)](https://www.rust-lang.org/)
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

## 🏗️ 项目结构

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

**环境要求：** Rust 1.88+

```bash
git clone git@github.com:ilksci/dupfind.git
cd dupfind
cargo build --release
```

> **注意：** 编译成功后 `dupfind` 不会自动加入系统 PATH。可以通过以下方式运行：
>
> ```powershell
> # 方式一：直接指定可执行文件路径
> .\target\release\dupfind.exe --path D:\03-Documents
>
> # 方式二：通过 cargo run（-- 之后的参数传给程序，而非 cargo）
> cargo run --release -- --path D:\03-Documents
>
> # 方式三：安装到系统，之后可在任意位置直接使用 dupfind 命令
> cargo install --path .
> ```

## 📋 使用指南

### 基本扫描

```powershell
# 扫描指定目录
dupfind --path D:\03-Documents

# 省略 --path 则扫描当前目录
dupfind
```

执行时会显示三阶段进度条：

```
⠋ 正在扫描目录 [00:02] 已发现 4523 个文件（跳过 128 个）
扫描完成: 4523 个文件
  文件类型分布:
    image/png                     2341
    text/plain                     892
    ...

使用 BLAKE3 算法计算哈希...
前缀哈希筛选 [00:01] [████████████░░░░░░░░░░░░] 2100/3210
哈希计算 [00:03] [████████████████░░░░░░░░░░░░] 523/856 (62%, 预计 2s)

发现 127 个重复组，共 384 个重复文件，可释放约 2.34 GB。
```

### 过滤参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `-m`, `--min-size` | 忽略小于指定大小的文件，支持 KB/MB/GB | `--min-size 1MB` |
| `-e`, `--ext` | 只扫描指定扩展名，逗号分隔 | `-e jpg,png,mp4` |
| `-x`, `--exclude` | 排除路径中包含指定字符串的文件/目录 | `-x node_modules -x .git` |
| `--type` | 按文件类型过滤（魔术字节识别），逗号分隔 | `--type image,video` |

支持的类型：`image`、`video`、`audio`、`document`、`archive`、`font`、`text` 等。

```powershell
# 只扫描大于 1MB 的图片
dupfind --path D:\Photos -m 1MB -e jpg,png

# 扫描文档目录，排除 node_modules 和 .git
dupfind --path D:\Projects -x node_modules -x .git --type document,image

# 排除系统目录
dupfind --path D:\03-Documents -x "System Volume Information" -x "$RECYCLE.BIN" -m 10KB
```

### 导出报告

使用 `-o` / `--output` 导出重复文件报告，格式由扩展名自动选择：

| 格式 | 扩展名 | 说明 |
|------|--------|------|
| JSON | `.json` | 结构化数据，适合程序处理 |
| CSV | `.csv` | 表格数据，适合 Excel 打开 |
| HTML | `.html` | 可视化报告，浏览器直接查看 |

```powershell
# JSON 报告
dupfind --path D:\03-Documents -o report.json

# CSV 报告（Excel 打开）
dupfind --path D:\03-Documents -o duplicates.csv

# HTML 可视化报告
dupfind --path D:\03-Documents -o report.html

# 组合过滤 + 导出
dupfind --path D:\Photos --min-size 100KB --ext jpg,png,gif -o photo_duplicates.html
dupfind --path D:\Projects -x node_modules -x target -x .git -o project_dupes.json
dupfind --path D:\Videos --min-size 10MB --type video -o video_dupes.csv
```

### 终端表格

`-t` / `--table` 直接在终端打印重复文件组表格：

```powershell
dupfind --path D:\03-Documents --table
```

```
+-------+--------------+-----------+------------------------------------------+
| group | hash         | size      | path                                     |
+-------+--------------+-----------+------------------------------------------+
| 1     | a1b2c3d4e5f6 | 1048576   | D:\Documents\photo.jpg                   |
| 1     | a1b2c3d4e5f6 | 1048576   | D:\Documents\backup\photo_copy.jpg       |
+-------+--------------+-----------+------------------------------------------+
```

### 哈希算法

```powershell
# BLAKE3（默认）：速度快，适合大文件
dupfind --path D:\03-Documents --hash-algo blake3

# SHA-256：密码学安全，兼容性更好
dupfind --path D:\03-Documents --hash-algo sha256

# 异步 I/O 路径（tokio）：大量小文件时更快
dupfind --path D:\03-Documents --async
```

### 相似文件检测

`--similar` 开启相似文件检测，`--threshold` 设置相似度阈值（0-100，默认 90）：

```powershell
# 查找相似度 ≥ 85% 的文件
dupfind --path D:\03-Documents --similar --threshold 85

# 只检测相似图片
dupfind --path D:\Photos --similar --type image --threshold 90
```

- **图片**：感知哈希（dHash），视觉相似即分组，无视尺寸/格式差异
- **文本**：归一化后编辑距离，内容相近即分组

### 配置文件

在目录下创建 `.dupfind.toml`，避免每次输入重复参数：

```toml
path = "D:\\03-Documents"
min_size = "1MB"
extensions = ["jpg", "png", "mp4"]
exclude = ["node_modules", ".git", "target"]
hash_algo = "blake3"
```

CLI 参数会覆盖配置文件中的同名设置。

### 清理重复文件

```powershell
# 安全预览（不实际删除）
dupfind --path D:\03-Documents --delete keep-newest --dry-run

# 交互式逐组选择
dupfind --path D:\03-Documents --delete interactive

# 自动保留最新文件，移入回收站
dupfind --path D:\03-Documents --delete keep-newest --trash

# 自动保留最大文件
dupfind --path D:\03-Documents --delete keep-largest
```

### Web 仪表盘

```powershell
# 启动 Web 服务器，浏览器查看
dupfind --path D:\03-Documents --serve

# 指定端口
dupfind --path D:\03-Documents --serve 3000
```

### 完整参数列表

```
dupfind --path <目录>            # 扫描目录（默认 .）
        --min-size 1MB           # 最小文件大小过滤
        --ext jpg,png,mp4        # 只扫描指定扩展名
        --exclude node_modules   # 排除路径
        --type image,video       # 按文件类型过滤（魔术字节识别）
        --hash-algo sha256       # 哈希算法（blake3 / sha256）
        --async                  # tokio 异步 I/O 计算哈希
        --output report.json     # 导出报告（json / csv / html）
        --table                  # 终端表格输出
        --delete interactive     # 交互式删除（ratatui 仪表盘）
        --delete keep-newest     # 自动保留最新文件
        --delete keep-largest    # 自动保留最大文件
        --similar                # 相似文件检测模式
        --threshold 90           # 相似度阈值（默认 90）
        --serve 8080             # Web 仪表盘（默认 8080）
        --dry-run                # 安全预览，不实际删除
        --trash                  # 移入回收站（而非永久删除）
        --config .dupfind.toml   # 指定配置文件路径
        -v / -vv                 # 详细日志
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
