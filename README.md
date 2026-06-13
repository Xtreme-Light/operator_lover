# 运维数据控制台 (Rust 全栈)

为运维场景打造的 Rust 全栈应用：处理数据、解析 Excel、连接 MySQL / GoldenDB 并在 Web 界面展示。

## 技术栈

- **Cargo workspace**：`backend` / `frontend` / `shared` 三个 crate
- **后端**：Axum 0.7 + Tokio + SQLx (mysql) + calamine (Excel 解析) + tower-http
- **前端**：Leptos 0.6 (CSR) + Trunk + gloo-net
- **数据库**：MySQL，GoldenDB 通过 MySQL 协议复用同一个驱动

## 目录结构

```
.
├── Cargo.toml          # workspace 根
├── backend/            # Axum HTTP 服务（API）
├── frontend/           # Leptos WASM 前端
└── shared/             # 前后端共享类型 (ApiResponse, TableData ...)
```

## 安装前置依赖

```bash
# Rust 工具链
rustup target add wasm32-unknown-unknown

# Trunk（前端打包/开发服务器）
cargo install trunk
```

## 配置数据库

复制示例并按需修改：

```bash
cp backend/.env.example backend/.env
```

`backend/.env` 中：

```
MYSQL_URL=mysql://user:password@host:3306/dbname
GOLDENDB_URL=mysql://user:password@host:3306/dbname
```

留空即表示不初始化对应连接池（不会阻塞启动）。

## 本地开发

打开两个终端：

```bash
# 终端 1：启动后端 (默认 :3000)
cargo run -p backend

# 终端 2：启动前端开发服务器 (默认 :8080，已配置 /api 反代到 :3000)
cd frontend
trunk serve
```

浏览器访问 <http://127.0.0.1:8080>。

## 生产构建

```bash
# 后端
cargo build -p backend --release

# 前端 (产物在 frontend/dist)
cd frontend && trunk build --release
```

可将 `frontend/dist` 静态产物用任意 Nginx / Caddy 托管，并把 `/api` 反代到后端 `:3000`。

## 已实现功能

- `GET /api/health`：健康检查
- `GET /api/db/status`：返回 MySQL / GoldenDB 当前连通状态
- `POST /api/db/query`：执行 SELECT / SHOW / DESC 语句，返回通用表数据
  - 请求体：`{ "target": "mysql" | "goldendb", "sql": "SELECT ..." }`
- `POST /api/excel/upload`：multipart 上传 .xlsx/.xls/.ods/.xlsb，返回各 sheet 的解析结果

前端三个页面：

- `/`：首页 + 数据库连接状态总览
- `/db`：在线执行只读 SQL，按表格展示结果
- `/excel`：上传 Excel，按 sheet 切换查看
