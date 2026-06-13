use leptos::*;
use shared::{DbTarget, QueryRequest, TableData};

use crate::api;
use crate::components::DataTable;

#[component]
pub fn DbPage() -> impl IntoView {
    let (target, set_target) = create_signal(DbTarget::Mysql);
    let (sql, set_sql) = create_signal(String::from("SELECT 1 AS hello, NOW() AS now;"));
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(String::new());
    let (table, set_table) = create_signal(TableData::default());

    let on_run = move |_| {
        let req = QueryRequest {
            target: target.get(),
            sql: sql.get(),
        };
        set_loading.set(true);
        set_error.set(String::new());
        spawn_local(async move {
            match api::post_json::<_, TableData>("/db/query", &req).await {
                Ok(resp) => {
                    if resp.code == 0 {
                        set_table.set(resp.data.unwrap_or_default());
                    } else {
                        set_error.set(resp.message);
                        set_table.set(TableData::default());
                    }
                }
                Err(e) => {
                    set_error.set(e);
                    set_table.set(TableData::default());
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="card">
            <h2>"数据库查询"</h2>
            <p class="muted">"仅支持 SELECT / SHOW / DESC 语句，避免误操作。"</p>

            <div class="row">
                <label>"目标库:"</label>
                <select on:change=move |ev| {
                    let v = event_target_value(&ev);
                    set_target.set(if v == "goldendb" { DbTarget::Goldendb } else { DbTarget::Mysql });
                }>
                    <option value="mysql">"MySQL"</option>
                    <option value="goldendb">"GoldenDB"</option>
                </select>
            </div>

            <textarea
                prop:value=move || sql.get()
                on:input=move |ev| set_sql.set(event_target_value(&ev))
            />

            <div class="row" style="margin-top:8px">
                <button on:click=on_run disabled=move || loading.get()>
                    {move || if loading.get() { "查询中..." } else { "执行查询" }}
                </button>
            </div>

            {move || {
                let e = error.get();
                if e.is_empty() { view! { <span/> }.into_view() }
                else { view! { <div class="error">{e}</div> }.into_view() }
            }}
        </div>

        <div class="card">
            <h2>"查询结果"</h2>
            {move || view! { <DataTable data=table.get()/> }}
        </div>
    }
}
