## Context

这次 change 不是单纯多一个 `--config` 开关，而是把一部分 CLI 可选参数从“临时输入”扩展成“可持久化默认值”。它会同时影响参数解析、帮助文本语言判定、现有环境变量优先级、调试输出中的来源标记，以及中英文文档里的用户合同。

当前实现里，选项解析分成两段：`parse_cli_from_env` 先根据 `--lang` 和 locale 选择帮助语言，再由 clap 做完整参数解析；`resolve_cli` 再把环境变量和默认值并到最终 `ResolvedOptions`。这套结构对纯 CLI + 环境变量已经够用，但配置文件会带来一个新问题：`lang` 也可能来自配置文件，因此帮助文本阶段也必须能读到配置文件，不能等完整解析结束后再决定语言。

## Goals / Non-Goals

**Goals:**
- 增加一个可选、显式、规则稳定的配置文件入口，用来持久化少量长期默认值。
- 支持默认配置路径 `~/.config/gh-download/config.toml` 和显式 `--config <path>`。
- 明确并落实统一优先级：命令行参数 > 配置文件 > 环境变量 > 内置默认值。
- 保持位置参数仍由命令行提供，不把仓库、远端路径、本地路径搬进配置文件。
- 保持现有下载合同不变，包括 metadata-first 判断、raw 下载代理边界、流式写盘，以及 JSON/debug 的输出规则。

**Non-Goals:**
- 不自动生成配置文件，不新增 `init`、`config set` 一类管理命令。
- 不把所有 CLI 开关都塞进配置文件，只支持少量适合长期默认值的字段。
- 不引入新的环境变量命名空间去镜像所有配置项。
- 不重构下载主流程，也不改变现有下载与代理策略本身。

## Decisions

### 1. 使用平铺 TOML 配置，并对键名和类型做严格校验

配置文件使用顶层平铺的 TOML 键，例如：

```toml
api_base = "https://api.github.com"
prefix_mode = "direct"
concurrency = 4
lang = "zh"
token = "xxxx"
```

实现上使用专门的配置结构体承接这些字段，并开启未知字段报错。支持的键限定为 `token`、`api_base`、`proxy_base`、`prefix_mode`、`concurrency`、`lang`。`repo`、`remote_path`、`local_target` 这类位置参数不允许出现在配置文件中。

Why:
- 这个项目参数面不大，平铺结构最直接，文档也最短。
- 仓库已经使用 `serde`，接入 TOML 解析的成本低，类型约束清楚。
- 严格报错能避免把拼写错误或错误类型静默吞掉。

Alternatives considered:
- INI：放弃，因为格式太松，类型表达弱，后续加字段时兼容面更差。
- 分节 TOML 或更复杂的嵌套结构：放弃，因为当前没有按功能分组的必要。

### 2. 默认配置文件按“存在即读取”处理，显式 `--config` 按“指定即唯一来源”处理

当用户没有传 `--config` 时，CLI 只在默认路径存在时读取 `~/.config/gh-download/config.toml`；文件不存在时直接跳过，不报错，也不主动创建。

当用户传了 `--config <path>` 时，CLI 只读取这一个文件，不再额外读取默认路径。若该文件不存在、不可读或格式非法，命令直接失败。

Why:
- 这和你要的“用户自己决定是否持久化”一致，不把配置管理做成另一个产品面。
- `--config` 的语义更清楚：本次执行显式指定使用这份配置，而不是“再加一个候选来源”。

Alternatives considered:
- 首次运行自动生成配置文件：放弃，因为会增加隐式副作用，也超出当前需求。
- `--config` 与默认路径一起合并：放弃，因为来源过多时排错更难，也不符合“指定即使用”的直觉。

### 3. 合并规则固定为：命令行参数 > 配置文件 > 环境变量 > 内置默认值

支持配置文件的字段都按同一套顺序合并。CLI 里的显式参数永远优先；没有 CLI 参数时，用配置文件；配置文件没有时，才回退到现有环境变量来源；最后才用内置默认值。

对语言字段，环境来源继续沿用现有 locale 检测顺序：`LC_ALL`，再 `LC_MESSAGES`，再 `LANG`。因此语言优先级会变成：`--lang` > 配置文件 `lang` > locale 环境变量 > 默认 `en`。

对 token 字段，环境来源继续沿用 `GITHUB_TOKEN`，再 `GH_TOKEN`。因此 token 优先级会变成：`--token` > 配置文件 `token` > `GITHUB_TOKEN` > `GH_TOKEN`。

Why:
- 这和当前讨论结论一致，也符合“配置文件是持久化默认值，环境变量是次一级回退”的产品语义。
- 一套统一规则比按字段分别解释更容易维护和文档化。

Alternatives considered:
- 环境变量高于配置文件：放弃，因为这与你希望“显式指定配置文件时就按配置文件来”的方向不一致。
- 让不同字段使用不同优先级：放弃，因为用户很难记，也会让帮助文档变长。

### 4. 为帮助文本语言增加一个轻量 bootstrap 解析阶段

为了让 `gh-download --help` 和无参数帮助也能吃到配置文件里的 `lang`，在完整 clap 解析前增加一个轻量 bootstrap 步骤：

1. 扫描原始参数中的 `--config` 和 `--lang`
2. 根据 `--config` 或默认路径，尝试读取配置文件里的 `lang`
3. 用 `--lang` > 配置文件 `lang` > locale 的顺序决定帮助语言
4. 再构造对应语言的 clap command 并做完整解析

完整解析后的 `resolve_cli` 再读取同一份配置，合并其他字段并生成最终 `ResolvedOptions`。

Why:
- 现在的帮助语言是在 clap 前决定的，配置文件支持会直接碰到这一层。
- 单独的 bootstrap 步骤改动范围可控，不需要把位置参数改成可选，也不必重写 clap 结构。

Alternatives considered:
- 只让下载流程吃配置文件，帮助文本忽略配置文件语言：放弃，因为这会让 `lang` 的行为前后不一致。
- 先用一种语言 parse，再二次 parse：放弃，因为路径更绕，错误信息也更难统一。

### 5. 调试输出只扩展“来源标签”，不暴露敏感值

如果 token 来自配置文件，调试输出中的 token source 标签可以新增 `config`，但绝不能打印 token 内容。README 需要补充说明：若用户把 token 写进配置文件，应自行控制文件权限。

Why:
- token 来源进入配置文件后，调试输出仍需要可解释性。
- 产品合同一直没有输出 token 明文，这条边界不能放松。

## Risks / Trade-offs

- [帮助文本语言需要在完整解析前就决定] -> 用一个只关心 `--config` 和 `--lang` 的 bootstrap 步骤，把影响面限制在解析入口。
- [配置文件支持会让参数来源更多，排错更复杂] -> 统一优先级，并在帮助文档中列出默认路径、`--config` 语义和来源顺序。
- [未知键静默通过会埋下配置错误] -> 配置解析使用严格 schema，对未知键、非法枚举值、非法并发值直接报错。
- [token 写入配置文件会增加本地泄露面] -> 不在输出中回显 token，并在文档中明确提示文件权限风险。

## Migration Plan

1. 新增 OpenSpec `cli-config-file` capability，并更新相关 delta specs。
2. 在 CLI 类型中加入 `--config`，新增配置文件加载与 bootstrap 语言判定逻辑。
3. 在选项解析层按既定优先级合并配置文件、环境变量和默认值。
4. 更新帮助文本、README.md、README.zh.md 和相关测试。
5. 运行 `just fmt`、`just test`、`just check`。

Rollback strategy:
- 如果配置文件接入后让帮助解析路径变得过于脆弱，可在发布前回退 `--config` 和默认配置支持，只保留内部实验代码，不对外承诺合同。

## Open Questions

- None currently.
