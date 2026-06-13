use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use calamine::{Data, Reader};
use shared::{ApiResponse, ExcelParsed, ExcelSheet, TableData};
use std::io::Cursor;

/// 接收上传的 Excel 文件并解析为 JSON 表格数据
pub async fn upload(mut multipart: Multipart) -> Response {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }
        let filename = field.file_name().unwrap_or("upload.xlsx").to_string();
        let bytes = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<ExcelParsed>::err(format!("读取文件失败: {e}"))),
                )
                    .into_response()
            }
        };

        return match parse_excel(&bytes) {
            Ok(parsed) => {
                tracing::info!("解析 Excel 成功: {} sheets={}", filename, parsed.sheets.len());
                Json(ApiResponse::ok(parsed)).into_response()
            }
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<ExcelParsed>::err(format!("Excel 解析失败: {e}"))),
            )
                .into_response(),
        };
    }

    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::<ExcelParsed>::err("未在表单中找到 file 字段")),
    )
        .into_response()
}

fn parse_excel(bytes: &[u8]) -> anyhow::Result<ExcelParsed> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook: calamine::Sheets<_> = calamine::open_workbook_auto_from_rs(cursor)?;
    let sheet_names = workbook.sheet_names().to_vec();

    let mut result = ExcelParsed::default();
    for name in sheet_names {
        let range = match workbook.worksheet_range(&name) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("跳过 sheet {name}: {e}");
                continue;
            }
        };

        let mut data = TableData::default();
        let mut rows_iter = range.rows();
        if let Some(header) = rows_iter.next() {
            data.columns = header.iter().map(cell_to_string).collect();
        }
        for row in rows_iter {
            data.rows.push(row.iter().map(cell_to_string).collect());
        }

        result.sheets.push(ExcelSheet { name, data });
    }
    Ok(result)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            // 整数显示为整数
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERR:{:?}", e),
    }
}
