# 运维数据控制台 (Rust 全栈)

为运维场景打造的 Rust 全栈应用：处理数据、解析 Excel、连接 MySQL / GoldenDB 并在 Web 界面展示。

- **后端**：Axum 0.7 + Tokio + SQLx (mysql) + calamine (Excel 解析) + tower-http
- **前端**：Leptos 0.6 (CSR) + Trunk + gloo-net
- **共享**：`shared` crate 提供前后端通用类型 (`ApiResponse` / `TableData` / `DbTarget` ...)
- **数据库**：MySQL，**GoldenDB 通过 MySQL 协议复用同一个驱动**

---

## 目录

- [1. 目录结构](#1-目录结构)
- [2. 环境准备](#2-环境准备)
- [3. 配置 MySQL / GoldenDB 连接](#3-配置-mysql--goldendb-连接)
- [4. 本地开发](#4-本地开发)
- [5. 生产构建](#5-生产构建)
- [6. 部署方案](#6-部署方案)
- [7. systemd 守护后端](#7-systemd-守护后端)
- [8. API 速查](#8-api-速查)
- [9. 常见问题排查](#9-常见问题排查)

---

## 1. 目录结构

```
.
├── Cargo.toml          # workspace 根
├── backend/            # Axum HTTP 服务（API）
│   ├── .env.example
│   └── src/
│       ├── main.rs     # 路由 + 启动
│       ├── state.rs    # MySQL / GoldenDB 双连接池
│       ├── db.rs       # /api/db/* 处理函数
│       └── excel.rs    # /api/excel/upload 处理函数
├── frontend/           # Leptos WASM 前端
│   ├── Trunk.toml      # /api 反代到 :3000
│   ├── index.html
│   ├── styles.css
│   └── src/
│       ├── main.rs
│       ├── api.rs
│       ├── components.rs
│       └── pages/      # home / db / excel
└── shared/             # 前后端共享类型
    └── src/lib.rs
```

---

## 2. 环境准备

| 工具      | 推荐版本    | 说明                                       |
| --------- | ----------- | ------------------------------------------ |
| Rust      | ≥ 1.75      | `rustup default stable`                    |
| wasm 目标 | -           | `rustup target add wasm32-unknown-unknown` |
| Trunk     | ≥ 0.20      | `cargo install trunk`                      |
| OpenSSL   | 系统默认即可 | sqlx 用 `rustls`，默认无需系统 OpenSSL    |

一键准备：

```bash
rustup default stable
rustup target add wasm32-unknown-unknown
cargo install trunk
```

---

## 3. 配置 MySQL / GoldenDB 连接

### 3.1 复制配置模板

```bash
cp backend/.env.example backend/.env
```

[backend/.env.example](file:///home/light/WorkSpacePojects/hello_trae/backend/.env.example) 提供了所有可配置项：

```dotenv
# 后端服务监听地址
BIND_ADDR=0.0.0.0:3000

# 日志级别
RUST_LOG=info,backend=debug

# MySQL 连接 (留空则不初始化该连接池)
MYSQL_URL=mysql://user:password@127.0.0.1:3306/dbname

# GoldenDB 连接 (兼容 MySQL 协议)
GOLDENDB_URL=mysql://user:password@127.0.0.1:3306/dbname
```

### 3.2 DSN（连接串）格式

通用格式：

```
mysql://<用户名>:<密码>@<主机>:<端口>/<数据库名>?<参数>
```

常见可选参数：

| 参数              | 说明                                                |
| ----------------- | --------------------------------------------------- |
| `ssl-mode`        | `DISABLED` / `PREFERRED` / `REQUIRED` / `VERIFY_CA` |
| `socket`          | Unix 套接字路径（设置后忽略 host/port）             |
| `statement-cache-capacity` | 预编译语句缓存大小                         |

> 密码包含 `@ : / ?` 等特殊字符时，必须做 URL 编码，例如 `p@ss` 写成 `p%40ss`。

示例：

```dotenv
# 1) 普通账号密码
MYSQL_URL=mysql://ops:Ops%40123@10.0.0.10:3306/ops_db

# 2) 强制 TLS
MYSQL_URL=mysql://ops:Ops%40123@10.0.0.10:3306/ops_db?ssl-mode=REQUIRED

# 3) Unix socket 本机连接
MYSQL_URL=mysql://ops:Ops%40123@localhost/ops_db?socket=/var/run/mysqld/mysqld.sock
```

### 3.3 GoldenDB 连接说明

GoldenDB 在协议层兼容 MySQL，因此本项目复用 sqlx 的 mysql 驱动，**无需额外驱动**。

- DSN 写法与 MySQL **完全一致**，只是把 host/port 指向 GoldenDB 计算节点（CN）：

  ```dotenv
  GOLDENDB_URL=mysql://ops:Ops%40123@goldendb-cn.intranet:3308/ops_db
  ```

- 如果集群部署了多个 CN，可在前面挂一个 LVS / HAProxy / DNS 轮询地址。
- GoldenDB 默认端口因版本而异（常见 3306 / 3308 / 5258），请向 DBA 确认。
- 若 GoldenDB 启用了强一致读，可在 SQL 中使用 hint，本应用不做改写。

### 3.4 留空即跳过

任意一个变量留空或不设置，对应连接池都不会初始化，后端依然能启动；前端首页会显示该数据库为「未连接」。这样在只有 MySQL 或只有 GoldenDB 的环境下也能用。

### 3.5 准备演示账号（可选）

如果想本地快速测一下，可以在 MySQL 里建一个只读账号：

```sql
CREATE USER 'ops_ro'@'%' IDENTIFIED BY 'Ops@123';
GRANT SELECT, SHOW VIEW, PROCESS ON *.* TO 'ops_ro'@'%';
FLUSH PRIVILEGES;
```

> 后端已强制只允许 `SELECT / SHOW / DESC`，建议数据库账号也只授予只读权限，双重防护。

### 3.6 验证连通性

启动后端后访问：

```bash
curl http://127.0.0.1:3000/api/db/status
# {"code":0,"message":"ok","data":{"mysql":true,"goldendb":true}}
```

或在前端首页查看「MySQL / GoldenDB」状态徽章。

---

## 4. 本地开发

需要两个终端。

### 终端 1 — 后端

```bash
cargo run -p backend
# 监听 http://0.0.0.0:3000
```

修改后端代码后 `Ctrl+C` 重启即可；如需热重载推荐：

```bash
cargo install cargo-watch
cargo watch -x 'run -p backend'
```

### 终端 2 — 前端

```bash
cd frontend
trunk serve
# 监听 http://127.0.0.1:8080，已把 /api 反代到 :3000
```

[frontend/Trunk.toml](file:///home/light/WorkSpacePojects/hello_trae/frontend/Trunk.toml) 已配置：

```toml
[serve]
address = "127.0.0.1"
port = 8080
open = false

[[proxy]]
backend = "http://127.0.0.1:3000/api/"
```

浏览器访问 <http://127.0.0.1:8080>，三个页面：

- `/` 首页：显示数据库连接状态
- `/db` 数据库查询：选 MySQL/GoldenDB 输入 SQL（仅 SELECT/SHOW/DESC）
- `/excel` Excel 解析：上传 .xlsx / .xls / .ods / .xlsb

---

## 5. 生产构建

### 后端二进制

```bash
cargo build -p backend --release
# 产物：target/release/backend
```

### 前端静态资源

```bash
cd frontend
trunk build --release
# 产物：frontend/dist/  (含 index.html + .wasm + .js + styles.css)
```

---

## 6. 部署方案

推荐架构：**Nginx 静态托管 + 反代 `/api` 到 backend**。

### 6.1 目录布局（示例）

```
/opt/ops-console/
├── bin/backend                # 复制自 target/release/backend
├── etc/.env                   # 复制自 backend/.env
└── web/                       # 复制自 frontend/dist
```

### 6.2 启动后端

```bash
cd /opt/ops-console
set -a; source etc/.env; set +a
./bin/backend
```

或用 systemd（见下一节）。

### 6.3 Nginx 配置示例

```nginx
server {
    listen 80;
    server_name ops.example.com;

    root /opt/ops-console/web;
    index index.html;

    # SPA 兜底
    location / {
        try_files $uri $uri/ /index.html;
    }

    # WASM mime
    location ~ \.wasm$ {
        types { application/wasm wasm; }
        default_type application/wasm;
    }

    # 反代 API
    location /api/ {
        proxy_pass         http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header   Host              $host;
        proxy_set_header   X-Real-IP         $remote_addr;
        proxy_set_header   X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header   X-Forwarded-Proto $scheme;

        # Excel 上传放宽 body 上限（默认 1m）
        client_max_body_size 64m;
        proxy_read_timeout   120s;
    }
}
```

### 6.4 Docker（可选）

如果要容器化，可以两段式构建：

```dockerfile
# ---------- backend ----------
FROM rust:1.82 AS backend-build
WORKDIR /src
COPY . .
RUN cargo build -p backend --release

# ---------- frontend ----------
FROM rust:1.82 AS frontend-build
RUN rustup target add wasm32-unknown-unknown && cargo install trunk
WORKDIR /src
COPY . .
RUN cd frontend && trunk build --release

# ---------- runtime ----------
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend-build  /src/target/release/backend /app/backend
COPY --from=frontend-build /src/frontend/dist          /app/web
WORKDIR /app
ENV BIND_ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["/app/backend"]
```

> 静态资源也可由后端直出（用 `tower-http::services::ServeDir`），如有需要再加。

---

## 7. systemd 守护后端

新建 `/etc/systemd/system/ops-console.service`：

```ini
[Unit]
Description=Ops Console Backend
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=ops
WorkingDirectory=/opt/ops-console
EnvironmentFile=/opt/ops-console/etc/.env
ExecStart=/opt/ops-console/bin/backend
Restart=on-failure
RestartSec=3
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

启用：

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now ops-console
sudo systemctl status ops-console
journalctl -u ops-console -f
```

---

## 8. API 速查

| 方法 | 路径                | 说明                                       |
| ---- | ------------------- | ------------------------------------------ |
| GET  | `/api/health`       | 健康检查                                   |
| GET  | `/api/db/status`    | 返回 MySQL / GoldenDB 连通状态             |
| POST | `/api/db/query`     | 执行只读 SQL（SELECT / SHOW / DESC）       |
| POST | `/api/excel/upload` | 上传 Excel 文件并解析（multipart `file`）  |

`/api/db/query` 请求体：

```json
{
  "target": "mysql",        // 或 "goldendb"
  "sql": "SELECT NOW() AS now"
}
```

统一响应包装：

```json
{ "code": 0, "message": "ok", "data": { "columns": ["..."], "rows": [["..."]] } }
```

---

## 9. 常见问题排查

| 现象                                     | 可能原因 / 处理                                                              |
| ---------------------------------------- | ---------------------------------------------------------------------------- |
| 启动日志：`MySQL 连接池初始化失败`       | DSN 错误 / 网络不通 / 账号无权限。逐个排查 host、port、user、password、库名 |
| `/api/db/status` 返回 `mysql:false`      | 配置正确但运行时被防火墙挡住，或 `wait_timeout` 已使连接断开；检查网络与权限 |
| 查询返回 `仅允许 SELECT/SHOW/DESC 语句`  | 该接口只允许只读语句，DDL/DML 请通过 DBA 通道执行                            |
| 前端访问 `/api` 404                      | 没经过 Nginx 反代，或 `Trunk.toml` proxy 地址不对                            |
| 上传 Excel 报 413 / 超时                 | Nginx `client_max_body_size` 太小；后端默认上限 32MB 可在 main.rs 调整       |
| `trunk serve` 启动慢/卡住                | 首次会编译大量 wasm 依赖，正常；之后增量编译会快                             |
| GoldenDB 连接超时但同 DSN 用 mysql 客户端可连 | 查看 sqlx 是否被 IPv6 解析坑到，可在 host 上写明确 IPv4 / 走 `127.0.0.1` |

---

## 许可证

MIT
