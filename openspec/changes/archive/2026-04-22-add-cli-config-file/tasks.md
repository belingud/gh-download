## 1. CLI 与配置入口

- [x] 1.1 在 CLI 类型和帮助文案中加入 `--config <path>`，并补充中英文说明
- [x] 1.2 新增配置文件加载模块，支持默认路径 `~/.config/gh-download/config.toml`、显式 `--config` 和严格 TOML 校验
- [x] 1.3 在完整 clap 解析前加入轻量 bootstrap 流程，让帮助语言按 `--lang` > 配置文件 `lang` > locale 判定

## 2. 选项合并与运行时接线

- [x] 2.1 将 `token`、`api_base`、`proxy_base`、`prefix_mode`、`concurrency` 和 `lang` 按“命令行参数 > 配置文件 > 环境变量 > 默认值”合并到最终 `ResolvedOptions`
- [x] 2.2 保持位置参数必须来自命令行，并在配置文件包含不支持字段或非法值时返回用户可见错误
- [x] 2.3 更新调试来源标记和相关输出逻辑，确保配置文件 token 不会泄露明文

## 3. 测试与文档

- [x] 3.1 为默认配置路径、`--config` 覆盖、配置优先级、语言优先级和非法配置分别补充单元测试
- [x] 3.2 更新 `README.md` 与 `README.zh.md`，说明配置文件格式、支持的键、默认路径、`--config` 语义和优先级
- [x] 3.3 校对相关帮助文本和错误文案，确保中英文行为语义一致

## 4. 验证

- [x] 4.1 运行 `just fmt`
- [x] 4.2 运行 `just test`
- [x] 4.3 运行 `just check`
