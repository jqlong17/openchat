# 当前状态

**阶段**：**MVP 阶段 1（账户与鉴权）** 已在代码中落地：SQLx + SQLite、`users` 迁移、注册/登录/refresh、`GET /api/v1/me`、Argon2、JWT、集成测试；OpenAPI 由 **utoipa** 生成（`GET /api/v1/openapi.json` + 根目录 `openapi.json` 快照）。

**Done**：阶段 0 全部；阶段 1 路由与测试（`tests/auth.rs`、`tests/common`）；CI 仍跑 `cargo fmt` / `clippy` / `test`。

**Next**：按 [03-plan/01plan.md](../03-plan/01plan.md) **阶段 2**（会话与消息 REST）。

**GitHub**：<https://github.com/jqlong17/openchat>
