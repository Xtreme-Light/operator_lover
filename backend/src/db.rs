use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared::{ApiResponse, DbTarget, QueryRequest, TableData};
use sqlx::{Column, MySql, Row, TypeInfo};

use crate::state::AppState;

/// 通用 SQL 查询接口（仅允许 SELECT，避免误操作）
pub async fn query(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> Response {
    let sql_trim = req.sql.trim();
    if !sql_trim.to_lowercase().starts_with("select")
        && !sql_trim.to_lowercase().starts_with("show")
        && !sql_trim.to_lowercase().starts_with("desc")
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<TableData>::err("仅允许 SELECT/SHOW/DESC 语句")),
        )
            .into_response();
    }

    let pool = match req.target {
        DbTarget::Mysql => state.mysql.clone(),
        DbTarget::Goldendb => state.goldendb.clone(),
    };

    let pool = match pool {
        Some(p) => p,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<TableData>::err(format!(
                    "目标数据库 {:?} 未配置或不可用",
                    req.target
                ))),
            )
                .into_response();
        }
    };

    match run_query(pool.as_ref(), sql_trim).await {
        Ok(table) => Json(ApiResponse::ok(table)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<TableData>::err(format!("查询失败: {e}"))),
        )
            .into_response(),
    }
}

/// 健康检查（顺便检查每个数据库是否可达）
pub async fn db_status(State(state): State<AppState>) -> Json<ApiResponse<serde_json::Value>> {
    let mysql_ok = match &state.mysql {
        Some(p) => sqlx::query("SELECT 1").execute(p.as_ref()).await.is_ok(),
        None => false,
    };
    let golden_ok = match &state.goldendb {
        Some(p) => sqlx::query("SELECT 1").execute(p.as_ref()).await.is_ok(),
        None => false,
    };

    Json(ApiResponse::ok(serde_json::json!({
        "mysql": mysql_ok,
        "goldendb": golden_ok,
    })))
}

async fn run_query(pool: &sqlx::Pool<MySql>, sql: &str) -> anyhow::Result<TableData> {
    let rows = sqlx::query(sql).fetch_all(pool).await?;

    let mut table = TableData::default();
    if let Some(first) = rows.first() {
        table.columns = first
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();
    }

    for row in &rows {
        let mut record: Vec<String> = Vec::with_capacity(row.columns().len());
        for (i, col) in row.columns().iter().enumerate() {
            let val = decode_cell(row, i, col.type_info().name());
            record.push(val);
        }
        table.rows.push(record);
    }

    Ok(table)
}

/// 简单将常用 MySQL 类型转 String，便于通用展示
fn decode_cell(row: &sqlx::mysql::MySqlRow, idx: usize, ty: &str) -> String {
    macro_rules! try_decode {
        ($t:ty) => {
            if let Ok(v) = row.try_get::<Option<$t>, _>(idx) {
                return v.map(|x| x.to_string()).unwrap_or_default();
            }
        };
    }

    match ty {
        "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" => {
            try_decode!(i64);
        }
        "BIGINT" => {
            try_decode!(i64);
        }
        "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "INT UNSIGNED" | "MEDIUMINT UNSIGNED"
        | "BIGINT UNSIGNED" => {
            try_decode!(u64);
        }
        "FLOAT" => {
            try_decode!(f32);
        }
        "DOUBLE" | "DECIMAL" | "NUMERIC" => {
            try_decode!(f64);
        }
        "BOOLEAN" | "BOOL" => {
            try_decode!(bool);
        }
        "DATE" => {
            try_decode!(chrono::NaiveDate);
        }
        "TIME" => {
            try_decode!(chrono::NaiveTime);
        }
        "DATETIME" | "TIMESTAMP" => {
            try_decode!(chrono::NaiveDateTime);
        }
        _ => {}
    }
    // 默认按字符串解析
    if let Ok(v) = row.try_get::<Option<String>, _>(idx) {
        return v.unwrap_or_default();
    }
    if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(idx) {
        return v
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
    }
    String::from("<unsupported>")
}
