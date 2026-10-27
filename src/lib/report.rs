use crate::providers::http;

#[derive(Debug, Clone)]
pub enum Report {
    Http(http::Report),
    Error(ErrorReport),
}

#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub id: String,
    pub error: String,
    pub req_start: u128,
    pub req_end: u128,
}

impl Report {
    pub fn id(&self) -> &str {
        match self {
            Report::Http(r) => &r.id,
            Report::Error(r) => &r.id,
        }
    }
}