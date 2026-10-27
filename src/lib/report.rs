use crate::providers::http;

#[derive(Debug)]
pub enum Report {
    Http(http::Report),
    Error(ErrorReport),
}

#[derive(Debug)]
pub struct ErrorReport {
    pub id: String,
    pub error: String,
    pub req_start: u128,
    pub req_end: u128,
}