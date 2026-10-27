use anyhow::{Context, Result};
use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::Empty;
use hyper::{Method, Request, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client as HyperClient;
use hyper_util::rt::TokioExecutor;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use url::Url;

use crate::report::{ErrorReport, Report};
use crate::tasks::{Params as TaskParams, Task};
use crate::utils::timestamp::current_timestamp;
use crate::worker::perform::Perform;

use super::{Params, Report as HttpReport};

type HttpClient = HyperClient<HttpsConnector<HttpConnector>, Empty<Bytes>>;

#[allow(dead_code)]
pub struct Provider {
    task_title: String,
    http_client: HttpClient,
    http_method: Method,
    params: Params,
    url: String,
}

impl Provider {
    pub fn new(task: Task) -> Result<Self> {
        let params = match task.params {
            TaskParams::Http(p) => p,
        };

        let url = Url::from_str(params.url.as_str())
            .context(format!("Invalid URL provided for task {}", task.id))?;

        let http_client: HttpClient = HyperClient::builder(TokioExecutor::new()).build(HttpsConnector::new());
        let http_method = Method::from_str(params.method.as_str())
            .context(format!("Invalid HTTP Method provided for task {}", task.id))?;

        Ok(Self {
            task_title: task.id,
            http_client,
            http_method,
            params,
            url: url.to_string(),
        })
    }
}

#[async_trait]
impl Perform for Provider {
    async fn perform(&self) -> Result<Report> {
        let req_start = current_timestamp();
        let request = Request::builder()
            .uri(Uri::from_str(self.url.as_str()).unwrap())
            .method(self.http_method.clone())
            .body(Empty::<Bytes>::new())
            .context(format!(
                "Failed to build Request struct on {}",
                self.task_title
            ))?;

        let response = match tokio::time::timeout(Duration::from_secs(60), self.http_client.request(request))
            .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(err)) => {
                return Ok(Report::Error(ErrorReport {
                    id: self.task_title.clone(),
                    error: describe_error(&err),
                    req_start,
                    req_end: current_timestamp(),
                }));
            }
            Err(_) => {
                return Ok(Report::Error(ErrorReport {
                    id: self.task_title.clone(),
                    error: "TIMEOUT".to_string(),
                    req_start,
                    req_end: current_timestamp(),
                }));
            }
        };
        let mut headers: HashMap<String, String> = HashMap::new();

        for (k, v) in response.headers() {
            headers.insert(k.to_string(), v.to_str().unwrap_or_default().to_string());
        }

        let http_report = HttpReport {
            id: self.task_title.clone(),
            status_code: response.status().as_u16(),
            headers,
            req_start,
            req_end: current_timestamp(),
        };

        Ok(Report::Http(http_report))
    }
}

fn describe_error(err: &dyn std::error::Error) -> String {
    let mut source = err.source();

    while let Some(cause) = source {
        if let Some(io_err) = cause.downcast_ref::<std::io::Error>() {
            return match io_err.kind() {
                std::io::ErrorKind::ConnectionRefused => "CONNECTION REFUSED".to_string(),
                std::io::ErrorKind::TimedOut => "TIMEOUT".to_string(),
                std::io::ErrorKind::ConnectionReset => "CONNECTION RESET".to_string(),
                std::io::ErrorKind::HostUnreachable => "HOST UNREACHABLE".to_string(),
                std::io::ErrorKind::NetworkUnreachable => "NETWORK UNREACHABLE".to_string(),
                _ => io_err.to_string(),
            };
        }

        source = cause.source();
    }

    err.to_string()
}
