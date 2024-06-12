#[derive(serde::Serialize)]
struct Response<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorInfo>,
}

#[derive(serde::Serialize)]
struct ErrorInfo {
    code: u32,
    message: &'static str,
}

pub mod get;
pub mod reverse;
