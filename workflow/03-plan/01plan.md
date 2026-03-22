# 01 执行计划：OpenChat 服务端 MVP（总表）

**定位**：**`01plan` 是开工后的主执行计划**——从 **本仓库内初始化 Rust 工程与目录**（阶段 0），到 MVP 可演示与收尾（阶段 5），**全部落在此文**，作为进度与范围的「一页总览」。

**范围**：在单仓库 `/Users/ruska/project/openchat` 内，交付**可运行、可验收**的 Rust 服务端首版，与设计文档 [02-design](../02-design/README.md) 对齐。  
**原则**：垂直切片、每阶段可演示；**测试与契约**优先于堆功能。

**与 `02plan` 的分工**：[02plan.md](./02plan.md) 用于 **更细的任务拆解**（按 PR、按文件、按人/按日），随迭代滚动追加；**不替代**本文的阶段目标与 DoD。

---

## TDD 总则（全阶段适用）

本仓库 **MVP 执行计划默认采用测试驱动开发（TDD）**，与 `workflow/03-plan` 下各文绑定。

| 环节 | 要求 |
|------|------|
| **红** | 先增加**会失败**的自动化测试（单元或集成）：可复现的断言、必要时固定种子/临时目录。 |
| **绿** | 写**刚好能让测试通过**的实现；禁止「先写一大段再补测试」作为默认流程。 |
| **重构** | 在**全部相关测试仍绿**的前提下整理结构、命名与重复代码。 |
| **边界** | HTTP/WebSocket 以 **集成测试**（内存服务或 `127.0.0.1:0`）为主；纯逻辑用单元测试；**CI 必须执行 `cargo test`**（阶段 0 起纳入 workflow）。 |
| **例外** | 脚手架（空 `main`、仅 `Cargo.toml`）可无测试；一旦暴露行为（路由、DB），**须先有测试**。例外须在 PR 中**一句话说明理由**。 |

各阶段 **DoD** 均须包含：**本阶段新增/修改行为均有对应测试**，且 PR 前本地与 CI 测试通过。

---

## 总览

| 阶段 | 主题 | 产出（摘要） |
|------|------|----------------|
| **0** | 工程与契约 | Cargo 工程、`openapi.yaml` 草稿、健康检查、CI 占位 |
| **1** | 账户与鉴权 | 注册/登录/refresh、`/api/v1/me`、密码哈希与 JWT |
| **2** | 会话与消息（REST） | 会话 CRUD、发消息、`seq` 拉历史、幂等 `client_msg_id` |
| **3** | 实时通道 | WebSocket 订阅与 `message.new` 推送 |
| **4** | Bot 与集成骨架 | `POST /api/v1/bot/messages`、出站 Webhook（OpenClaw）可配置、防回灌 |
| **5** | 收尾 | 更新 `README`、迁移 `current-state`、补 `04-test` 验收说明 |

**依赖设计文档**：[01 技术架构](../02-design/01-技术架构.md)、[02 数据模型](../02-design/02-数据模型.md)、[03 API 契约](../02-design/03-API与集成契约.md)、[04 仓库结构](../02-design/04-代码仓库结构.md)。

---

## 阶段 0：工程与契约

### 目标

仓库内出现**可 `cargo run` 的服务**，并与 **OpenAPI 草稿** 同源对齐（见 [05-协作与演进治理](../02-design/05-协作与演进治理.md)）。

### 任务（TDD）

- [ ] 在仓库根初始化 **Rust workspace**（MVP 可先 **单 crate** `openchat-server`，与 [04](../02-design/04-代码仓库结构.md) 一致）。
- [ ] **先写**集成测试：`GET /api/v1/health`（或选定路径）期望 `200` 与 JSON 体（测试**应先失败**）。
- [ ] 增加根目录 **`openapi.yaml`**（至少含 `info`、`servers`、健康检查路径、`components/securitySchemes` 占位）。
- [ ] **再实现**路由与处理器，直至测试转绿；必要时重构。
- [ ] **`.env.example`** 补充：`DATABASE_URL`、`JWT_SECRET`（或分文件密钥路径）等变量名。
- [ ] **`.github/workflows/ci.yml`**：`cargo fmt --check`、`cargo clippy`、**`cargo test`**（阶段 0 起非空测试集至少含健康检查）。

### DoD（完成标准）

- [ ] **TDD**：健康检查路径存在**先于实现**的测试记录（同一 PR 或紧邻提交可追溯）。
- [ ] 本地执行 `cargo run` 可监听端口，`cargo test` 与 curl 健康检查均成功。
- [ ] `openapi.yaml` 能被 Swagger Editor / Redoc 打开且无语法错误。
- [ ] `README.md`（根）增加「如何启动服务端」**最少命令**（可与最终实现一致后微调）。

### 验收方式

命令行或脚本：`curl -sSf http://127.0.0.1:<port>/api/v1/health`（路径以 OpenAPI 为准）。

---

## 阶段 1：账户与鉴权

### 目标

用户可注册、登录、刷新令牌；**Bearer** 可访问 `/api/v1/me`。

### 任务（TDD）

- [ ] **先**为「注册→登录→访问 me」与「错误密码」编写失败用例（集成测试或 `#[sqlx::test]`），再补迁移与实现。
- [ ] SQLx 迁移：`users` 表（对齐 [02 数据模型 §3](../02-design/02-数据模型.md) 最小字段）。
- [ ] `POST /api/v1/auth/register`、`POST /api/v1/auth/login`、`POST /api/v1/auth/refresh`、`GET /api/v1/me`（对齐 [03](../02-design/03-API与集成契约.md)）。
- [ ] 密码 **argon2**（或项目统一算法）；JWT access + refresh 策略与过期时间写进代码常量或配置。
- [ ] 补充 **refresh 轮换**、边界错误码的测试，再重构。

### DoD

- [ ] 上述行为均有自动化测试覆盖；主路径为 **红→绿** 可追溯。
- [ ] OpenAPI 中上述路径与请求/响应模型已更新。
- [ ] 无密钥硬编码；测试使用临时目录数据库或 `#[sqlx::test]`。

### 验收方式

用 `curl` 或 `httpie` 跑通注册 → login → 带 Bearer 访问 `me`。

---

## 阶段 2：会话与消息（REST）

### 目标

同一用户可创建/列出会话，在会话内发消息，并按 **`seq`** 分页拉取历史；**`client_msg_id` 幂等**。

### 任务（TDD）

- [ ] **先**写集成测试：创建会话、发送消息、拉历史、`client_msg_id` 重复时的预期行为（首版可红）。
- [ ] 迁移：`conversations`、`conversation_members`、`messages`（含 `sender_kind`、`seq`、`client_msg_id` 唯一约束，对齐 [02 §4–5](../02-design/02-数据模型.md)）。
- [ ] API：`GET/POST /api/v1/conversations`、`GET .../messages`、`POST .../messages`（对齐 [03](../02-design/03-API与集成契约.md)）。
- [ ] 业务规则：`seq` 单调；重复 `client_msg_id` 返回同一消息或 409（实现选一种并写进 OpenAPI 说明）。
- [ ] 绿后重构；补边界用例（空会话、非法会话 ID）。

### DoD

- [ ] 幂等与 `seq` 行为有测试断言；OpenAPI 与实现一致；迁移可从空库一键应用到最新。

### 验收方式

集成测试通过 + 手动 `curl` 发两条消息并拉历史。

---

## 阶段 3：WebSocket

### 目标

客户端连接 WebSocket 后，能收到 **`message.new`** 事件（至少同会话内实时）。

### 任务（TDD）

- [ ] **先**写集成测试：建立 WS 连接（测试用 token）、REST 发消息后 **断言** 收到 `message.new`（可先红）。
- [ ] WS 路由与鉴权（query `token` 或协议约定，与 [03 §7](../02-design/03-API与集成契约.md) 一致）。
- [ ] 客户端 `subscribe`（若实现）后，REST 新消息触发下行 JSON（`type`、`conversation_id`、`message`）。
- [ ] 可选：多连接并发测试（不替代主路径 TDD）。

### DoD

- [ ] WS 行为有自动化测试（非仅手工两终端）；文档：在 `03` 或单独 `websocket-events` 中列出已实现事件类型。

### 验收方式

两个终端：一个 WS 订阅，另一个 REST 发消息，前者收到事件。

---

## 阶段 4：Bot 与 OpenClaw 集成骨架

### 目标

- Bot 可用 API Key 发消息，`sender_kind` 恒为 **`bot`**。  
- 配置开启时，**用户消息**可触发 **出站 Webhook**；**不**对 `bot` 消息再次出站（防回灌）。

### 任务（TDD）

- [ ] **先**写测试：Mock HTTP 服务端断言**仅**在 `sender_kind == user` 时收到出站请求；`bot` 消息不触发（防回灌，可先红）。
- [ ] 迁移：`api_keys`、`integration_settings`（最小字段，对齐 [02 §7–8](../02-design/02-数据模型.md)）。
- [ ] `POST /api/v1/bot/messages` + API Key 鉴权；**Bot 路径**另有单测或集成测。
- [ ] 出站：异步 HTTP POST 到配置的 `openclaw_base_url`，body 使用 [03 §8.1](../02-design/03-API与集成契约.md) 的 `openchat.outbound.v1`；失败可日志 + 重试占位（不必生产级队列）。

### DoD

- [ ] 防回灌与 Bot 写入均有测试；与 OpenClaw 真机联调可作为 **可选**。

---

## 阶段 5：收尾与入档

### 任务

- [ ] 更新根 **`README.md`**：依赖（Rust 版本）、环境变量、迁移命令、**`cargo test` 与 CI 说明**。
- [ ] 更新 **`workflow/00-rule/current-state.md`**：阶段改为「MVP 服务端已可演示」或如实填写。
- [ ] **`workflow/04-test`**：MVP 验收以 **自动化测试为主**；手工步骤仅补充 E2E/浏览器等无法覆盖部分。
- [ ] （可选）首条 **ADR**：为何 MVP 用 SQLite、JWT 策略等。

### DoD

- [ ] CI 中 **`cargo test`** 绿；新成员克隆后按 README 可跑通测试与 0→3 主路径。

---

## 风险与依赖

| 风险 | 缓解 |
|------|------|
| 框架选型拖延 | 阶段 0 结束前在 PR/ADR 中锁定 **Axum 或 Poem** 之一。 |
| OpenClaw 协议变动 | 出站负载带 `schema` 版本；连接器在 OpenClaw 侧独立迭代。 |
| SQLite 并发写 | MVP 单进程；文档标明并发上限，后续 ADR 再换存储。 |

---

## 修订记录

| 版本 | 日期 | 说明 |
|------|------|------|
| 0.1 | 2026-03-22 | 可执行 MVP：阶段 0–5 与 DoD |
| 0.2 | 2026-03-22 | 明确 01plan 为总表（含工程初始化）；细化拆至 [02plan](./02plan.md) |
| 0.3 | 2026-03-22 | 全计划按 **TDD**（红-绿-重构）；各阶段任务与 DoD 对齐；CI 从阶段 0 纳入 `cargo test` |
