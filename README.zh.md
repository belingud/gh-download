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
- 支持通过 `--concurrency` 或 `-c` 并发下载目录中的文件
- 默认跳过本地已存在文件，并支持显式 `--overwrite`
- 支持通过 `--ref` 指定分支、tag 或 commit
- 支持把常用参数持久化到 `~/.config/gh-download/config.toml` 或显式 `--config` 文件
- 支持私有仓库访问，可读取 `GITHUB_TOKEN` 或 `GH_TOKEN`
- 支持 raw 文件下载的显式前缀代理模式
- 支持通过 `--json` 输出机器可读的最终结果
- 支持通过 `--api-base` 配置自定义 GitHub metadata API 端点
- 支持按需开启的请求 URL、token 来源和策略调试输出
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
gh-download <repo> <remote-path> <local-target> [--config <path>] [--ref <ref>] [--token <token>] [--api-base <url>] [--proxy-base <url>] [--prefix-mode <direct|fallback|prefer>] [--concurrency <n>|-c <n>] [--overwrite] [--json] [--lang <en|zh>] [--debug] [--no-color]
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

以更高并发下载目录：

```bash
gh-download owner/repo src ./downloads -c 8
```

显式覆盖本地已存在文件：

```bash
gh-download owner/repo src ./downloads --overwrite
```

输出机器可读 JSON 结果：

```bash
gh-download owner/repo README.md ./README.md --json
```

使用自定义 GitHub metadata API base 下载：

```bash
gh-download owner/repo docs ./docs --api-base https://ghe.example.com/api/v3
```

下载私有仓库内容：

```bash
gh-download owner/private-repo docs ./docs --token "$GITHUB_TOKEN"
```

强制英文输出：

```bash
gh-download owner/repo docs ./docs --lang en
```

使用显式配置文件：

```bash
gh-download owner/repo docs ./docs --config ./gh-download.toml
```

## 配置说明

### 主要参数

- `<repo>`: GitHub 仓库，格式如 `openai/openai-python`
- `<remote-path>`: 仓库内路径，例如 `README.md` 或 `src/openai`
- `<local-target>`: 本地输出路径
- `--config`: 只从这份 TOML 配置文件读取选项。未提供时，如果 `~/.config/gh-download/config.toml` 存在则会读取它
- `--ref`: 分支、tag 或 commit SHA
- `--token`: GitHub token。优先级依次是 `--token`、配置文件、`GITHUB_TOKEN`、`GH_TOKEN`
- `--api-base`: GitHub metadata API 基础地址。优先级依次是 `--api-base`、配置文件、默认值 `https://api.github.com`
- `--proxy-base`: 匿名 raw 文件下载使用的 URL 前缀代理基址。优先级依次是命令行、配置文件、`GH_PROXY_BASE`
- `--prefix-mode`: raw 下载前缀代理模式，`direct`、`fallback` 或 `prefer`。优先级依次是命令行、配置文件、`GH_DOWNLOAD_PREFIX_MODE`
- `--concurrency`、`-c`: 目录下载时的最大并发文件数，最小为 `1`。会先读取配置文件，未设置时默认值为 `4`
- `--overwrite`: 覆盖本地已存在文件，而不是默认跳过
- `--json`: 在 stdout 输出一个最终的机器可读 JSON 结果
- `--lang`: 显式指定输出语言，支持 `en` 和 `zh`。未提供 `--lang` 时，配置文件 `lang` 优先于 locale 检测
- `--debug`: 打印请求 URL、token 来源和策略选择的调试信息
- `--no-color`: 关闭 ANSI 彩色输出

### 配置文件

- 默认路径：`~/.config/gh-download/config.toml`
- 如果传入 `--config <path>`，本次执行只读取这一个文件，不再读取默认路径
- `gh-download` 不会自动创建配置文件
- 支持的键：`token`、`api_base`、`proxy_base`、`prefix_mode`、`concurrency`、`lang`
- 配置文件不支持仓库、远端路径、本地目标路径这类位置参数
- 未知键、非法枚举值和非法类型都会直接报配置错误

示例：

```toml
api_base = "https://api.github.com"
proxy_base = "https://gh-proxy.com/"
prefix_mode = "direct"
concurrency = 4
lang = "zh"
token = "xxxx"
```

如果把 token 写进配置文件，请自行收紧文件权限。

### 环境变量

- 只有当对应的命令行参数和配置文件键都没有设置时，才会读取环境变量
- `GITHUB_TOKEN`: GitHub token，优先于 `GH_TOKEN`
- `GH_TOKEN`: GitHub token 备用变量
- `GH_PROXY_BASE`: 显式覆盖 URL 前缀代理基址
- `GH_DOWNLOAD_PREFIX_MODE`: 默认的 raw 下载前缀代理模式
- `GH_DOWNLOAD_DEBUG`: 为真值时开启调试输出

### 语言规则

- 默认输出英文
- 如果 `LC_ALL`、`LC_MESSAGES` 或 `LANG` 的有效 locale 指向中文，则自动切换为中文
- 优先级依次是 `--lang`、配置文件 `lang`、locale 检测、默认英文

### 前缀代理行为

- `--api-base` 只改变仓库内容探测使用的 GitHub metadata API base
- `--proxy-base`、配置文件 `proxy_base` 和 `GH_PROXY_BASE` 只用于 raw 文件下载 URL，不用于 GitHub metadata API 请求
- 默认模式是 `--prefix-mode direct`
- `--prefix-mode fallback` 会在直连 raw 文件下载出现可重试失败后再走前缀代理；如果未显式设置代理基址，则使用内置的 `https://gh-proxy.com/`
- `--prefix-mode prefer` 会先尝试前缀代理，再在失败后回退到直连 raw 文件 URL；如果未显式设置代理基址，则使用内置的 `https://gh-proxy.com/`
- GitHub metadata API 请求不会发送到 `gh-proxy` 这类 URL 前缀式回退代理
- 当请求带有 token 时，`gh-download` 不会把该凭据转发到公共回退代理
- 当前缀代理重试被触发时，警告输出会打印完整生成的回退 URL，并自动打码其中可能包含的凭据

### 自定义 GitHub API base

- `--api-base` 适用于 GitHub Enterprise 或兼容部署中暴露在不同基础地址上的 GitHub contents API
- CLI 只会把这个 base 用于 `/repos/<owner>/<repo>/contents/...` 这类 metadata 请求
- 即使设置了 `--api-base`，raw 文件下载行为、前缀代理模式和 token 转发规则也保持不变
- debug 输出中的 `metadata-url` 会反映最终生效的自定义 API base，便于排查请求是否打到了预期端点

### 目录下载并发行为

- 当命令行和配置文件都没有指定其他值时，目录下载会先枚举远端目录树，再以默认最多 `4` 个并发传输下载文件
- 可通过 `--concurrency <n>` 或 `-c <n>` 提高或降低目录下载时同时进行的文件数
- 如果需要显式顺序模式，便于排障或降低资源占用，可使用 `--concurrency 1` 或 `-c 1`
- 单文件下载也接受 `--concurrency` 和 `-c`，但最终仍只会下载一个解析出的文件目标
- 并发目录下载会保持与当前版本相同的相对路径落盘结构，只是进度行出现顺序可能变化
- 目录下载开始时的日志会显示本次实际使用的线程数

### 本地写入行为

- 默认会跳过本地已存在文件，而不是隐式覆盖
- 如果希望替换本地已存在文件，可显式传入 `--overwrite`
- 这个规则同时适用于单文件下载和目录下载中的逐文件写入
- 跳过判断只基于最终解析出的本地文件路径是否已存在，不会在该模式下比较文件内容

### Debug 行为

- `--debug` 和 `GH_DOWNLOAD_DEBUG` 会开启流程级调试输出
- 调试输出包含生成的 GitHub metadata URL、识别到的 token 来源标签、解析出的 raw 下载 URL、适用时生成的前缀代理 URL，以及当前选择的 raw 下载策略
- 调试输出写到 `stderr`，不会改变下载行为

### JSON 输出

- `--json` 会把 stdout 切换成一个最终的机器可读 JSON 文档，而不是默认的人类可读启动、进度、完成或错误文本
- JSON 成功结果包含 `success`、`saved_path` 和聚合下载统计信息
- JSON 失败结果会在 `error` 对象下包含 `title`、`reason` 和 `suggestions`
- 即使消息文本跟随当前生效语言，JSON 字段名也保持稳定的英文标识
- 如果同时开启 `--json` 和 `--debug`，JSON 保持在 stdout，debug 诊断信息继续输出到 stderr

### CI 与发布

- `.github/workflows/ci.yml` 会在普通 `push` 和 `pull_request` 上执行标准 `just check` 校验
- `.github/workflows/release.yml` 仍然只负责基于 tag 的打包与发布
- 常规 CI 不会构建归档文件，也不会发布 GitHub Release

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
🔎 发现 3 个文件，目录：src，使用 3 个线程
📁 创建本地目录：/tmp/downloads/src
-------------------------------------
⬇️ 下载：main.rs
⬇️ 下载：nested/lib.rs
⬇️ 下载：nested/mod.rs
-------------------------------------
✅ 完成：owner/repo 的 src 已保存到 /tmp/downloads/src
共下载 3 个文件，跳过 0 个已存在文件，跳过 0 个不支持条目
```

跳过已存在文件的输出：

```text
-------------------------------------
📦 仓库：owner/repo
🌿 分支：默认分支
📂 远端路径：README.md
💾 本地路径：/tmp/README.md
-------------------------------------
⏭ 跳过已存在文件：README.md
-------------------------------------
✅ 完成：owner/repo 的 README.md 已保存到 /tmp/README.md
共下载 0 个文件，跳过 1 个已存在文件，跳过 0 个不支持条目
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
[debug] token-source: GITHUB_TOKEN
[debug] download-url: https://raw.githubusercontent.com/owner/repo/main/README.md
[debug] prefix-url: https://gh-proxy.com/https://raw.githubusercontent.com/owner/repo/main/README.md
[debug] raw-download-strategy: prefix-proxy
```

JSON 输出：

```json
{
  "success": true,
  "saved_path": "/tmp/README.md",
  "stats": {
    "files_downloaded": 1,
    "skipped_existing_files": 0,
    "skipped_unsupported_entries": 0
  }
}
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
