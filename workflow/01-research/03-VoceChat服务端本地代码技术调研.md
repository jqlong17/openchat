# 03 VoceChat 服务端（Rust）本地代码技术调研

**本地路径**：`/Users/ruska/开源项目/vocechat-server-rust`  
**远程**：<https://github.com/Privoce/vocechat-server-rust>  
**说明**：以下基于本机克隆的 **Rust 服务端** 仓库；客户端（Flutter / Web）见官方表格，不在此目录。

---

## 1. 项目定位

VoceChat 服务端定位为 **轻量、自托管** 的社交/即时通讯后端：README 强调 **Rust** 实现、体积小、私有化部署与 **Open API** 集成；功能上覆盖 DM/群聊、文件、音视频（Agora 等）、**Bot 与 Webhook（入站/出站）** 等。  

文档站：<https://doc.voce.chat>（与本机另克隆的 `vocechat-doc` 对应）。

**许可与商业条款**：README 说明服务端许可证为 **Big Time Public License**，并区分个人免费使用与商业集成授权等——**OpenChat 若参考或衍生实现，需单独做法务审阅**，本调研不做法务结论。

---

## 2. 技术栈（自 `Cargo.toml` 与源码）

| 类别 | 内容 |
|------|------|
| **语言** | **Rust 2021**；包名 `vocechat-server`，版本号见 `Cargo.toml`（如 `0.3.3`） |
| **Web 框架** | **[Poem](https://github.com/poem-web/poem)**，启用 `rustls`、`sse`、`static-files` 等 |
| **API 描述** | **poem-openapi**：自带 OpenAPI、Swagger UI、RapiDoc、ReDoc 等 |
| **异步** | **tokio**（`macros`、`rt-multi-thread`、`sync` 等） |
| **关系库** | **sqlx** + **SQLite**（`runtime-tokio-rustls`），迁移目录 `migrations/` |
| **消息存储** | 自有 **`rc-msgdb`**（`crates/msgdb`），与 SQLite 业务库分离 |
| **序列化 / 工具** | `serde` / `serde_json`、`tracing`、`reqwest`（rustls）、`uuid` 等 |
| **其他 crate** | `crates/` 下含 `token`、`fcm`、`magic-link`、`agora-token`、`vc-license`、`open-graph` 等 |

整体上：**单进程 HTTP(S) 服务 + SQLite 元数据 + 独立消息库 + OpenAPI 化 REST**，适合作为「小型 IM 后端」参考。

---

## 3. 源码布局（概要）

| 路径 | 作用 |
|------|------|
| **`src/main.rs`** | 进程入口：解析配置、日志、`poem::Server` 监听、TLS/证书等 |
| **`src/server.rs`** | 组装路由与中间件：`Cors`、`Tracing`、注册 `api`；初始化 **`State`**（含 DB、缓存、广播等） |
| **`src/state.rs`** | 核心运行时状态：用户/群缓存、`broadcast`/`mpsc`、Webhook 转发 **`forward_chat_messages_to_webhook`** 等 |
| **`src/api/`** | 按领域拆分：`message`、`group`、`user`、`bot`、`admin_*`、`token` 等 |
| **`src/config.rs`** | 配置结构与加载（`config.toml` 等） |
| **`migrations/`** | SQLx 迁移脚本 |
| **`crates/msgdb`** | 消息落盘与查询抽象（需深入时再读） |

入口模块声明见 `main.rs`：`mod api; mod server; mod state; ...`。

---

## 4. API 与 Bot 集成要点（与 OpenChat 最相关）

- **OpenAPI**：`poem_openapi::OpenApi` 分模块挂载（如 `ApiBot`）。
- **Bot 路由**：`src/api/bot.rs` 中 **`ApiBot`** 使用前缀 **`/bot`**（`prefix_path = "/bot"`），接口包括拉取相关群、**向用户发消息**等；鉴权使用 **`x-api-key`**，并与服务端 `server_key` / 用户维度 **bot_keys** 校验（见 `check_api_key`）。
- **Webhook**：`state` 中存在向外部 **forward** 聊天消息到 Webhook 的逻辑（函数名 **`forward_chat_messages_to_webhook`**），与 README 所述「入站/出站 webhook」一致，适合与外部 Agent（如 OpenClaw）对接时对照。

阅读建议：从 **`api/bot.rs`**、**`api/message*.rs`**、**`state.rs` 中与 webhook 相关段落** 入手，梳理「一条消息从入库到推送/回调」的路径。

---

## 5. 数据与并发

- **SQLite**：业务数据（用户、群、设置等）走 **sqlx**；启动时 **`sqlx::migrate!()`** 执行迁移。
- **消息**：**MsgDb** 单独打开（`MsgDb::open`），路径在配置的数据目录下；与「元数据 DB」分离，利于大消息量时的存储策略。
- **内存缓存**：`State` 内 **`Cache`**（如 `RwLock` 包裹的 users/groups）与 **`broadcast`** 等，用于在线推送与事件广播（具体需结合 `state.rs` 全文）。

---

## 6. 与 OpenChat 的参考关系

| 维度 | VoceChat 可借鉴点 |
|------|-------------------|
| **工程** | Poem + OpenAPI 一体化、SQLite + 迁移、独立消息存储 crate |
| **产品** | Bot API Key、按用户维度的 bot_keys、`/bot` 前缀的 HTTP 接口 |
| **集成** | Webhook 出站 + API 入站发消息，与「OpenChat ↔ OpenClaw」双向链路同构 |
| **差异** | OpenChat 目标是对标微信级体验 + **开源开放**；VoceChat 另有商业许可与路线图项，**不可等同照搬** |

---

## 7. 修订记录

| 版本 | 日期 | 说明 |
|------|------|------|
| 0.1 | 2026-03-22 | 初稿：基于本地 vocechat-server-rust 目录与关键 crate/模块阅读 |
