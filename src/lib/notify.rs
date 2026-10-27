use anyhow::Result;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Request};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client as HyperClient;
use hyper_util::rt::TokioExecutor;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use crate::providers::http::Report as HttpReport;
use crate::report::{ErrorReport, Report};
use crate::tasks::{Notify, NotifyFormat, SendMode};
use crate::utils::timestamp::{current_timestamp, format_timestamp};

type NotifyClient = HyperClient<HttpsConnector<HttpConnector>, Full<Bytes>>;

const NOTIFY_TIMEOUT_SECS: u64 = 5;

pub fn matches(mode: SendMode, report: &Report) -> bool {
    match report {
        Report::Http(http_report) => {
            if http_report.status_code == 500 {
                return true;
            }

            match mode {
                SendMode::All => true,
                SendMode::Non200 => http_report.status_code != 200,
                SendMode::Errors => false,
            }
        }
        Report::Error(_) => true,
    }
}

pub async fn send_notify(notify: &Notify, report: &Report) -> Result<()> {
    let payload = payload(notify, report);
    let notify_client: NotifyClient =
        HyperClient::builder(TokioExecutor::new()).build(HttpsConnector::new());

    let mut builder = Request::builder()
        .uri(notify.url.as_str())
        .method(Method::from_str(&notify.method)?);

    for (key, value) in notify.headers.iter() {
        builder = builder.header(key.as_str(), value.as_str());
    }

    let request = builder.body(Full::new(Bytes::from(payload)))?;

    let response = tokio::time::timeout(
        Duration::from_secs(NOTIFY_TIMEOUT_SECS),
        notify_client.request(request),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Notify request timed out"))?
    .map_err(|e| anyhow::anyhow!("Notify request failed: {}", e))?;

    if !response.status().is_success() {
        anyhow::bail!("Notify request returned status {}", response.status());
    }

    Ok(())
}

fn payload(notify: &Notify, report: &Report) -> String {
    if let Some(template) = &notify.template {
        return render_template(template, report);
    }

    match notify.format {
        NotifyFormat::Plaintext => plaintext(report),
        NotifyFormat::Json => json(report),
    }
}

const PLACEHOLDERS: [&str; 7] = [
    "%ReqTime",
    "%ReqStart",
    "%ReqEnd",
    "%LoggedAt",
    "%Status",
    "%Error",
    "%ID",
];

fn render_template(template: &str, report: &Report) -> String {
    let (id, status, error) = match report {
        Report::Http(http_report) => (http_report.id.clone(), http_report.status_code.to_string(), String::new()),
        Report::Error(error_report) => (error_report.id.clone(), error_report.error.clone(), error_report.error.clone()),
    };
    let (req_start, req_end) = match report {
        Report::Http(http_report) => (http_report.req_start, http_report.req_end),
        Report::Error(error_report) => (error_report.req_start, error_report.req_end),
    };

    let mut values: HashMap<&str, String> = HashMap::new();
    values.insert("%ID", id);
    values.insert("%Status", status);
    values.insert("%ReqTime", format!("{}ms", req_end - req_start));
    values.insert("%ReqStart", req_start.to_string());
    values.insert("%ReqEnd", req_end.to_string());
    values.insert("%Error", error);
    values.insert("%LoggedAt", current_timestamp().to_string());

    substitute(template, &values)
}

fn substitute(template: &str, values: &HashMap<&str, String>) -> String {
    let mut out = String::new();
    let mut rest = template;

    while let Some(pos) = rest.find('%') {
        out.push_str(&rest[..pos]);

        let candidate = &rest[pos..];
        let placeholder = PLACEHOLDERS.iter().find(|p| candidate.starts_with(**p));

        match placeholder {
            Some(name) => out.push_str(values.get(name).map(String::as_str).unwrap_or("")),
            None => out.push('%'),
        }

        let consumed = placeholder.map_or(1, |p| p.len());
        rest = &rest[pos + consumed..];
    }

    out.push_str(rest);
    out
}

fn plaintext(report: &Report) -> String {
    match report {
        Report::Http(http_report) => format!(
            "{} | {} | {} | {}ms",
            format_timestamp(current_timestamp()),
            http_report.id,
            http_report.status_code,
            http_report.req_end - http_report.req_start,
        ),
        Report::Error(error_report) => format!(
            "{} | {} | {} | {}ms",
            format_timestamp(current_timestamp()),
            error_report.id,
            error_report.error,
            error_report.req_end - error_report.req_start,
        ),
    }
}

fn json(report: &Report) -> String {
    let common = |id: &str, req_start: u128, req_end: u128| {
        json!({
            "id": id,
            "req_start": req_start,
            "req_end": req_end,
            "logged_at": current_timestamp(),
        })
    };

    match report {
        Report::Http(HttpReport {
            id,
            headers,
            status_code,
            req_start,
            req_end,
        }) => {
            let mut value = common(id, *req_start, *req_end);
            value["type"] = json!("http");
            value["status_code"] = json!(status_code);
            value["headers"] = json!(headers);
            value.to_string()
        }
        Report::Error(ErrorReport {
            id,
            error,
            req_start,
            req_end,
        }) => {
            let mut value = common(id, *req_start, *req_end);
            value["type"] = json!("error");
            value["error"] = json!(error);
            value.to_string()
        }
    }
}