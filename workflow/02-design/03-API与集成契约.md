# 03 API 与集成契约（设计稿）

**状态**：设计稿；实现以 **utoipa 生成的 OpenAPI**（`GET /api/v1/openapi.json`、根目录 `openapi.json` 快照）为准，本文为语义与路径约定。  
**关联**：[02-数据模型.md](./02-数据模型.md)、[01-技术架构.md](./01-技术架构.md)。

---

## 1. 全局约定

| 项 | 约定 |
|----|------|
| **Base URL** | `/api/v1` 为对外 REST 前缀（不含域名）。 |
| **Content-Type** | `application/json`；文件上传用 `multipart/form-data`。 |
| **时间** | ISO 8601 UTC，字段名 `created_at` 等。 |
| **ID** | 字符串 ID（UUID 等），与数据模型一致。 |
| **错误** | HTTP 状态码 + JSON：`{ "error": { "code": "...", "message": "..." } }` |

### 1.1 鉴权

| 场景 | 方式 |
|------|------|
| **用户** | `Authorization: Bearer <access_token>` |
| **Bot / 集成** | `Authorization: Bearer <api_key>` 或 `X-Api-Key: <api_key>`（实现二选一，文档与 OpenAPI 固定一种） |

---

## 2. 认证与用户（MVP）

| 方法 | 路径 | 说明 |
|------|------|------|
| `POST` | `/api/v1/auth/register` | 注册：`username`、`password`、`display_name` |
| `POST` | `/api/v1/auth/login` | 登录：返回 `access_token`、`refresh_token`、`expires_in` |
| `POST` | `/api/v1/auth/refresh` | 刷新访问令牌 |
| `GET` | `/api/v1/me` | 当前用户资料 |

响应中的用户对象建议包含：`id`、`username`、`display_name`、`avatar_url`（可拼自存储 key）。

---

## 3. 会话

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/v1/conversations` | 当前用户参与的会话列表 |
| `POST` | `/api/v1/conversations` | 创建群聊或发起 DM（body 含 `type`、`peer_user_id` 或成员列表） |
| `GET` | `/api/v1/conversations/{conversation_id}` | 会话详情 |
| `GET` | `/api/v1/conversations/{conversation_id}/members` | 成员列表 |

---

## 4. 消息

| 方法 | 路径 | 说明 |
|------|------|------|
| `POST` | `/api/v1/conversations/{conversation_id}/messages` | 发送消息 |
| `GET` | `/api/v1/conversations/{conversation_id}/messages` | 历史：`cursor`=`seq` 或 `before_seq` / `limit` |

### 4.1 `POST .../messages` 请求体（示例）

```json
{
  "client_msg_id": "uuid-from-client",
  "msg_type": "text",
  "body": "hello",
  "reply_to_message_id": null
}
```

### 4.2 消息资源（响应片段）

与 [02-数据模型](./02-数据模型.md) 对齐：

```json
{
  "id": "msg_uuid",
  "conversation_id": "conv_uuid",
  "seq": 42,
  "sender_id": "user_uuid",
  "sender_kind": "user",
  "msg_type": "text",
  "body": "hello",
  "client_msg_id": "uuid-from-client",
  "reply_to_message_id": null,
  "created_at": "2026-03-22T10:00:00Z"
}
```

**规则**：`sender_kind` 为用户发送时恒为 `user`；经 Bot API 写入时为 `bot`。

---

## 5. 文件上传（MVP 简版）

| 方法 | 路径 | 说明 |
|------|------|------|
| `POST` | `/api/v1/files` | `multipart`：返回 `file_id`、`url`、`mime`、`size_bytes` |
| `GET` | `/api/v1/files/{file_id}` | 下载或重定向（鉴权） |

消息中引用文件时在 `body` 或扩展 JSON 中带 `file_id` / URL（实现锁定一种）。

---

## 6. Bot API（供 OpenClaw / 自动化调用）

**鉴权**：仅 API Key；scope 至少包含 `message:write`。

| 方法 | 路径 | 说明 |
|------|------|------|
| `POST` | `/api/v1/bot/messages` | 以 Bot 身份向指定会话发消息 |

### 6.1 请求体（示例）

```json
{
  "conversation_id": "conv_uuid",
  "msg_type": "text",
  "body": "Agent 回复内容",
  "integration_meta": {
    "provider": "openclaw",
    "trace_id": "upstream-trace"
  }
}
```

服务端 **强制** 落库为 `sender_kind = bot`，并关联到该 API Key 对应的 bot 用户（见数据模型 `api_keys`）。

---

## 7. WebSocket

### 7.1 连接

- **URL**：`/api/v1/ws` 或 `/ws`（实现择一，OpenAPI 写死）。  
- **鉴权**：查询参数 `token=<access_token>` 或 `Sec-WebSocket-Protocol` 携带（实现择一）。

### 7.2 客户端 → 服务端（示例）

```json
{ "type": "ping", "id": "req-1" }
{ "type": "subscribe", "conversation_ids": ["conv_a", "conv_b"] }
```

### 7.3 服务端 → 客户端（示例）

```json
{ "type": "message.new", "conversation_id": "...", "message": { } }
{ "type": "message.stream_chunk", "conversation_id": "...", "message_id": "...", "chunk_index": 0, "content": "partial", "done": false }
{ "type": "error", "code": "...", "message": "..." }
```

事件类型表在实现阶段补全；**流式**与 [01-技术架构](./01-技术架构.md) 第 8 节一致。

---

## 8. OpenClaw 集成契约

### 8.1 OpenChat → OpenClaw（出站 Webhook）

当会话开启集成且消息满足策略（如 `sender_kind == user`、非命令过滤）时，向配置的 **`openclaw_base_url`** 发送 HTTPS POST（路径以 OpenClaw 扩展约定为准；此处定义 **OpenChat 负载**）。

**请求头（建议）**：

- `Content-Type: application/json`
- `X-OpenChat-Signature: <hmac>`（对 body 做 HMAC，密钥来自 `integration_settings`）

**请求体（逻辑模型）**：

```json
{
  "schema": "openchat.outbound.v1",
  "event_id": "uuid",
  "occurred_at": "2026-03-22T10:00:00Z",
  "conversation_id": "conv_uuid",
  "message": {
    "id": "msg_uuid",
    "seq": 42,
    "sender_id": "user_uuid",
    "sender_kind": "user",
    "msg_type": "text",
    "body": "用户说的话"
  }
}
```

**集成策略**：若 `sender_kind != user`，**不得**再次触发向 OpenClaw 的出站（防回灌）；与架构文档一致。

### 8.2 OpenClaw → OpenChat（入站）

OpenClaw 扩展调用 **§6 Bot API**，将 Agent 结果写入会话；OpenChat 仅校验 API Key 与 scope。

---

## 9. 版本与演进

- URL 前缀 **`/api/v1`** 表示破坏性变更时递增主版本。  
- `schema` 字段（如 `openchat.outbound.v1`）用于 Webhook 负载演进。

---

## 10. 修订记录

| 版本 | 日期 | 说明 |
|------|------|------|
| 0.1 | 2026-03-22 | 初稿：REST、WS、Bot、OpenClaw 出站负载 |
