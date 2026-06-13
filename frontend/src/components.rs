use leptos::*;
use leptos_router::A;
use shared::TableData;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="nav">
            <h1>"运维数据控制台"</h1>
            <A href="/">"首页"</A>
            <A href="/db">"数据库查询"</A>
            <A href="/excel">"Excel 解析"</A>
        </nav>
    }
}

#[component]
pub fn DataTable(data: TableData) -> impl IntoView {
    let columns = data.columns.clone();
    let rows = data.rows.clone();
    let total_rows = rows.len();
    let total_cols = columns.len();
    view! {
        <div style="overflow-x:auto">
            <table>
                <thead>
                    <tr>
                        {columns.into_iter().map(|c| view! { <th>{c}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {rows.into_iter().map(|row| {
                        view! {
                            <tr>
                                {row.into_iter().map(|cell| view! { <td>{cell}</td> }).collect_view()}
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
            <div class="muted" style="margin-top:8px">
                {format!("共 {total_rows} 行 / {total_cols} 列")}
            </div>
        </div>
    }
}
