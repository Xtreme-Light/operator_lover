use std::sync::Arc;

use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

/// 应用全局状态
#[derive(Clone)]
pub struct AppState {
    pub mysql: Option<Arc<MySqlPool>>,
    pub goldendb: Option<Arc<MySqlPool>>,
}

impl AppState {
    pub async fn from_env() -> Self {
        let mysql = match std::env::var("MYSQL_URL") {
            Ok(url) if !url.is_empty() => match build_pool(&url).await {
                Ok(pool) => {
                    tracing::info!("MySQL 连接池初始化成功");
                    Some(Arc::new(pool))
                }
                Err(e) => {
                    tracing::warn!("MySQL 连接池初始化失败: {e}");
                    None
                }
            },
            _ => {
                tracing::warn!("未设置 MYSQL_URL，跳过 MySQL 初始化");
                None
            }
        };

        let goldendb = match std::env::var("GOLDENDB_URL") {
            Ok(url) if !url.is_empty() => match build_pool(&url).await {
                Ok(pool) => {
                    tracing::info!("GoldenDB 连接池初始化成功");
                    Some(Arc::new(pool))
                }
                Err(e) => {
                    tracing::warn!("GoldenDB 连接池初始化失败: {e}");
                    None
                }
            },
            _ => {
                tracing::warn!("未设置 GOLDENDB_URL，跳过 GoldenDB 初始化");
                None
            }
        };

        Self { mysql, goldendb }
    }
}

async fn build_pool(url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(url)
        .await
}
