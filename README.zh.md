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
- 支持 raw 文件下载的显式前缀代理模式
- 支持按需开启的请求 URL 和策略调试输出
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
gh-download <repo> <remote-path> <local-target> [--ref <ref>] [--token <token>] [--proxy-base <url>] [--prefix-mode <direct|fallback|prefer>] [--lang <en|zh>] [--debug] [--no-color]
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
- `--proxy-base`: raw 文件下载重试或 prefer 模式使用的 URL 前缀代理基址
- `--prefix-mode`: raw 下载前缀代理模式，`direct`、`fallback` 或 `prefer`
- `--lang`: 显式指定输出语言，支持 `en` 和 `zh`
- `--debug`: 打印请求 URL 和策略选择的调试信息
- `--no-color`: 关闭 ANSI 彩色输出

### 环境变量

- `GITHUB_TOKEN`: GitHub token，优先于 `GH_TOKEN`
- `GH_TOKEN`: GitHub token 备用变量
- `GH_PROXY_BASE`: 显式覆盖 URL 前缀代理基址
- `GH_DOWNLOAD_PREFIX_MODE`: 默认的 raw 下载前缀代理模式
- `GH_DOWNLOAD_DEBUG`: 为真值时开启调试输出

### 语言规则

- 默认输出英文
- 如果 `LC_ALL`、`LC_MESSAGES` 或 `LANG` 的有效 locale 指向中文，则自动切换为中文
- `--lang` 优先级最高，可覆盖 locale 检测

### 前缀代理行为

- `--proxy-base` 和 `GH_PROXY_BASE` 只用于 raw 文件下载 URL，不用于 GitHub metadata API 请求
- 默认模式是 `--prefix-mode direct`
- `--prefix-mode fallback` 会在直连 raw 文件下载出现可重试失败后再走前缀代理；如果未显式设置代理基址，则使用内置的 `https://gh-proxy.com/`
- `--prefix-mode prefer` 会先尝试前缀代理，再在失败后回退到直连 raw 文件 URL；如果未显式设置代理基址，则使用内置的 `https://gh-proxy.com/`
- GitHub metadata API 请求不会发送到 `gh-proxy` 这类 URL 前缀式回退代理
- 当请求带有 token 时，`gh-download` 不会把该凭据转发到公共回退代理
- 当前缀代理重试被触发时，警告输出会打印完整生成的回退 URL，并自动打码其中可能包含的凭据

### Debug 行为

- `--debug` 和 `GH_DOWNLOAD_DEBUG` 会开启流程级调试输出
- 调试输出包含生成的 GitHub metadata URL、解析出的 raw 下载 URL、适用时生成的前缀代理 URL，以及当前选择的 raw 下载策略
- 调试输出写到 `stderr`，不会改变下载行为

推荐方式：

- 对外保持默认 `direct` 模式，兼顾开源场景下的可移植性
- 如果希望直连失败后再让 raw 文件 URL 走内置 gh-proxy，可设置 `GH_DOWNLOAD_PREFIX_MODE=fallback`
- 如果希望 raw 文件 URL 一开始就优先走内置 gh-proxy，可设置 `GH_DOWNLOAD_PREFIX_MODE=prefer`
- 只有在你想覆盖内置前缀代理时，才需要设置 `GH_PROXY_BASE=...`

## 输出示例

成功输出：

```text
-------------------------------------
📦 仓库：owner/repo
🌿 分支：main
📂 远端路径：src
💾 本地路径：/tmp/downloads
-------------------------------------
🔎 发现 3 个文件，目录：src
📁 创建本地目录：/tmp/downloads/src
-------------------------------------
⬇️ 下载：main.rs
⬇️ 下载：nested/lib.rs
⬇️ 下载：nested/mod.rs
-------------------------------------
✅ 完成：owner/repo 的 src 已保存到 /tmp/downloads/src
共下载 3 个文件，跳过 0 个条目
```

前缀代理输出：

```text
-------------------------------------
📦 仓库：owner/repo
🌿 分支：默认分支
📂 远端路径：README.md
💾 本地路径：/tmp/README.md
-------------------------------------
⚠ 直连文件下载失败，正在通过前缀代理重试：https://gh-proxy.com/https://raw.githubusercontent.com/OWNER/REPO/REF/README.md
⬇️ 下载：README.md
-------------------------------------
✅ 完成：owner/repo 的 README.md 已保存到 /tmp/README.md
```

Debug 输出：

```text
[debug] metadata-url: https://api.github.com/repos/owner/repo/contents/README.md
[debug] download-url: https://raw.githubusercontent.com/owner/repo/main/README.md
[debug] prefix-url: https://gh-proxy.com/https://raw.githubusercontent.com/owner/repo/main/README.md
[debug] raw-download-strategy: prefix-proxy
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
