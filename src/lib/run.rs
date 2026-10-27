use anyhow::{Context, Error, Result};
use tokio::sync::mpsc::channel;

use crate::providers::http::Report as HttpReport;
use crate::report::{ErrorReport, Report};
use crate::tasks::adapters::from_file;
use crate::utils::color::{green, red, yellow};
use crate::utils::timestamp::{current_timestamp, format_timestamp};
use crate::worker::{Squad, Worker};

/// This is likely to change in the future in order
/// to be able to define where to gather the config
/// from. Default for now is "nasu.json"
const CONFIG_FILE_NAME: &str = "nasu.json";

pub async fn run(use_color: bool) -> Result<()> {
    let tasks =
        from_file(CONFIG_FILE_NAME).context(format!("Failed to parse \"{}\"", CONFIG_FILE_NAME))?;
    let workers: Vec<Worker> = tasks.into_iter().map(|task| Worker::from(task)).collect();
    let (tx, mut rx) = channel::<Report>(1024);
    let squad = Squad::new(workers, tx);

    let print_proc = tokio::spawn(async move {
        println!(
            "{0: <30} | {1: <15} | {2: <20} | {3: <10}",
            "Log Time", "Task", "HTTP. Status Code", "Req. Time"
        );
        println!("=====================================================================================");
        while let Some(report) = rx.recv().await {
            match report {
                Report::Http(HttpReport {
                    id,
                    req_end,
                    req_start,
                    status_code,
                    ..
                }) => {
                    println!(
                        "{0: <30} | {1: <15} | {2: <20} | {3: <10}",
                        format_timestamp(current_timestamp()),
                        id,
                        status_cell(use_color, status_code),
                        delay_cell(use_color, req_end - req_start)
                    );
                }
                Report::Error(ErrorReport {
                    id,
                    error,
                    req_start,
                    req_end,
                }) => {
                    println!(
                        "{0: <30} | {1: <15} | {2: <20} | {3: <10}",
                        format_timestamp(current_timestamp()),
                        id,
                        red(use_color, error.clone()),
                        delay_cell(use_color, req_end - req_start)
                    );
                }
            }
        }
    });

    let run_proc = tokio::spawn(async move {
        squad.start().await;
    });

    tokio::select! {
      _ = print_proc => {
        Err(Error::msg("Output process stopped"))
      },
      _ = run_proc => {
        Err(Error::msg("Run process stopped"))
      },
    }
}

fn status_cell(use_color: bool, status_code: u16) -> String {
    match status_code {
        200 => green(use_color, status_code.to_string()),
        300..=499 => yellow(use_color, status_code.to_string()),
        _ => red(use_color, status_code.to_string()),
    }
}

fn delay_cell(use_color: bool, elapsed_ms: u128) -> String {
    let value = format!("{}ms", elapsed_ms);

    match elapsed_ms {
        0..=499 => green(use_color, value),
        500..=999 => yellow(use_color, value),
        _ => red(use_color, value),
    }
}
