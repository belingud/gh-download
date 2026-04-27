## Context

本次变更面向仓库入口，不改变 CLI 参数、下载行为、代理行为或运行时输出。当前仓库已有中英文 README、MIT 许可证、CI 和 Release 工作流，但协作入口文件和模板缺失；Release 归档只包含英文 README 和许可证，中文用户下载二进制包后看不到中文说明。

这类改动适合放在仓库内版本化文件中维护，避免依赖 GitHub 设置项。仓库设置、分支保护、Wiki、Discussions、Dependabot 开关不纳入本次变更。

## Goals / Non-Goals

**Goals:**
- 增加轻量的贡献、安全、支持和行为准则文件，让外部用户知道如何反馈问题、提交修改和报告安全问题。
- 增加 Issue 和 PR 模板，让反馈包含排查 CLI 下载问题所需的关键信息。
- 优化 README 首屏，让新用户更快看到项目定位、状态、安装入口和最短使用示例。
- 保持中英文 README 语义一致。
- 让 Release 归档包含 `README.zh.md`。

**Non-Goals:**
- 不调整 GitHub 仓库设置。
- 不新增 Homebrew、Scoop、Winget、Chocolatey 等安装渠道。
- 不改变 CLI 合同、帮助文本、下载规则、JSON 输出或错误文案。
- 不引入新依赖。

## Decisions

### 1. 使用少量顶层协作文件

新增 `CONTRIBUTING.md`、`SECURITY.md`、`SUPPORT.md` 和 `CODE_OF_CONDUCT.md`。这些文件放在仓库根目录，GitHub 能直接识别，用户在克隆仓库后也容易看到。

Why:
- 这个项目规模小，根目录文件比单独文档站更直接。
- GitHub community profile 会识别这些文件，开源协作信号更完整。

Alternatives considered:
- 把协作文档放进 `docs/`：放弃，因为当前仓库没有独立 docs 目录，入口会更深。
- 只在 README 中写贡献说明：放弃，因为 GitHub 对贡献、安全和行为准则有独立入口，单独文件更稳定。

### 2. Issue 模板使用 GitHub issue forms

新增 bug report 和 feature request 两个 YAML 表单。bug 表单收集版本、系统、安装方式、命令、远端路径、本地目标、认证方式、代理模式、期望结果、实际结果和已打码日志。feature 表单聚焦使用场景、期望行为和替代方案。

Why:
- CLI 下载问题通常和系统、网络、认证、代理和路径有关，结构化字段能减少往返追问。
- issue forms 比纯 Markdown 模板更适合必填字段。

Alternatives considered:
- 只写一个通用模板：放弃，因为 bug 和 feature 所需信息不同。
- 增加更多模板：暂不采用，简单项目不需要过多选择。

### 3. PR 模板保持短清单

PR 模板只要求说明变更内容、验证方式、是否影响 CLI 行为、是否更新中英文文档和 OpenSpec。它不要求复杂流程，也不加入仓库设置类检查项。

Why:
- 项目小，模板太长会妨碍贡献。
- 这个仓库把用户可见行为视为产品合同，PR 模板需要提醒文档和 OpenSpec 同步。

### 4. README 首屏只做入口优化

README 顶部增加状态徽章和 quick start，保留现有详细配置说明。首屏信息只回答四件事：项目是什么、为什么用、怎样安装、怎样运行最短命令。

Why:
- README 已经有完整细节，问题在于新用户需要更快建立判断。
- 不新增安装渠道，避免和用户已有发布安排冲突。

### 5. Release 归档加入中文 README

Release 工作流的 Unix 和 Windows 打包步骤都复制 `README.zh.md`。资产命名、目标平台矩阵、校验文件和发布触发条件保持不变。

Why:
- 项目已经承诺英文和中文用户友好，二进制包也应包含两份 README。
- 这只改变归档内容，不改变发布流程结构。

## Risks / Trade-offs

- [模板过长导致用户不愿填写] → 只保留排查 CLI 下载问题必需的信息，描述尽量短。
- [README 首屏信息和后文重复] → 首屏只放 quick start，详细参数仍留在后文。
- [安全报告入口可能让用户误以为有正式安全响应团队] → `SECURITY.md` 明确维护者响应能力和报告方式，不做超出项目规模的承诺。
- [Release 归档内容变化影响校验值] → 新版本发布时重新生成 `checksums.txt`，这是正常发布行为。

## Migration Plan

1. 新增 OpenSpec capability 和 Release 归档 delta spec。
2. 增加协作文件、Issue/PR 模板。
3. 更新中英文 README 首屏。
4. 更新 Release 打包步骤以包含 `README.zh.md`。
5. 运行格式化和测试命令，确认 Rust 代码未受影响。

Rollback strategy:
- 若模板或文档入口不合适，可在后续变更中删除或简化这些文件；Release 归档也可恢复只包含英文 README 和许可证。

## Open Questions

- None currently.
