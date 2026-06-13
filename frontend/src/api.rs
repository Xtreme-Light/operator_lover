use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shared::ApiResponse;

const BASE: &str = "/api";

pub async fn get_json<T: DeserializeOwned>(path: &str) -> Result<ApiResponse<T>, String> {
    let url = format!("{BASE}{path}");
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;
    resp.json::<ApiResponse<T>>()
        .await
        .map_err(|e| format!("解析响应失败: {e}"))
}

pub async fn post_json<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<ApiResponse<T>, String> {
    let url = format!("{BASE}{path}");
    let resp = Request::post(&url)
        .json(body)
        .map_err(|e| format!("序列化失败: {e}"))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;
    resp.json::<ApiResponse<T>>()
        .await
        .map_err(|e| format!("解析响应失败: {e}"))
}
