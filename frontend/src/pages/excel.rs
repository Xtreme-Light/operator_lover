use leptos::*;
use shared::{ApiResponse, ExcelParsed, TableData};
use web_sys::FormData;

use crate::components::DataTable;

#[component]
pub fn ExcelPage() -> impl IntoView {
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(String::new());
    let (parsed, set_parsed) = create_signal(ExcelParsed::default());
    let (active, set_active) = create_signal(0usize);

    let input_ref = create_node_ref::<html::Input>();

    let on_upload = move |_| {
        let Some(input_el) = input_ref.get() else { return; };
        let Some(files) = input_el.files() else { return; };
        if files.length() == 0 {
            set_error.set("请先选择文件".into());
            return;
        }
        let Some(file) = files.get(0) else { return; };

        let form = match FormData::new() {
            Ok(f) => f,
            Err(_) => {
                set_error.set("无法构造 FormData".into());
                return;
            }
        };
        if form.append_with_blob_and_filename("file", &file, &file.name()).is_err() {
            set_error.set("追加文件失败".into());
            return;
        }

        set_loading.set(true);
        set_error.set(String::new());

        spawn_local(async move {
            let result = upload_form(form).await;
            match result {
                Ok(resp) => {
                    if resp.code == 0 {
                        set_parsed.set(resp.data.unwrap_or_default());
                        set_active.set(0);
                    } else {
                        set_error.set(resp.message);
                    }
                }
                Err(e) => set_error.set(e),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="card">
            <h2>"Excel 解析"</h2>
            <p class="muted">"支持 .xlsx / .xls / .ods / .xlsb，第一行作为表头。"</p>
            <div class="row">
                <input type="file" accept=".xlsx,.xls,.ods,.xlsb" node_ref=input_ref/>
                <button on:click=on_upload disabled=move || loading.get()>
                    {move || if loading.get() { "上传中..." } else { "上传并解析" }}
                </button>
            </div>
            {move || {
                let e = error.get();
                if e.is_empty() { view! { <span/> }.into_view() }
                else { view! { <div class="error">{e}</div> }.into_view() }
            }}
        </div>

        <Show
            when=move || !parsed.get().sheets.is_empty()
            fallback=|| view! { <div/> }
        >
            <div class="card">
                <h2>"解析结果"</h2>
                <div class="tabs">
                    {move || parsed.get().sheets.iter().enumerate().map(|(i, s)| {
                        let name = s.name.clone();
                        let is_active = active.get() == i;
                        view! {
                            <button
                                class=if is_active { "active" } else { "" }
                                on:click=move |_| set_active.set(i)
                            >{name}</button>
                        }
                    }).collect_view()}
                </div>
                {move || {
                    let p = parsed.get();
                    let i = active.get().min(p.sheets.len().saturating_sub(1));
                    let sheet_data: TableData = p.sheets.get(i).map(|s| s.data.clone()).unwrap_or_default();
                    view! { <DataTable data=sheet_data/> }
                }}
            </div>
        </Show>
    }
}

async fn upload_form(form: FormData) -> Result<ApiResponse<ExcelParsed>, String> {
    let resp = gloo_net::http::Request::post("/api/excel/upload")
        .body(form)
        .map_err(|e| format!("构造请求失败: {e}"))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;
    resp.json::<ApiResponse<ExcelParsed>>()
        .await
        .map_err(|e| format!("解析响应失败: {e}"))
}
