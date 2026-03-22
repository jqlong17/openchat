# 01 开源 IM 生态与 OpenClaw 集成调研

**记录位置**：`workflow/01-research/`  
**来源**：与 Gemini 的对话整理 + 项目方向归纳（部分数据为对话时点信息，**仓库 Star、版本、语言栈以 GitHub 页面为准**，下文不保证实时准确）。

---

## 1. 调研目的与范围

- **目的**：为 **OpenChat**（本项目）寻找可参考的开源聊天/即时通讯形态，并梳理与 **OpenClaw** 集成的架构思路。
- **范围**：企业协作类、社交 IM 类、AI 增强型、以及「自建类微信」所需的架构与协议参考；**非**「直接接入第三方 SaaS 聊天工具」的选型，而是**技术参考与自建**。
- **技术偏好（对话结论）**：后端倾向 **Rust**（轻量、高并发、单二进制分发友好）；使用 **AI Coding** 降低语言细节成本；集成 **OpenClaw** 作为 Agent/连接器生态。

---

## 2. 本地参考仓库（本机已克隆）

本机统一放在 **`/Users/ruska/开源项目/`** 下，便于与 OpenChat 仓库（`/Users/ruska/project/openchat`）区分：前者为**只读参考**，后者为**本产品源码**。更新时在对应目录执行 `git pull` 即可。

| 项目 | GitHub | 本机绝对路径 |
|------|--------|----------------|
| **OpenClaw** | [openclaw/openclaw](https://github.com/openclaw/openclaw) | `/Users/ruska/开源项目/openclaw` |
| **VoceChat 服务端（Rust）** | [Privoce/vocechat-server-rust](https://github.com/Privoce/vocechat-server-rust) | `/Users/ruska/开源项目/vocechat-server-rust` |
| **VoceChat 文档站源码** | [Privoce/vocechat-doc](https://github.com/Privoce/vocechat-doc) | `/Users/ruska/开源项目/vocechat-doc` |
| **OpenIM 服务端** | [openimsdk/open-im-server](https://github.com/openimsdk/open-im-server) | `/Users/ruska/开源项目/open-im-server` |

- **在线文档**：VoceChat 官方文档站点为 [doc.voce.chat](https://doc.voce.chat)，与 **`vocechat-doc`** 仓库对应（Docusaurus 等，本地构建方式见该仓库 README）。

---

## 3. GitHub 上常见开源「聊天/协作」形态（概览）

以下为对话中归纳的**分类线索**，具体仓库名、活跃度、协议请以 GitHub 检索为准。

### 3.1 企业级 / 团队协作（类 Slack）

| 方向 | 代表项目（线索） | 技术栈（线索） | 适用参考 |
|------|------------------|----------------|----------|
| 功能全面、可私有部署 | Rocket.Chat | JS/Meteor、Node、Mongo 等 | 全渠道、移动端、插件生态 |
| 话题/线程组织 | Zulip | Python/Django、PostgreSQL | 大量消息下的可读性 |
| 贴近开发者工具链 | Mattermost | Go、React | 与 GitLab/GitHub 等集成思路 |

### 3.2 社交 / 即时通讯（类微信/Telegram 的「IM 底座」）

| 方向 | 代表项目（线索） | 备注 |
|------|------------------|------|
| 通用 IM：单聊/群聊/音视频等 | Tinode | Go 后端；多端与协议层可参考 |
| 轻量、私有化 | 对话中另提及若干「新锐/去中心化」项目 | 需单独核实仓库与维护状态 |

### 3.3 AI 增强 / Agent 框架（与「聊天产品」相邻）

| 方向 | 说明 |
|------|------|
| OpenClaw | 对话中作为 **与本项目深度集成** 的目标；通过连接器对接 Telegram/微信等；本项目需考虑 **OpenChat 端** 与 **OpenClaw 端** 双向协议。 |
| Dify / FastGPT 等 | 更偏「AI 应用平台」；内置 Web 聊天与 API，可作智能客服/知识库对话参考，与「完整 IM 产品」定位不同。 |

---

## 4. 自建「类微信」IM：更广的参考维度

微信类产品的难点通常不在 UI，而在：**高并发长连接、消息可靠性与顺序、多端同步、弱网**。对话中将参考源分为三类。

### 4.1 架构级：通信底座与消息语义

| 项目（线索） | 定位 | 可学习点 |
|--------------|------|----------|
| OpenIM | 工业级 IM 组件（Go） | 接入/逻辑/存储分层；消息不丢、不乱序、不重复等语义 |
| Nebula 等 | 偏长连接与大规模连接 | 心跳、连接保持（需核实具体仓库） |
| Tinode | 完整 IM 服务端 | 用户/话题/订阅等模型；多存储后端 |

### 4.2 功能级：含前端或「全功能」参考

| 项目（线索） | 可学习点 |
|--------------|----------|
| VoceChat | 极简、小体积二进制、轻依赖部署；REST + WebSocket；默认 SQLite 等（**细节以官方文档与仓库为准**） |
| Element / Matrix 生态 | 加密通信、音视频与协议层（Matrix） |

### 4.3 协议与架构模式（非具体仓库）

- **消息 ID / 序列号**：分布式下唯一递增（如 Leaf、或各 IM 自研 Seq）。
- **读扩散 vs 写扩散**：大群、朋友圈类场景下的推送与存储策略。
- **弱网**：移动端长连接与重连（对话中提到微信开源组件 Mars 仅作网络层参考）。

---

## 5. VoceChat 专题（对话重点）

**为何单独关注**：极简部署、单二进制思路与 **Rust 单文件分发** 契合；适合作为「产品形态 + 工程取舍」的参考。

**对话中归纳的技术点（需对照官方实现核实）**：

- 后端：对话称服务端为 **Rust** 向的技术栈（请以官方仓库与文档为准）。
- 存储：默认 **SQLite**，利于私有化与小团队。
- 通信：**WebSocket** 实时推送 + **JSON** API。
- 工程取舍：减少 Redis/Kafka 等中间件依赖，强调「下载即用」。

**若用 Rust 自研类似形态，对话建议的模块拆分**：

- 异步运行时：**Tokio**；Web：**Axum**；WebSocket 与连接管理。
- 持久化：**SQLx**（或 SeaORM）+ SQLite。
- 序列化：**Serde**（JSON）。
- 在线用户与广播：**DashMap**、`broadcast`/`mpsc` 等（避免每用户一线程）。
- 进阶：消息先落库再推送；静态资源 **rust-embed** 嵌入前端产物；文件落盘而非进库。

---

## 6. 技术栈与 AI Coding（对话结论）

- **语言**：Rust 用于服务端时，借用 AI Coding 处理样板与迭代，重点放在 **协议、并发模型、边界条件**。
- **第一阶段可验证目标**：Axum + WebSocket Echo → 用户与会话模型 → JWT + SQLite → 私聊/群聊与历史消息 → 富媒体与上传。

---

## 7. OpenClaw 与 OpenChat 的集成形态

**目标**：OpenChat 作为可私有部署的聊天产品，**深度接入 OpenClaw**（开源代码在 GitHub，具体语言栈以克隆的仓库为准）。

### 7.1 对 OpenChat（Rust 服务端）的改造方向

- **出站**：用户消息落库后，按需 **Webhook / HTTP POST** 推给 OpenClaw（或配置的基础 URL）。
- **入站**：提供 **Bot 专用发送接口**（如对话示例 `POST /api/bot/send`），**Bearer Token** 鉴权，供 OpenClaw 将 AI 回复写回频道。
- **消息模型**：预留 **结构化扩展**（如 `extra_data` / JSON），承载工具调用、卡片、推理过程等元数据。
- **流式体验**：LLM 流式输出时，WebSocket 侧支持 **分片/增量推送**（避免整段生成后再发）。

### 7.2 对 OpenClaw 的改造方向

- 新增 **Connector（连接器）**：类比 Telegram 等现有连接器，将 OpenChat 的 JSON 协议映射为 OpenClaw 内部统一消息对象。
- **配置**：在 OpenClaw 的配置中增加 OpenChat 的 **Base URL、Token、Webhook 路径** 等。
- **入站 HTTP**：OpenClaw 侧可暴露轻量 HTTP 接收来自 OpenChat 的推送（具体形态以实现为准）。

### 7.3 架构关系（逻辑示意）

```text
用户 (Web/App) <—WebSocket/HTTP—> OpenChat Server <—SQLite/存储—> 数据
                                      │
                    Webhook / Bot API │ 双向
                                      ▼
                               OpenClaw (Agent/LLM/连接器)
```

**原则**：以 **清晰的消息契约** 为核心，尽量减少对 OpenClaw 主流程的侵入，优先 **插件化连接器**。

---

## 8. 产品命名与定位（OpenChat）

- **名称**：**OpenChat**（本项目）。
- **对话中的定位表述**：可概括为 **AI 原生、可扩展的开源通讯/协议向产品**——高性能、可私有化、与 OpenClaw 协同，将聊天作为 **人与 AI Agent** 的协作界面。
- **需在设计阶段写清**：与「纯 IM」和「纯 Agent 控制台」的边界（例如是否强依赖 OpenClaw、离线消息策略等）。

---

## 9. 风险与注意事项

| 风险 | 说明与缓解 |
|------|------------|
| **消息死循环** | AI/Bot 发出的消息若再次被转发给 OpenClaw，可能自激。缓解：**`is_bot` / `sender_type` 等字段**，连接器层忽略 Bot 回灌。 |
| **事实准确性** | 对话中的 Star 数、「2026 现象级」、个别项目名称（如 OpenClaw 别名、Haven 等）**必须**以 GitHub 与官方文档核实。 |
| **VoceChat 开源范围** | 对话称部分服务端可能偏商业/闭源；参考时需区分 **文档与客户端开源部分**，避免侵权与错误假设。 |

---

## 10. 后续调研建议（可记入 `02-design`）

1. 在 GitHub 上 **核实** OpenClaw、VoceChat、OpenIM、Tinode 等仓库的 **许可证、语言、目录结构、连接器扩展点**。
2. 起草 **OpenChat ↔ OpenClaw** 的 **OpenAPI/JSON Schema**（消息、事件、鉴权、流式 chunk 格式）。
3. 定义 **MVP**：仅私聊 + 单 Bot + 先落库后推送 + 防 Bot 回灌。

---

## 11. 修订记录

| 版本 | 日期 | 说明 |
|------|------|------|
| 0.1 | 2026-03-22 | 初稿：基于对话整理，编号 01 |
| 0.2 | 2026-03-22 | 增加「本地参考仓库」表：OpenClaw、vocechat-server-rust、vocechat-doc、open-im-server 的本机路径与链接 |
