## 1. 协作入口文件

- [x] 1.1 新增 `CONTRIBUTING.md`，说明 Issue、PR、本地验证、文档同步和 OpenSpec 同步要求
- [x] 1.2 新增 `SUPPORT.md`，说明用户应先查看 README、使用 Issues 获取支持，并避免公开敏感凭据
- [x] 1.3 新增 `SECURITY.md`，说明支持版本、安全问题报告方式和 token/凭据打码要求
- [x] 1.4 新增 `CODE_OF_CONDUCT.md`，提供简洁的社区行为准则

## 2. GitHub 模板

- [x] 2.1 新增 bug report issue form，收集版本、系统、安装方式、命令、路径、认证、代理、预期结果、实际结果和已打码日志
- [x] 2.2 新增 feature request issue form，收集使用场景、期望行为和替代方案
- [x] 2.3 新增 pull request 模板，提示变更摘要、验证方式、CLI 行为影响、文档影响和 OpenSpec 影响

## 3. README 与 Release

- [x] 3.1 优化 `README.md` 首屏，加入状态徽章、定位说明、quick start 和最短示例，同时保留已有安装方式
- [x] 3.2 优化 `README.zh.md` 首屏，保持与英文 README 语义一致
- [x] 3.3 更新 `.github/workflows/release.yml`，让 Unix 和 Windows 归档都包含 `README.zh.md`

## 4. 验证

- [x] 4.1 运行 OpenSpec 校验，确认新增和修改规格可解析
- [x] 4.2 运行 `just fmt`
- [x] 4.3 运行 `just test`
- [x] 4.4 运行 `just check`
