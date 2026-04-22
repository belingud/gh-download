## Why

`gh-download` 现在只能靠命令行参数和一部分环境变量传配置。对经常重复使用同一组参数的用户，这会带来重复输入，也让 `api_base`、`prefix_mode`、`concurrency`、`lang` 和 `token` 这类长期偏好缺少稳定的持久化入口。

## What Changes

- 增加可选的 TOML 配置文件，默认路径为 `~/.config/gh-download/config.toml`，用于持久化常用 CLI 选项。
- 增加 `--config <path>`，允许用户显式指定本次执行要读取的配置文件；指定后不再读取默认配置文件路径。
- 规定配置来源优先级为：命令行参数 > 配置文件 > 环境变量 > 内置默认值。
- 允许配置文件覆盖现有环境变量来源，包括 `token`、`api_base`、`proxy_base`、`prefix_mode`、`concurrency` 和 `lang`，同时保持位置参数仍然必须由命令行提供。
- 更新中英文文档和相关 spec，说明配置文件格式、支持的键、查找规则、报错行为和新的优先级。

## Capabilities

### New Capabilities
- `cli-config-file`: 定义配置文件的路径发现、TOML 格式、支持的键、报错规则，以及与命令行参数和环境变量的合并顺序。

### Modified Capabilities
- `github-path-download`: 扩展 CLI 合同，支持 `--config` 和默认配置文件，并调整 `api_base`、`token`、`proxy_base` 与 `lang` 的来源优先级。
- `prefix-proxy-mode`: 扩展前缀代理模式的配置来源，允许从配置文件读取，并明确其优先级高于环境变量。
- `download-concurrency`: 扩展并发度来源，允许从配置文件读取有效并发值，同时保留无配置时的默认值和现有校验规则。

## Impact

- CLI 解析与选项归一化：`src/cli.rs`、`src/cli/types.rs`、`src/cli/resolve.rs`、`src/cli/help.rs`
- 可能新增配置加载模块，并增加 TOML 解析依赖
- 用户文档：`README.md`、`README.zh.md`
- OpenSpec：新增 `cli-config-file`，并更新 `github-path-download`、`prefix-proxy-mode`、`download-concurrency`
