# 02 OpenClaw 本地代码技术调研

**本地路径**：`/Users/ruska/开源项目/openclaw`  
**远程**：<https://github.com/openclaw/openclaw>  
**说明**：以下基于**本机克隆目录**的目录与文件阅读整理；版本以 `git log -1` 为准，升级后行为以官方文档为准。

---

## 1. 项目定位（与仓库自述一致）

OpenClaw 在 `package.json` 中的描述为：**Multi-channel AI gateway with extensible messaging integrations**（多通道 AI 网关 + 可扩展消息集成）。README 将其定位为在用户自有设备上运行的 **个人 AI 助手（Personal AI Assistant）**：通过用户已使用的各类通讯渠道（WhatsApp、Telegram、Slack、Discord、Matrix、飞书等）收发信息，并驱动 **Agent** 完成任务；强调 **Gateway 为控制面**，产品形态是「助手」本身。

官方文档入口：<https://docs.openclaw.ai> · 站点：<https://openclaw.ai>

---

## 2. 技术栈与运行形态

| 类别 | 内容 |
|------|------|
| **语言 / 运行时** | **TypeScript**，构建产物在 `dist/`；CLI 入口 `openclaw.mjs`；**Node 24（推荐）或 Node 22.16+** |
| **包管理** | 仓库内开发推荐 **pnpm**（`pnpm install`、`pnpm build`）；亦支持 npm/bun 等（见 README） |
| **主入口** | `package.json` 中 `"main": "dist/index.js"`；源码在 `src/`，含 `entry.ts`、CLI、`channels`、`agents`、`config` 等 |
| **许可证** | **MIT**（`LICENSE`） |
| **版本** | `package.json` 中 `version` 字段为日历式版本（如 `2026.3.14`），与 npm 发布一致 |

从源码构建的典型流程（README）：`pnpm install` → `pnpm ui:build`（首次 UI）→ `pnpm build`；开发可用 `pnpm gateway:watch`。

---

## 3. 仓库顶层结构（与本项目集成相关）

| 路径 | 作用（概要） |
|------|----------------|
| **`src/`** | 核心逻辑：网关、通道抽象、Agent、CLI、配置、自动回复、沙箱与浏览器等 |
| **`extensions/`** | **按通道拆分的扩展包**（数十个目录），每个通道常以独立 `package.json` + `openclaw.extensions` 声明入口；例如 `extensions/telegram`、`extensions/discord`、`extensions/matrix` 等 |
| **`packages/`** | 与子项目/工具链相关的包（如 `clawdbot`、`moltbot` 等，需按需阅读） |
| **`ui/`** | Web UI 前端资源 |
| **`docs/`、`docs-site/`** | 文档与文档站 |
| **`apps/`** | macOS / iOS / Android 等客户端相关工程 |
| **`skills/`** | 与助手技能相关的打包内容（随 npm `files` 发布） |
| **`test/`** | 测试与 E2E |

**通道数量**：`extensions/` 下独立扩展包约 **80+**（目录数），覆盖 IM、邮件、团队协作、部分模型提供商等。

---

## 4. 扩展 / Channel 模型（OpenChat 对接的关键）

每个通道扩展通常在 **`extensions/<name>/package.json`** 中通过 **`openclaw`** 字段注册，例如 Telegram：

- **`openclaw.extensions`**：指向入口 TS 文件（如 `./index.ts`）
- **`openclaw.channel`**：`id`、`label`、`docsPath` 等元数据，供 CLI/向导与文档索引
- **`openclaw.setupEntry`**：可选，安装/配置向导入口

官方文档（中文）在 `docs/zh-CN/tools/plugin.md` 等处说明：**原生扩展**、**工作区插件**路径（如 `~/.openclaw/extensions`）、以及与 **gateway** 认证、`channels` 状态命令等的关系。

**对 OpenChat 的启示**：若要让 OpenClaw「认识」OpenChat，工程上最贴近现有模式的是：实现一个 **新的 channel 扩展**（类比 `telegram`），在扩展内完成：

- 入站：接收来自 OpenChat 的 HTTP Webhook 或长连接事件，转为 OpenClaw 内部统一消息；
- 出站：将 Agent 回复通过 OpenChat 提供的 **Bot/API** 写回会话。

同时需对齐 **`plugin-sdk`**：`package.json` 中大量 `./plugin-sdk/*` 导出，涵盖 **channel-runtime、reply-runtime、config-runtime** 等，集成时应阅读对应类型定义与现有通道实现。

---

## 5. 安全与运维（阅读 README 的要点）

- 入站 DM 视为**不可信输入**；默认对多通道启用 **DM pairing**（陌生人需配对码，避免未授权驱动 Agent）。
- 生产部署需结合官方 **Security** 文档与 `SECURITY.md`。
- CLI 提供 `openclaw onboard`、`openclaw gateway`、`openclaw agent`、`openclaw message send` 等（README 快速开始），便于验证端到端链路。

---

## 6. 与 OpenChat 集成的调研结论（技术向）

| 方向 | 建议 |
|------|------|
| **对齐对象** | 优先阅读 **`extensions/`** 中与「类 IM」最接近的通道（如 Telegram、Matrix、Slack）的 **`index.ts` 与 channel 元数据**，抽象 inbound/outbound 契约 |
| **扩展形态** | 使用仓库提供的 **plugin-sdk / channel runtime**，避免 fork 核心网关大量复制代码 |
| **协议层** | OpenChat 侧需提供稳定 **HTTP API + 鉴权**（如 Bot Token）与可选 **Webhook**；与 OpenClaw 的配置项（base URL、secret）一一对应 |
| **验证路径** | 本地 `pnpm build` 后跑 **gateway**，用最小消息环（OpenChat → Webhook → OpenClaw → 回发 API）验证 |

---

## 7. 修订记录

| 版本 | 日期 | 说明 |
|------|------|------|
| 0.1 | 2026-03-22 | 初稿：基于本地 openclaw 目录结构与技术栈阅读 |
