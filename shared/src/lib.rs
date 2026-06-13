use serde::{Deserialize, Serialize};

/// 通用 API 响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".to_string(),
            data: Some(data),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            code: -1,
            message: message.into(),
            data: None,
        }
    }
}

/// 数据库目标
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DbTarget {
    Mysql,
    Goldendb,
}

/// SQL 查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub target: DbTarget,
    pub sql: String,
}

/// SQL 查询返回的通用表结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Excel 解析结果（按 sheet 分组）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExcelSheet {
    pub name: String,
    pub data: TableData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExcelParsed {
    pub sheets: Vec<ExcelSheet>,
}
