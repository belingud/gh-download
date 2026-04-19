# gh-download

[English](README.md)

`gh-download` 是一个命令行工具，用来从 GitHub 仓库中下载单个文件或整个目录。

它适合这些场景：

- 想直接拿仓库里的某个文件，不想克隆整个仓库
- 想把某个目录拷到本地，但不需要完整的 git 历史
- 想在脚本或终端里快速下载公开仓库或私有仓库内容

## 功能特点

- 支持下载单个文件
- 支持递归下载整个目录
- 支持通过 `--ref` 指定分支、tag 或 commit
- 支持私有仓库访问，可读取 `GITHUB_TOKEN` 或 `GH_TOKEN`
- 支持匿名请求失败时通过代理回退
- 输出友好，错误时会给出明确建议
- 支持英文和中文输出，可按 locale 自动切换，也可显式指定

## 安装指南

### 通过 Cargo 安装

```bash
cargo install gh-download
```

### 下载预编译二进制

从项目的 GitHub Releases 页面下载对应平台的压缩包，解压后即可使用。

当前提供的二进制覆盖：

- macOS Intel
- macOS Apple Silicon
- Linux x86_64
- Linux ARM64
- Windows x86_64

### 从源码构建

```bash
cargo build --release
```

构建完成后，可执行文件位于：

```bash
./target/release/gh-download
```

## 使用示例

基本用法：

```bash
gh-download <repo> <remote-path> <local-target> [--ref <ref>] [--token <token>] [--proxy-base <url>] [--lang <en|zh>] [--no-color]
```

直接运行 `gh-download` 且不带参数时，会按当前生效语言显示帮助信息。

下载单个文件：

```bash
gh-download openai/openai-python README.md ./README.md
```

下载整个目录：

```bash
gh-download owner/repo src ./downloads
```

下载指定分支上的目录：

```bash
gh-download owner/repo docs ./site-docs --ref main
```

下载私有仓库内容：

```bash
gh-download owner/private-repo docs ./docs --token "$GITHUB_TOKEN"
```

强制英文输出：

```bash
gh-download owner/repo docs ./docs --lang en
```

## 配置说明

### 主要参数

- `<repo>`: GitHub 仓库，格式如 `openai/openai-python`
- `<remote-path>`: 仓库内路径，例如 `README.md` 或 `src/openai`
- `<local-target>`: 本地输出路径
- `--ref`: 分支、tag 或 commit SHA
- `--token`: GitHub token
- `--proxy-base`: 匿名请求失败时使用的代理前缀
- `--lang`: 显式指定输出语言，支持 `en` 和 `zh`
- `--no-color`: 关闭 ANSI 彩色输出

### 环境变量

- `GITHUB_TOKEN`: GitHub token，优先于 `GH_TOKEN`
- `GH_TOKEN`: GitHub token 备用变量
- `GH_PROXY_BASE`: 默认代理前缀

### 语言规则

- 默认输出英文
- 如果 `LC_ALL`、`LC_MESSAGES` 或 `LANG` 的有效 locale 指向中文，则自动切换为中文
- `--lang` 优先级最高，可覆盖 locale 检测

## 输出示例

成功输出：

```text
● gh-download
仓库 owner/repo
引用 main
远端 src
本地 /tmp/downloads

↻ 正在读取目录结构...
ℹ 发现 3 个文件
↓ main.rs
↓ nested/lib.rs
↓ nested/mod.rs
✔ 完成，已保存到 /tmp/downloads/src
共下载 3 个文件，跳过 0 个条目
```

错误输出：

```text
✖ 下载失败
原因：GitHub 认证失败，或匿名请求触发了限流（HTTP 403）
建议：
- 设置环境变量 GITHUB_TOKEN 或 GH_TOKEN
- 或使用 --token <token> 重新执行
- 如果直连 GitHub 不稳定，请确认 --proxy-base 可访问
```

## 贡献指南

欢迎提 Issue 和 Pull Request。

本地开发常用命令：

```bash
cargo fmt
cargo test
```

如果你调整了 CLI 行为，尤其是用户可见输出、参数含义或下载规则，建议同步更新 `openspec/` 下对应的规格文档。

## 许可证

本项目基于 [MIT License](LICENSE) 开源。
