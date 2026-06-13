use leptos::*;

use crate::api;

#[component]
pub fn HomePage() -> impl IntoView {
    let status = create_resource(
        || (),
        |_| async move { api::get_json::<serde_json::Value>("/db/status").await },
    );

    view! {
        <div class="card">
            <h2>"欢迎使用运维数据控制台"</h2>
            <p class="muted">"该控制台提供：数据库查询（MySQL / GoldenDB），以及 Excel 上传解析。"</p>
        </div>

        <div class="card">
            <h2>"后端连接状态"</h2>
            <Suspense fallback=move || view! { <p class="muted">"加载中..."</p> }>
                {move || match status.get() {
                    Some(Ok(resp)) => {
                        let v = resp.data.unwrap_or_default();
                        let mysql = v.get("mysql").and_then(|x| x.as_bool()).unwrap_or(false);
                        let golden = v.get("goldendb").and_then(|x| x.as_bool()).unwrap_or(false);
                        view! {
                            <div>
                                <p>"MySQL: " <span class=if mysql { "status ok" } else { "status bad" }>
                                    { if mysql { "已连接" } else { "未连接" } }
                                </span></p>
                                <p>"GoldenDB: " <span class=if golden { "status ok" } else { "status bad" }>
                                    { if golden { "已连接" } else { "未连接" } }
                                </span></p>
                            </div>
                        }.into_view()
                    }
                    Some(Err(e)) => view! { <p class="error">{e}</p> }.into_view(),
                    None => view! { <p class="muted">"加载中..."</p> }.into_view(),
                }}
            </Suspense>
        </div>
    }
}
