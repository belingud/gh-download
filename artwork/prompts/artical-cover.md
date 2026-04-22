---
type: cover
palette: cool
rendering: digital
---

# Content Context
Article title: 我经常只想从 GitHub 仓库里拿几个文件或某个目录，于是做了 `gh-download`
Content summary: 这张封面强调 `gh-download` 的核心动作：按 GitHub 仓库路径只取真正需要的那一部分内容，不必先 clone 整个仓库。画面把“仓库树里选中路径”到“终端执行下载”再到“本地目标写入完成”连成一条清晰流程，同时把文件下载、目录递归、终端优先、直接写盘这些特性放在左侧标签区。整体语气克制、务实，重点是把一个常见动作变成一条明确的命令。
Keywords: gh-download, GitHub, 文件下载, 目录递归, 仓库路径, 命令行工具, repository tree, terminal, local target

# Visual Design
Cover theme: GitHub 路径直取
Type: hero
Palette: cool
Rendering: digital
Font: clean
Text level: title-subtitle-plus-ui-copy
Mood: balanced
Aspect ratio: 2.35:1
Language: zh

# Text Elements
Title: gh-download
Subtitle: GitHub 路径下载工具
Top label: GITHUB PATH DOWNLOAD
Body copy:
- 按仓库路径下载文件或目录，省掉 zip、解压和手工整理。
- 先判断目标类型，再把真正需要的那一部分直接写到本地。
Feature pills:
- 文件下载
- 目录递归
- 终端优先
- 直接落盘
Example card:
- 只下载部分文件
- gh-download owner/repo path ./target
Repository panel text:
- repository tree
- owner/repo
- branch: main
- README.md
- skills/baoyu-translate/
- docs/guide.md
- .github/workflows/ci.yml
- selected
- path
Callout bubble:
- 只拿真正要用的那一部分，不必先 clone 整个仓库。
- files + folders + paths
Terminal panel text:
- terminal
- > gh-download owner/repo docs ./downloads
- 识别目标类型 · 递归写入本地 · 保留相对路径
- file
- directory
- stream
Footer and target label:
- gh-download
- GitHub 路径下载工具
- local target

# Mood Application
Use medium contrast, restrained saturation, and calm visual weight. The image should feel precise, technical, and dependable rather than dramatic.

# Font Application
Use clean sans-serif typography with a product-poster feel. Chinese text should be steady and readable; terminal and interface labels should use a clear monospace font.

# Composition
Type composition:
- Place a large left title block with generous empty space, then use the center as a diagonal transfer path, and reserve the right half for structured interface panels.

Visual composition:
- Main visual: an upper-right repository tree window shows several items, with `skills/baoyu-translate/` marked as selected and `docs/guide.md` marked as path. Three bright transfer lines and one soft beam flow from that panel into a lower-right terminal panel.
- Layout: top-left small rounded label, oversized `gh-download` title beneath it, subtitle and two-line description in the left-middle area, a row of four rounded feature pills below, and a lower-left example command card.
- Supporting panels: add a top-right speech-bubble card overlapping the repository panel, a lower-right terminal card with command and three status chips, and a checked folder icon labeled `local target` at the far right edge.
- Decorative: dark navy grid background, layered blue gradients, cyan glow spots, rounded panels with subtle shadows, thin orbit lines behind the right-side panels, and crisp interface borders.

Color scheme: deep navy background, layered steel-blue panels, cyan and sky-blue highlights, soft white typography, and small amber window-control accents. Color constraint: Color values and palette labels are guidance only; do not render palette names or raw color codes as visible text.
Rendering notes: polished digital illustration with crisp edges, layered gradients, soft glows, structured UI cards, clean drop shadows, and no people or photographic textures.
Type notes: preserve a poster-like hierarchy, but let the interface copy remain fully readable because the SVG includes many explicit labels and command examples.
Palette notes: cool engineering blues with cyan accents, stable and product-oriented, with no flashy neon overload.
